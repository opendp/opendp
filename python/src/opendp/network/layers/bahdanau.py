import math

import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.nn.parameter import Parameter

from opendp.network.layers.base import InstanceGrad


class DPBahdanauAttentionScale(nn.Module, InstanceGrad):
    def __init__(self, embed_dim, normalize):
        super().__init__()
        self.v = Parameter(torch.Tensor(embed_dim))
        self.normalize = normalize
        if self.normalize:
            self.g = Parameter(torch.Tensor(1))

    def reset_parameters(self):
        nn.init.uniform_(self.v, -0.1, 0.1)
        if self.normalize:
            nn.init.constant_(self.g, math.sqrt(1.0 / self.embed_dim))

    def forward(self, x):
        if self.normalize:
            # learn the norm scale
            x = x * self.g / torch.norm(self.v)
        return x * self.v

    def update_instance_grad(self, activation, backprop):
        ba = backprop * activation
        v_grad_instance = torch.einsum('n...i->ni', ba)

        if self.normalize:
            g_grad_instance = torch.einsum('n...->n', ba * self.v) / torch.norm(self.v)
            self._accumulate_instance_grad(self.g, g_grad_instance.unsqueeze(-1))
            v_grad_instance *= self.g / torch.norm(self.v)

        self._accumulate_instance_grad(self.v, v_grad_instance)


class DPBahdanauAttention(nn.Module):
    """Bahdanau Attention"""

    def __init__(self, query_dim, value_dim, embed_dim, normalize=True):
        super().__init__()
        self.query_dim = query_dim
        self.value_dim = value_dim
        self.embed_dim = embed_dim
        self.query_proj = nn.Linear(self.query_dim, embed_dim, bias=normalize)
        self.value_proj = nn.Linear(self.value_dim, embed_dim, bias=False)
        self.scaler = DPBahdanauAttentionScale(embed_dim, normalize)

        self.reset_parameters()

    def reset_parameters(self):
        self.query_proj.weight.data.uniform_(-0.1, 0.1)
        self.value_proj.weight.data.uniform_(-0.1, 0.1)
        self.scaler.reset_parameters()

    def forward(self, query, value, key_padding_mask=None, state=None):
        # projected_query: 1 x bsz x embed_dim
        projected_query = self.query_proj(query).unsqueeze(0)
        key = self.value_proj(value)  # len x bsz x embed_dim
        attn_scores = self.scaler(torch.tanh(projected_query + key)).sum(dim=2)

        if key_padding_mask is not None:
            attn_scores = (
                attn_scores.float()
                    .masked_fill_(key_padding_mask, float("-inf"))
                    .type_as(attn_scores)
            )  # FP16 support: cast to float and back

        attn_scores = F.softmax(attn_scores, dim=0)  # srclen x bsz

        # sum weighted value. context: bsz x value_dim
        context = (attn_scores.unsqueeze(2) * value).sum(dim=0)
        next_state = attn_scores

        return context, attn_scores, next_state

    @staticmethod
    def replace(module):
        from espresso.modules.speech_attention import BahdanauAttention
        assert type(module) == BahdanauAttention

        replacement_module = DPBahdanauAttention(
            query_dim=module.query_dim,
            value_dim=module.value_dim,
            embed_dim=module.embed_dim,
            normalize=module.normalize)
        replacement_module.scaler.v = module.v
        if module.normalize:
            replacement_module.scaler.g = module.g

        return replacement_module
