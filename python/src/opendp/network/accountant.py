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

import torch
import torch.nn as nn
from torch.nn.parameter import Parameter

import opendp
from opendp.network.layers.bahdanau import DPBahdanauAttention
from opendp.network.layers.base import InstanceGrad
from opendp.network.layers.lstm import DPLSTM, DPLSTMCell

CHECK_CORRECTNESS = False

REPLACEMENT_MODULES = {
    nn.LSTM: DPLSTM,
    nn.LSTMCell: DPLSTMCell,
    'BahdanauAttention': DPBahdanauAttention
}

DATASET_SPACES = ('HammingDistance', 'SymmetricDistance')

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
            self, model: nn.Module, step_epsilon, step_delta=0.,
            dataset_distance: int = 1, dataset_space="HammingDistance",
            hook=True, disable=False, replacement_modules=None):
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
        if dataset_space not in DATASET_SPACES:
            raise ValueError(f"dataset_space must be one of {DATASET_SPACES}")
        self.dataset_space = dataset_space

        self._hooks_enabled = False  # work-around for https://github.com/pytorch/pytorch/issues/25723
        self.step_epsilon = step_epsilon
        self.step_delta = step_delta
        self._epochs = []
        self.steps = 0
        self._disable = disable

        self.odp = opendp.OpenDP()

        if not disable and hook:
            self.hook()

    def find_suitable_step_measure(self, reduction, clipping_norm, n):
        mechanism_name = 'gaussian' if self.step_delta else 'laplace'
        # find the tightest scale between 0. and 10k that satisfies the stepwise budget
        scale = binary_search(lambda s: self._check_noise_scale(reduction, clipping_norm, n, mechanism_name, s),
                                     0., 10_000.)
        return self._make_base_mechanism(mechanism_name, scale)

    def _check_noise_scale(self, reduction, clipping_norm, n, mechanism_name, scale):
        base_mechanism = self._make_base_mechanism(mechanism_name, scale)
        sensitivity_space = {'laplace': 'L1Sensitivity<f64>', 'gaussian': 'L2Sensitivity<f64>'}[mechanism_name]
        aggregator = {
            "mean": lambda: self.odp.trans.make_bounded_mean(
                f"<{self.dataset_space}, {sensitivity_space}, f64>".encode(),
                opendp.f64_p(-clipping_norm), opendp.f64_p(clipping_norm), n),
            "sum": lambda: self.odp.trans.make_bounded_sum(
                f"<{self.dataset_space}, {sensitivity_space}, f64>".encode(),
                opendp.f64_p(-clipping_norm), opendp.f64_p(clipping_norm))
        }[reduction]()
        chained = self.odp.core.make_chain_mt(base_mechanism, aggregator)

        budget = (self.step_epsilon, self.step_delta) if self.step_delta else self.step_epsilon
        check = self.odp.measurement_check(chained, self.dataset_distance, budget)
        self.odp.core.measurement_free(chained)
        return check

    def _make_base_mechanism(self, mechanism_name, scale):
        return getattr(self.odp.meas, f"make_base_{mechanism_name}")(b"<f64>", opendp.f64_p(scale))

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

        def capture_activations(module_: nn.Module, inputs: List[torch.Tensor], _output):
            """Save activations into module.activations in forward pass"""
            if not self._hooks_enabled:
                return
            if not hasattr(module_, 'activations'):
                module_.activations = []
            # always take the first argument of the input
            # NOTE: clone is required to prevent in-place-overwrite of stored activations
            module_.activations.append(inputs[0].detach().clone())

        def capture_backprops(module_: nn.Module, _inputs: List[torch.Tensor], outputs: List[torch.Tensor]):
            """Save backprops into module.backprops in backward pass"""
            if not self._hooks_enabled:
                return
            if not hasattr(module_, 'backprops'):
                module_.backprops = []
            # always take the first output's backprop
            # NOTE: clone is required to prevent in-place-overwrite of stored backprops
            module_.backprops.append(outputs[0].detach().clone())

        self.model.autograd_hooks = []
        for module in self.model.modules():
            if next(module.parameters(recurse=False), None) is not None:
                module_register_backward_hook = module.register_full_backward_hook \
                    if FULL_BACKWARD_HOOK else module.register_backward_hook

                self.model.autograd_hooks.extend([
                    module.register_forward_hook(capture_activations),
                    module_register_backward_hook(capture_backprops)
                ])

        return self

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
        return self.hook()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.unhook()

    def privatize_grad(self, *, modules=None, clipping_norm=1., reduction: str = 'mean') -> None:
        """
        Compute per-example gradients for each parameter, privatize them, and save them under 'param.grad'.
        Must be called after loss.backprop()
        Must be called before optimizer.step()

        :param modules:
        :param clipping_norm:
        :param reduction:  either "mean" or "sum" depending whether backpropped loss was averaged or summed over batch
        """
        assert reduction in ('sum', 'mean')

        if self._disable:
            return

        self.steps += 1

        modules = [m for m in modules or self.model.modules() if self._has_params(m)]

        # check for existence of activations and backprops
        for module in modules:
            assert hasattr(module, 'activations'), "No activations detected, run forward after .hook()"
            assert hasattr(module, 'backprops'), "No backprops detected, run backward after .hook()"

            if not module.backprops or not module.activations:
                return

        # retrieve batch size
        if modules:
            batch_size = modules[0].activations[0].shape[0]
        else:
            return

        # preprocess activations and backprops
        for module in modules:
            # ignore leading activations from evaluations outside of the training loop
            module.activations = module.activations[-len(module.backprops):]
            # backprops are in reverse-order
            module.backprops = module.backprops[::-1]

            if reduction == 'mean':
                for backprop in module.backprops:
                    backprop *= batch_size

        # reconstruct instance-level gradients and reduce privately
        for module in modules:
            # pair forward activations with reversed backpropagations
            for A, B in zip(module.activations, module.backprops):
                if isinstance(module, InstanceGrad):
                    module.update_instance_grad(A, B)

                elif isinstance(module, nn.Embedding):
                    A = A.unsqueeze(-1).expand(*A.shape, module.embedding_dim)
                    shape = batch_size, -1, module.embedding_dim

                    # massive... empty... tensor, because clip doesn't distribute
                    grad_instance = torch.zeros([batch_size, *module.weight.shape])
                    grad_instance.scatter_add_(1, A.reshape(*shape), B.reshape(*shape))

                    InstanceGrad._accumulate_instance_grad(module.weight, grad_instance)

                    # # reconstructs exact grad
                    # grad = torch.zeros_like(module.weight.grad)
                    # grad.index_add_(0, A.reshape(-1), B.reshape(-1, module.embedding_dim))
                    # self._accumulate_grad(module.weight, grad)

                elif isinstance(module, nn.Linear):
                    if len(A.shape) > 2:
                        for A, B in zip(torch.chunk(A, chunks=10, dim=1), torch.chunk(B, chunks=10, dim=1)):
                            grad_instance = torch.einsum('n...i,n...j->n...ij', B, A)
                            InstanceGrad._accumulate_instance_grad(module.weight, torch.einsum('n...ij->nij', grad_instance))
                            if module.bias is not None:
                                InstanceGrad._accumulate_instance_grad(module.bias, torch.einsum('n...i->ni', B))
                    else:
                        grad_instance = torch.einsum('n...i,n...j->n...ij', B, A)
                        InstanceGrad._accumulate_instance_grad(module.weight, torch.einsum('n...ij->nij', grad_instance))
                        if module.bias is not None:
                            InstanceGrad._accumulate_instance_grad(module.bias, torch.einsum('n...i->ni', B))

                else:
                    raise NotImplementedError(f"Gradient reconstruction is not implemented for {module}")

            if CHECK_CORRECTNESS:
                for param in module.parameters(recurse=False):
                    print('checking:', module, param.shape)
                    self._check_grad(param, reduction)

            del module.activations
            del module.backprops

        # compute L2 norm for flat clipping
        sum_squares = []
        for module in modules:
            for param in module.parameters(recurse=False):
                sum_squares.append(torch.sum(param.grad_instance.reshape(batch_size, -1) ** 2, dim=1))
        actual_norm = torch.sqrt(torch.stack(sum_squares, dim=1).sum(dim=1))

        # privatize gradients
        for module in modules:
            for param in module.parameters(recurse=False):
                param.grad = self._privatize_grad(param.grad_instance, reduction, actual_norm, clipping_norm)
                del param.grad_instance

    @staticmethod
    def _check_grad(param, reduction):
        grad = PrivacyAccountant._reduce_grad(param.grad_instance, reduction)
        if not torch.equal(torch.Tensor(list(param.grad.shape)), torch.Tensor(list(grad.shape))):
            raise ValueError(f"Non-private reconstructed gradient {grad.shape} differs from expected shape {param.grad.shape}")
        if not torch.allclose(param.grad, grad, atol=.01, equal_nan=True):
            print('          failed')
            print('          difference:')
            print(param.grad - grad)
            print('          expected:')
            print(param.grad)
            print('          reconstructed:')
            print(grad)
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

        measurement = self.find_suitable_step_measure(reduction, clipping_norm, n)
        grad.apply_(lambda x: self.odp.measurement_invoke(measurement, x))
        self.odp.core.measurement_free(measurement)

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
                step_budget = (self.step_epsilon, self.step_delta or 0.)
                batch_budget = (batch_epsilon, batch_delta)

                amplifier = self.odp.meas.make_shuffle_amplification(f"<{self.dataset_space}>".encode(), *step_budget, batch_len)
                check = self.odp.measurement_check(amplifier, self.dataset_distance, batch_budget)
                self.odp.core.measurement_free(amplifier)
                return check

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
