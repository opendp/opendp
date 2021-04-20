import numbers
import warnings
from typing import Optional, Tuple

import torch
import torch.nn as nn
from torch.nn.modules.rnn import apply_permutation
from torch.nn.utils.rnn import PackedSequence
from torch.tensor import Tensor


class DPRNNBase(nn.Module):
    __constants__ = ['mode', 'input_size', 'hidden_size', 'num_layers', 'bias',
                     'batch_first', 'dropout', 'bidirectional']
    __jit_unused_properties__ = ['all_weights']

    mode: str
    input_size: int
    hidden_size: int
    num_layers: int
    bias: bool
    batch_first: bool
    dropout: float
    bidirectional: bool

    def __init__(self, mode: str, input_size: int, hidden_size: int,
                 num_layers: int = 1, bias: bool = True, batch_first: bool = False,
                 dropout: float = 0., bidirectional: bool = False) -> None:
        super().__init__()
        print('lstm base!')
        self.mode = mode
        self.input_size = input_size
        self.hidden_size = hidden_size
        self.num_layers = num_layers
        self.bias = bias
        self.batch_first = batch_first
        self.dropout = float(dropout)
        self.bidirectional = bidirectional

        if bidirectional:
            raise NotImplementedError("bidirectional lstm is not implemented.")

        if not isinstance(dropout, numbers.Number) or not 0 <= dropout <= 1 or \
                isinstance(dropout, bool):
            raise ValueError("dropout should be a number in range [0, 1] "
                             "representing the probability of an element being "
                             "zeroed")
        if dropout > 0 and num_layers == 1:
            warnings.warn("dropout option adds dropout after all but last "
                          "recurrent layer, so non-zero dropout expects "
                          "num_layers greater than 1, but got dropout={} and "
                          "num_layers={}".format(dropout, num_layers))

    def check_input(self, input: Tensor, batch_sizes: Optional[Tensor]) -> None:
        expected_input_dim = 2 if batch_sizes is not None else 3
        if input.dim() != expected_input_dim:
            raise RuntimeError(
                'input must have {} dimensions, got {}'.format(
                    expected_input_dim, input.dim()))
        if self.input_size != input.size(-1):
            raise RuntimeError(
                'input.size(-1) must be equal to input_size. Expected {}, got {}'.format(
                    self.input_size, input.size(-1)))

    def check_hidden_size(self, hx: Tensor, expected_hidden_size: Tuple[int, int, int],
                          msg: str = 'Expected hidden size {}, got {}') -> None:
        if hx.size() != expected_hidden_size:
            raise RuntimeError(msg.format(expected_hidden_size, list(hx.size())))

    def get_expected_hidden_size(self, input: Tensor, batch_sizes: Optional[Tensor]) -> Tuple[int, int, int]:
        if batch_sizes is not None:
            mini_batch = batch_sizes[0]
            mini_batch = int(mini_batch)
        else:
            mini_batch = input.size(0) if self.batch_first else input.size(1)
        num_directions = 2 if self.bidirectional else 1
        expected_hidden_size = (self.num_layers * num_directions,
                                mini_batch, self.hidden_size)
        return expected_hidden_size


class DPLSTMCell(nn.Module):
    def __init__(self, input_size: int, hidden_size: int, bias: bool = True):
        super().__init__()
        self.input_size = input_size
        self.hidden_size = hidden_size
        self.input_lin = nn.Linear(input_size, 4 * hidden_size, bias=bias)
        self.hidden_lin = nn.Linear(hidden_size, 4 * hidden_size, bias=bias)

    def forward(self, input: torch.Tensor, state: Tuple[torch.Tensor, torch.Tensor]):
        h_0, c_0 = state
        gates = self.input_lin(input) + self.hidden_lin(h_0)
        gate_in, gate_forget, gate_cell, gate_out = gates.chunk(4, dim=-1)

        gate_in = torch.sigmoid(gate_in)
        gate_forget = torch.sigmoid(gate_forget)
        gate_cell = torch.tanh(gate_cell)
        gate_out = torch.sigmoid(gate_out)

        c_1 = gate_forget * c_0 + gate_in * gate_cell
        h_1 = gate_out * torch.tanh(c_1)

        return h_1, c_1

    @staticmethod
    def replace(module: nn.LSTMCell):
        replacement_cell = DPLSTMCell(
            input_size=module.input_size,
            hidden_size=module.hidden_size,
            bias=module.bias)

        replacement_cell.input_lin.weight = getattr(module, f'weight_ih')
        replacement_cell.hidden_lin.weight = getattr(module, f'weight_hh')
        if module.bias:
            replacement_cell.input_lin.bias = getattr(module, f'bias_ih')
            replacement_cell.hidden_lin.bias = getattr(module, f'bias_hh')

        return replacement_cell


class DPLSTM(DPRNNBase):
    def __init__(self, *args, **kwargs):
        super().__init__('LSTM', *args, **kwargs)
        print('dplstm init!')

        self.layers = nn.ModuleList(
            [DPLSTMCell(self.input_size, self.hidden_size, bias=self.bias)] +
            [DPLSTMCell(self.hidden_size, self.hidden_size, bias=self.bias) for _ in range(self.num_layers - 1)])

    def forward(self, input, hx: Optional[Tuple[torch.Tensor, torch.Tensor]] = None):
        orig_input = input

        # xxx: isinstance check needs to be in conditional for TorchScript to compile
        if isinstance(orig_input, PackedSequence):
            input, batch_sizes, sorted_indices, unsorted_indices = input
            max_batch_size = batch_sizes[0]
            max_batch_size = int(max_batch_size)
        else:
            batch_sizes = None
            max_batch_size = input.size(0 if self.batch_first else 1)
            sorted_indices = None
            unsorted_indices = None

        if hx is None:
            num_directions = 2 if self.bidirectional else 1
            h = torch.zeros(self.num_layers * num_directions,
                            max_batch_size, self.hidden_size,
                            dtype=input.dtype, device=input.device)
            c = torch.zeros(self.num_layers * num_directions,
                            max_batch_size, self.hidden_size,
                            dtype=input.dtype, device=input.device)
        else:
            # Each batch of the hidden state should match the input sequence that
            # the user believes he/she is passing in.
            h, c = self.permute_hidden(hx, sorted_indices)

        self.check_forward_args(input, (h, c), batch_sizes)

        # unbind so that each layer gets separate backprop
        # h: hidden state, c: cell state
        # TODO: switch to chunk(x, self.num_layers) for bidirectional
        h, c = list(torch.unbind(h)), list(torch.unbind(c))

        output = []
        # for each step in sequence
        for input_t in torch.unbind(input, dim=1 if self.batch_first else 0):
            # pass forward through each layer
            for l in range(self.num_layers):
                h[l], c[l] = self.layers[l](input_t, (h[l], c[l]))
                input_t = h[l]
            output.append(input_t)

        output = torch.stack(output, dim=1 if self.batch_first else 0)

        hx = (torch.stack(h), torch.stack(c))

        # xxx: isinstance check needs to be in conditional for TorchScript to compile
        if isinstance(orig_input, PackedSequence):
            output_packed = PackedSequence(output, batch_sizes, sorted_indices, unsorted_indices)
            return output_packed, self.permute_hidden(hx, unsorted_indices)
        else:
            return output, self.permute_hidden(hx, unsorted_indices)

    def permute_hidden(self, hx: Tuple[Tensor, Tensor], permutation: Optional[Tensor]) -> Tuple[Tensor, Tensor]:
        if permutation is None:
            return hx
        return apply_permutation(hx[0], permutation), apply_permutation(hx[1], permutation)

    def check_forward_args(self, input: Tensor, hidden: Tuple[Tensor, Tensor], batch_sizes: Optional[Tensor]):
        self.check_input(input, batch_sizes)
        expected_hidden_size = self.get_expected_hidden_size(input, batch_sizes)

        self.check_hidden_size(hidden[0], expected_hidden_size,
                               'Expected hidden[0] size {}, got {}')
        self.check_hidden_size(hidden[1], expected_hidden_size,
                               'Expected hidden[1] size {}, got {}')

    @staticmethod
    def replace(module: nn.LSTM):
        replacement_module = DPLSTM(
            input_size=module.input_size,
            hidden_size=module.hidden_size,
            num_layers=module.num_layers,
            bias=module.bias,
            batch_first=module.batch_first,
            dropout=module.dropout,
            bidirectional=module.bidirectional)

        # reuse the parameter matrices from the original model
        for i, layer in enumerate(replacement_module.layers):
            layer.input_lin.weight = getattr(module, f'weight_ih_l{i}')
            layer.hidden_lin.weight = getattr(module, f'weight_hh_l{i}')
            if module.bias:
                layer.input_lin.bias = getattr(module, f'bias_ih_l{i}')
                layer.hidden_lin.bias = getattr(module, f'bias_hh_l{i}')

        return replacement_module
