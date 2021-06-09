"""
Tools for building differentially private models.
Thanks to https://github.com/cybertronai/autograd-hacks for demonstrating gradient hacks.

A, activations: inputs into current module
B, backprops: backprop values (aka Jacobian-vector product) observed at current module

"""
import copy
import math
from functools import wraps
from typing import List
import numpy as np

import torch
import torch.nn as nn
from opendp.typing import RuntimeType, DatasetMetric
from torch.nn.parameter import Parameter

import opendp.trans as trans
import opendp.meas as meas

from opendp.network.layers.bahdanau import DPBahdanauAttention
from opendp.network.layers.base import InstanceGrad
from opendp.network.layers.lstm import DPLSTM, DPLSTMCell

CHECK_CORRECTNESS = False

REPLACEMENT_MODULES = {
    nn.LSTM: DPLSTM,
    nn.LSTMCell: DPLSTMCell,
    'BahdanauAttention': DPBahdanauAttention
}

# see https://github.com/pytorch/pytorch/issues/56380
FULL_BACKWARD_HOOK = False


class _SharedParameter(Parameter):
    @classmethod
    def mark(cls, parameter):
        assert isinstance(parameter, Parameter)
        parameter.__class__ = cls

    def unmark(self):
        self.__class__ = Parameter

    def __copy__(self):
        return self

    def __deepcopy__(self, memodict={}):
        return self


class PrivacyAccountant(object):
    def __init__(
            self,
            model: nn.Module,
            step_epsilon, step_delta=0., clipping_norm=1.,
            dataset_distance: int = 1, dataset_space="HammingDistance",
            modules=None,
            hook=True,
            disable=False,
            reduction='mean',
            replacement_modules=None):
        """
        Utility for tracking privacy usage
        :param model: pyTorch model
        :param step_epsilon:
        :param step_delta:
        :param dataset_distance: group size
        :param dataset_space: HammingDistance or SymmetricDistance
        :param hook: whether to call hook() on __init__
        :param disable: turn off all privatization
        :param replacement_modules: dict of module classes to replace (for example nn.LSTM -> DPLSTM)
        """

        # copy network architecture, but share parameters
        for param in model.parameters():
            _SharedParameter.mark(param)
        self.model = copy.deepcopy(model)
        for param in model.parameters():
            param.unmark()

        replacement_modules = {**REPLACEMENT_MODULES, **(replacement_modules or {})}

        # restructure the copied network architecture in-place, without breaking references to original parameters
        self._replace_modules(self.model, replacement_modules)
        # overwrite forward on the first module to set requires_grad=True
        if FULL_BACKWARD_HOOK:
            self._patch_first_hook()

        self.dataset_distance = dataset_distance
        dataset_space = RuntimeType.parse(dataset_space)
        if not isinstance(dataset_space, DatasetMetric):
            raise ValueError(f"dataset_space must be a dataset metric")
        self.dataset_space = dataset_space

        self._hooks_enabled = False  # work-around for https://github.com/pytorch/pytorch/issues/25723
        self.step_epsilon = step_epsilon
        self.step_delta = step_delta
        self._epochs = []
        self.steps = 0
        self._disable = disable

        assert reduction in ('sum', 'mean')
        self.reduction = reduction

        self.modules = modules
        self.clipping_norm = clipping_norm

        if not disable and hook:
            self.hook()

    def find_suitable_step_measure(self, reduction, clipping_norm, size):
        mechanism_name = 'gaussian' if self.step_delta else 'laplace'
        # find the tightest scale between 0. and 10k that satisfies the stepwise budget
        scale = binary_search(lambda s: self._check_noise_scale(reduction, clipping_norm, size, mechanism_name, s),
                              0., 10_000.)
        return self._make_base_mechanism(mechanism_name, scale)

    def _check_noise_scale(self, reduction, clipping_norm, size, mechanism_name, scale):

        if reduction == 'mean':
            aggregator = trans.make_sized_bounded_mean(
                size, (-clipping_norm, clipping_norm))
        elif reduction == 'sum':
            aggregator = trans.make_sized_bounded_sum(
                size, (-clipping_norm, clipping_norm))
        else:
            raise ValueError(f'unrecognized reduction: {reduction}. Must be "mean" or "sum"')

        chained = aggregator >> self._make_base_mechanism(mechanism_name, scale)

        budget = (self.step_epsilon, self.step_delta) if self.step_delta else self.step_epsilon
        return chained.check(self.dataset_distance, budget)

    @staticmethod
    def _make_base_mechanism(mechanism_name, scale):
        if mechanism_name == 'laplace':
            return meas.make_base_laplace(scale)
        if mechanism_name == 'gaussian':
            return meas.make_base_gaussian(scale)

    @staticmethod
    def _replace_modules(module, replacement_modules):
        """
        replaces modules with DP-capable versions of modules throughout a network
        """

        for attr_str in dir(module):
            target_attr = getattr(module, attr_str)
            # ignore anything that isn't a module
            if not issubclass(type(target_attr), nn.Module):
                continue

            replacement_module = replacement_modules.get(type(target_attr))
            if not replacement_module:
                replacement_module = replacement_modules.get(target_attr.__class__.__name__)
            if replacement_module:
                replacement_attr = replacement_module.replace(target_attr)
                setattr(module, attr_str, replacement_attr)

        # recurse down child modules
        for name, child_module in module.named_children():
            PrivacyAccountant._replace_modules(child_module, replacement_modules)

    def _patch_first_hook(self):
        def set_requires_grad(arg):
            if torch.is_grad_enabled() and isinstance(arg, torch.Tensor) and arg.dtype.is_floating_point:
                arg.requires_grad = True
            return arg

        old_forward = self.model.forward

        @wraps(old_forward)
        def wrapper(*args, **kwargs):
            return old_forward(
                *(set_requires_grad(arg) for arg in args),
                **{kw: set_requires_grad(arg) for kw, arg in kwargs.items()})

        setattr(self.model, 'forward', wrapper)

    def hook(self):
        """
        Adds hooks to model to save activations and backprop values.

        The hooks will
        1. save activations into module.activations during forward pass
        2. save backprops into module.backprops during backward pass.

        Use unhook to disable this.
        """

        if self._hooks_enabled:
            # hooks have already been added
            return self

        self._hooks_enabled = True

        modules = [m for m in self.modules or self.model.modules() if self._has_params(m)]

        def capture_activations(module: nn.Module, input: List[torch.Tensor], _output):
            """Save activations into module.activations in forward pass"""
            if not self._hooks_enabled:
                return
            if not hasattr(module, 'activations'):
                module.activations = []
            # NOTE: clone is required to prevent in-place-overwrite of stored activations
            module.activations.append(tuple(in_arg.detach().clone() for in_arg in input))

        def capture_backprops(module: nn.Module, _input: List[torch.Tensor], output: List[torch.Tensor]):
            """Save backprops into module.backprops in backward pass"""
            if not self._hooks_enabled:
                return
            if not hasattr(module, 'backprops'):
                module.backprops = []
            # NOTE: clone is required to prevent in-place-overwrite of stored backprops
            module.backprops.append(tuple(out_arg.detach().clone() for out_arg in output))

        def get_batch_size(module):
            # first activation, first arg, first axis shape
            return module.activations[0][0].shape[0]

        def make_privatization_hook(module, param, instance_grad_generator):
            def privatization_hook(grad):

                # ignore leading activations from evaluations outside of the training loop
                module.activations = module.activations[-len(module.backprops):]

                if self.reduction == 'mean':
                    for backprop in module.backprops:
                        backprop *= get_batch_size(module)

                # backprops are in reverse-order
                for A, B in zip(module.activations, module.backprops[::-1]):
                    for chunk in instance_grad_generator(A, B):
                        InstanceGrad.accumulate_instance_grad(param, chunk)

                if CHECK_CORRECTNESS:
                    print('checking:', module, param.shape)
                    self._check_grad(grad, param.grad_instance, self.reduction)

                actual_norm = torch.norm(
                    param.grad_instance.reshape(param.grad_instance.shape[0], -1) ** 2,
                    dim=1)

                private_grad = self._privatize_grad(
                    param.grad_instance, self.reduction,
                    actual_norm, self.clipping_norm)

                del param.grad_instance
                param.is_grad_dp = True
                # clear module hook data once all param grads are dp
                if all(hasattr(par, 'is_grad_dp') and par.is_grad_dp for par in module.parameters(recurse=False)):
                    del module.activations
                    del module.backprops
                    for par in module.parameters(recurse=False):
                        del par.is_grad_dp

                return private_grad
            return privatization_hook

        self.model.autograd_hooks = []
        for module in modules:
            # ignore the module if it has no parameters
            if next(module.parameters(recurse=False), None) is None:
                continue

            # register global hooks
            self.model.autograd_hooks.extend([
                module.register_forward_hook(capture_activations),
                module.register_backward_hook(capture_backprops)
            ])

            if isinstance(module, InstanceGrad):
                # Dict[Parameter, Callable[[A, B], G]], where A is activations, B is backprops, G is instance grad
                # A: tuple of activations, one for each argument
                # B: tuple of backprops, one for each output from the network. Implicitly upgraded to a singleton
                # G: instance gradient of shape- n x (*param.shape)
                instance_grads = module.get_instance_grad_functions()

            elif isinstance(module, nn.Embedding):
                def make_embedding_grad_generator(module):
                    def embedding_grad_generator(A, B):
                        # only take the first argument to embedding forward
                        A, B = A[0], B[0]
                        batch_size = A.shape[0]
                        A = A.unsqueeze(-1).expand(*A.shape, module.embedding_dim)
                        shape = batch_size, -1, module.embedding_dim

                        # massive... empty... tensor, because clip doesn't distribute
                        grad_instance = torch.zeros([batch_size, *module.weight.shape])
                        grad_instance.scatter_add_(1, A.reshape(*shape), B.reshape(*shape))
                        yield grad_instance
                    return embedding_grad_generator

                instance_grads = {module.weight: make_embedding_grad_generator(module)}

                # # reconstructs exact grad
                # grad = torch.zeros_like(module.weight.grad)
                # grad.index_add_(0, A.reshape(-1), B.reshape(-1, module.embedding_dim))
                # self._accumulate_grad(module.weight, grad)

            elif isinstance(module, nn.Linear):
                def make_weight_grad_generator(_module):
                    def weight_grad_generator(A, B):
                        # linear is unary
                        A, B = A[0], B[0]

                        chunk_count = self._determine_chunk_count(A, B, chunk_size_limit=1000)
                        if len(A.shape) > 2 and chunk_count:
                            for A, B in zip(
                                    torch.chunk(A, chunks=chunk_count, dim=1),
                                    torch.chunk(B, chunks=chunk_count, dim=1)):
                                grad_instance = torch.einsum('n...i,n...j->n...ij', B, A)
                                yield torch.einsum('n...ij->nij', grad_instance)
                        else:
                            grad_instance = torch.einsum('n...i,n...j->n...ij', B, A)
                            yield torch.einsum('n...ij->nij', grad_instance)
                    return weight_grad_generator

                def make_bias_grad_generator(module):
                    def bias_grad_generator(A, B):
                        A, B = A[0], B[0]
                        if module.bias is None:
                            return

                        if len(A.shape) > 2:
                            for A, B in zip(torch.chunk(A, chunks=10, dim=1), torch.chunk(B, chunks=10, dim=1)):
                                yield torch.einsum('n...i->ni', B)
                        else:
                            yield torch.einsum('n...i->ni', B)
                    return bias_grad_generator

                instance_grads = {
                    module.weight: make_weight_grad_generator(module),
                    module.bias: make_bias_grad_generator(module),
                }

            else:
                raise NotImplementedError(f"Gradient reconstruction is not implemented for {module}")

            for param in instance_grads:
                privatization_hook = make_privatization_hook(module, param, instance_grads[param])
                self.model.autograd_hooks.append(param.register_hook(privatization_hook))

    @staticmethod
    def _determine_chunk_count(A, B, chunk_size_limit):
        if A.ndim <= 2 and B.ndim <= 2:
            return

        chunk_axis = 1
        A_shape = list(A.shape)
        if len(A_shape) >= chunk_axis:
            A_shape.pop(chunk_axis)
        B_shape = list(B.shape)
        if len(B_shape) >= chunk_axis:
            B_shape.pop(chunk_axis)

        other_axis_size = max(np.prod(A_shape), np.prod(B_shape))
        return int(other_axis_size / chunk_size_limit)

    def unhook(self):
        """
        Remove and deactivate hooks added by .hook()
        """

        # This issue indicates that hooks are not actually removed if the forward pass is run
        # https://github.com/pytorch/pytorch/issues/25723
        # Based on testing, the hooks are actually removed
        # Since hooks are removed, there is not an accumulation of hooks if the context manager is used within a loop

        if not hasattr(self.model, 'autograd_hooks'):
            print("Warning, asked to remove hooks, but no hooks found")
        else:
            for handle in self.model.autograd_hooks:
                handle.remove()
            del self.model.autograd_hooks

        # The _hooks_enabled flag is a secondary fallback if hooks aren't removed
        self._hooks_enabled = False

    def __enter__(self):
        self.hook()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.unhook()

    @staticmethod
    def _check_grad(grad, instance_grad, reduction):
        grad_2 = PrivacyAccountant._reduce_grad(instance_grad, reduction)
        if not torch.equal(torch.Tensor(list(grad.shape)), torch.Tensor(list(grad_2.shape))):
            raise ValueError(f"Non-private reconstructed gradient {grad_2.shape} differs from expected shape {grad.shape}")
        if not torch.allclose(grad, grad_2, atol=.01, equal_nan=True):
            print('          failed')
            print('          difference:')
            print(grad - grad_2)
            print('          expected:')
            print(grad)
            print('          reconstructed:')
            print(grad_2)
            raise ValueError("Non-private reconstructed gradient differs in value")

    def _privatize_grad(self, grad_instance, reduction, actual_norm, clipping_norm):
        """

        :param grad_instance:
        :param reduction:
        :param actual_norm:
        :param clipping_norm:
        :return:
        """

        # clip
        PrivacyAccountant._clip_grad_(grad_instance, actual_norm, clipping_norm)
        # reduce
        grad = PrivacyAccountant._reduce_grad(grad_instance, reduction)
        # noise
        grad = self._noise_grad(grad, clipping_norm, reduction, grad_instance.shape[0])

        return grad

    @staticmethod
    def _clip_grad_(grad_instance, actual_norm, clipping_norm):
        singletons = (1,) * (grad_instance.ndim - 1)
        grad_instance /= torch.max(torch.ones_like(actual_norm), actual_norm / clipping_norm) \
            .reshape(-1, *singletons) \
            .expand_as(grad_instance)

    @staticmethod
    def _reduce_grad(grad_instance, reduction):
        return {'sum': torch.sum, 'mean': torch.mean}[reduction](grad_instance, dim=0)

    def _noise_grad(self, grad, clipping_norm, reduction, n):
        device = grad.device
        if device != 'cpu':
            grad = grad.to('cpu')

        grad.apply_(self.find_suitable_step_measure(reduction, clipping_norm, n))

        if device != 'cpu':
            grad = grad.to(device)
        return grad

    @staticmethod
    def _has_params(module):
        return next(module.parameters(recurse=False), None) is not None

    def make_private_optimizer(self, optimizer, *args, **kwargs):

        class _PrivacyOptimizer(optimizer):
            """Extend *Optimizer with custom step function."""

            def __init__(_self, *_args, **_kwargs):
                _self.accountant = self
                super().__init__(*_args, **_kwargs)

            def step(_self, *_args, **_kwargs):
                r"""Performs a single optimization step (parameter update)."""
                with _self.accountant:
                    _self.accountant.privatize_grad(*_args, **_kwargs)
                return super().step(*_args, **_kwargs)

        return _PrivacyOptimizer(*args, **kwargs)

    def increment_epoch(self):
        if self.steps:
            self._epochs.append(self.steps)
        self.steps = 0

    def compute_usage(self, suggested_delta=None):
        """
        Compute epsilon/delta privacy usage for all tracked epochs
        :param suggested_delta: delta to
        :return:
        """
        self.increment_epoch()

        epsilon = 0
        delta = 0

        for batch_len in self._epochs:
            if suggested_delta is None:
                batch_delta = 2 * math.exp(-batch_len / 16 * math.exp(-self.step_epsilon)) + 1E-8
            else:
                batch_delta = suggested_delta / len(self._epochs)

            def check_epsilon(batch_epsilon):
                return meas.make_shuffle_amplification(
                    step_epsilon=self.step_epsilon,
                    step_delta=self.step_delta or 0.,
                    num_steps=batch_len
                ).check(self.dataset_distance, (batch_epsilon, batch_delta))

            epsilon += binary_search(check_epsilon, 0., 10_000.)
            delta += batch_delta

        return epsilon, delta


def binary_search(predicate, start, end):
    if start > end:
        raise ValueError

    if not predicate(end):
        raise ValueError("no possible value in range")

    while True:
        mid = (start + end) / 2
        passes = predicate(mid)

        if passes and end - start < .00001:
            return mid

        if passes:
            end = mid
        else:
            start = mid
