import warnings
from typing import Optional, Tuple, Dict, Callable, Iterable

import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.nn.modules.linear import _LinearWithBias
from torch.tensor import Tensor

from opendp.network.layers.base import InstanceGrad


class DPCatBias(nn.Module, InstanceGrad):
    """
    The biasing matrix needs a gradient.
    Instead of attempting to reconstruct the backprop chain within DPMultiHeadAttention,
    move the computation into a separate Module where hooks can be applied directly.
    """
    def __init__(self, hidden_size: int):
        self.bias = self.Parameter(torch.normal(size=hidden_size))

    def forward(self, query):
        return torch.cat([query, self.bias.repeat(1, query.size(1), 1)])

    def get_instance_grad_functions(self) -> Dict[nn.Parameter, Callable[[torch.Tensor, torch.Tensor], Iterable[torch.Tensor]]]:
        # grab the last column of the backprop for the layer, which corresponds to the cat'ed column
        return {self.bias: lambda _activation, backprop: backprop[:, -1]}


class DPMultiheadAttention(nn.Module):

    def __init__(self,
                 embed_dim: int,
                 num_heads: int,
                 dropout: float = 0.0,
                 bias: bool = True,
                 add_bias_kv: bool = False,
                 add_zero_attn: bool = False,
                 kdim: bool = None,
                 vdim: bool = None):
        """

        embed_dim: total dimension of the model.
        num_heads: parallel attention heads.
        dropout: a Dropout layer on attn_output_weights. Default: 0.0.
        bias: add bias as module parameter. Default: True.
        add_bias_kv: add bias to the key and value sequences at dim=0.
        add_zero_attn: add a new batch of zeros to the key and
                       value sequences at dim=1.
        kdim: total number of features in key. Default: None.
        vdim: total number of features in value. Default: None.

        Note: if kdim and vdim are None, they will be set to embed_dim such that
        query, key, and value have the same number of features.
        """
        super().__init__()

        self.embed_dim = embed_dim
        self.kdim = kdim if kdim is not None else embed_dim
        self.vdim = vdim if vdim is not None else embed_dim

        self.num_heads = num_heads
        self.dropout = dropout
        self.head_dim = embed_dim // num_heads
        assert self.head_dim * num_heads == self.embed_dim, "embed_dim must be divisible by num_heads"

        # in nn.MultiheadAttention, if the kdim and vdim are equal to embed_dim, these three layers are merged.
        # removed conditional merging to avoid situations where slicing of weight matrices is necessary
        # now all linear operations pass through Linear module, which have dp hooks
        self.q_proj = nn.Linear(embed_dim, embed_dim, bias=bias)
        self.k_proj = nn.Linear(embed_dim, self.kdim, bias=bias)
        self.v_proj = nn.Linear(embed_dim, self.vdim, bias=bias)

        self.out_proj = _LinearWithBias(embed_dim, embed_dim)

        if add_bias_kv:
            self.bias_k = DPCatBias(embed_dim)
            self.bias_v = DPCatBias(embed_dim)
        else:
            self.bias_k = self.bias_v = None

        self.add_zero_attn = add_zero_attn

        self._reset_parameters()

    def forward(self, query, key, value, key_padding_mask=None,
                need_weights=True, attn_mask=None) -> Tuple[Tensor, Optional[Tensor]]:
        r"""
        Args:
            query, key, value: map a query and a set of key-value pairs to an output.
                See "Attention Is All You Need" for more details.
            key_padding_mask: if provided, specified padding elements in the key will
                be ignored by the attention. This is an binary mask. When the value is True,
                the corresponding value on the attention layer will be filled with -inf.
            need_weights: output attn_output_weights.
            attn_mask: 2D or 3D mask that prevents attention to certain positions. A 2D mask will be broadcasted for all
                the batches while a 3D mask allows to specify a different mask for the entries of each batch.

        Shape:
            Inputs:
            - query: :math:`(L, N, E)` where L is the target sequence length, N is the batch size, E is
              the embedding dimension.
            - key: :math:`(S, N, E)`, where S is the source sequence length, N is the batch size, E is
              the embedding dimension.
            - value: :math:`(S, N, E)` where S is the source sequence length, N is the batch size, E is
              the embedding dimension.
            - key_padding_mask: :math:`(N, S)` where N is the batch size, S is the source sequence length.
              If a ByteTensor is provided, the non-zero positions will be ignored while the zero positions
              will be unchanged. If a BoolTensor is provided, the positions with the
              value of ``True`` will be ignored while the position with the value of ``False`` will be unchanged.
            - attn_mask: 2D mask :math:`(L, S)` where L is the target sequence length, S is the source sequence length.
              3D mask :math:`(N*num_heads, L, S)` where N is the batch size, L is the target sequence length,
              S is the source sequence length. attn_mask ensures that position i is allowed to attend the unmasked
              positions. If a ByteTensor is provided, the non-zero positions are not allowed to attend
              while the zero positions will be unchanged. If a BoolTensor is provided, positions with ``True``
              are not allowed to attend while ``False`` values will be unchanged. If a FloatTensor
              is provided, it will be added to the attention weight.

            Outputs:
            - attn_output: :math:`(L, N, E)` where L is the target sequence length, N is the batch size,
              E is the embedding dimension.
            - attn_output_weights: :math:`(N, L, S)` where N is the batch size,
              L is the target sequence length, S is the source sequence length.
        """

        if not torch.jit.is_scripting():
            raise NotImplementedError()
            # tens_ops = (query, key, value, in_proj_weight, in_proj_bias, self.bias_k, bias_v,
            #             out_proj_weight, out_proj_bias)
            # if any([type(t) is not Tensor for t in tens_ops]) and has_torch_function(tens_ops):
            #     return handle_torch_function(
            #         multi_head_attention_forward, tens_ops, query, key, value,
            #         self.embed_dim, num_heads, in_proj_weight, in_proj_bias,
            #         self.bias_k, bias_v, self.add_zero_attn, self.dropout, out_proj_weight,
            #         out_proj_bias, training=self.training, key_padding_mask=key_padding_mask,
            #         need_weights=need_weights, attn_mask=attn_mask,
            #         use_separate_proj_weight=use_separate_proj_weight,
            #         q_proj_weight=q_proj_weight, k_proj_weight=k_proj_weight,
            #         v_proj_weight=v_proj_weight, static_k=None, static_v=None)
        tgt_len, bsz, embed_dim = query.size()
        assert embed_dim == self.embed_dim
        # allow MHA to have different sizes for the feature dimension
        assert key.size(0) == value.size(0) and key.size(1) == value.size(1)

        head_dim = embed_dim // self.num_heads
        assert head_dim * self.num_heads == embed_dim, "embed_dim must be divisible by num_heads"
        scaling = float(head_dim) ** -0.5

        q = self.q_proj(query)
        if key is None and torch.equal(key, value):
            # encoder-decoder attention
            k = None
            v = None
        else:
            # self-attention
            k = self.k_proj(key)
            v = self.v_proj(value)

        q = q * scaling

        if attn_mask is not None:
            assert attn_mask.dtype == torch.float32 or attn_mask.dtype == torch.float64 or \
                   attn_mask.dtype == torch.float16 or attn_mask.dtype == torch.uint8 or attn_mask.dtype == torch.bool, \
                'Only float, byte, and bool types are supported for attn_mask, not {}'.format(attn_mask.dtype)
            if attn_mask.dtype == torch.uint8:
                warnings.warn(
                    "Byte tensor for attn_mask in nn.MultiheadAttention is deprecated. Use bool tensor instead.")
                attn_mask = attn_mask.to(torch.bool)

            if attn_mask.dim() == 2:
                attn_mask = attn_mask.unsqueeze(0)
                if list(attn_mask.size()) != [1, query.size(0), key.size(0)]:
                    raise RuntimeError('The size of the 2D attn_mask is not correct.')
            elif attn_mask.dim() == 3:
                if list(attn_mask.size()) != [bsz * self.num_heads, query.size(0), key.size(0)]:
                    raise RuntimeError('The size of the 3D attn_mask is not correct.')
            else:
                raise RuntimeError("attn_mask's dimension {} is not supported".format(attn_mask.dim()))
            # attn_mask's dim is 3 now.

        # convert ByteTensor key_padding_mask to bool
        if key_padding_mask is not None and key_padding_mask.dtype == torch.uint8:
            warnings.warn(
                "Byte tensor for key_padding_mask in nn.MultiheadAttention is deprecated. Use bool tensor instead.")
            key_padding_mask = key_padding_mask.to(torch.bool)

        if self.bias_k is not None and self.bias_v is not None:
            k = self.bias_k(k)
            v = self.bias_v(v)
            if attn_mask is not None:
                attn_mask = F.pad(attn_mask, (0, 1))
            if key_padding_mask is not None:
                key_padding_mask = F.pad(key_padding_mask, (0, 1))
        else:
            assert self.bias_k is None
            assert self.bias_v is None

        q = q.contiguous().view(tgt_len, bsz * self.num_heads, head_dim).transpose(0, 1)
        if k is not None:
            k = k.contiguous().view(-1, bsz * self.num_heads, head_dim).transpose(0, 1)
        if v is not None:
            v = v.contiguous().view(-1, bsz * self.num_heads, head_dim).transpose(0, 1)

        src_len = k.size(1)

        if key_padding_mask is not None:
            assert key_padding_mask.size(0) == bsz
            assert key_padding_mask.size(1) == src_len

        if self.add_zero_attn:
            src_len += 1
            k = torch.cat([k, torch.zeros((k.size(0), 1) + k.size()[2:], dtype=k.dtype, device=k.device)], dim=1)
            v = torch.cat([v, torch.zeros((v.size(0), 1) + v.size()[2:], dtype=v.dtype, device=v.device)], dim=1)
            if attn_mask is not None:
                attn_mask = F.pad(attn_mask, (0, 1))
            if key_padding_mask is not None:
                key_padding_mask = F.pad(key_padding_mask, (0, 1))

        attn_output_weights = torch.bmm(q, k.transpose(1, 2))
        assert list(attn_output_weights.size()) == [bsz * self.num_heads, tgt_len, src_len]

        if attn_mask is not None:
            if attn_mask.dtype == torch.bool:
                attn_output_weights.masked_fill_(attn_mask, float('-inf'))
            else:
                attn_output_weights += attn_mask

        if key_padding_mask is not None:
            attn_output_weights = attn_output_weights.view(bsz, self.num_heads, tgt_len, src_len)
            attn_output_weights = attn_output_weights.masked_fill(
                key_padding_mask.unsqueeze(1).unsqueeze(2),
                float('-inf'),
            )
            attn_output_weights = attn_output_weights.view(bsz * self.num_heads, tgt_len, src_len)

        attn_output_weights = F.softmax(attn_output_weights, dim=-1)
        attn_output_weights = F.dropout(attn_output_weights, p=self.dropout, training=self.training)

        attn_output = torch.bmm(attn_output_weights, v)
        assert list(attn_output.size()) == [bsz * self.num_heads, tgt_len, head_dim]
        attn_output = attn_output.transpose(0, 1).contiguous().view(tgt_len, bsz, embed_dim)
        attn_output = self.out_proj(attn_output)

        if need_weights:
            # average attention weights over heads
            attn_output_weights = attn_output_weights.view(bsz, self.num_heads, tgt_len, src_len)
            return attn_output, attn_output_weights.sum(dim=1) / self.num_heads
        else:
            return attn_output, None
