"""
Tools for building differentially private models.
Thanks to https://github.com/cybertronai/autograd-hacks for demonstrating gradient hacks.

A, activations: inputs into current module
B, backprops: backprop values (aka Jacobian-vector product) observed at current module

"""
import copy
import math
from functools import wraps
from typing import List, Optional, Union, Dict
import numpy as np

import torch
import torch.nn as nn
from opendp.v1.typing import L1Distance, L2Distance, RuntimeType, DatasetMetric, HammingDistance
from torch.nn.parameter import Parameter

import opendp.v1.trans as trans
import opendp.v1.meas as meas

from opendp.network.layers.bahdanau import DPBahdanauAttention
from opendp.network.layers.base import InstanceGrad
from opendp.network.layers.lstm import DPLSTM, DPLSTMCell

from functools import partial

CHECK_CORRECTNESS = False

REPLACEMENT_MODULES = {
    nn.LSTM: DPLSTM,
    nn.LSTMCell: DPLSTMCell,
    'BahdanauAttention': DPBahdanauAttention
}

# see https://github.com/pytorch/pytorch/issues/56380
FULL_BACKWARD_HOOK = False


class PrivacyOdometer(object):
    def __init__(
            self,
            step_epsilon, step_delta=0.,
            clipping_norm=1.,
            reduction='mean',
            dataset_distance: int = 1,
            MI: DatasetMetric = HammingDistance):
        """
        Utility for tracking privacy usage
        :param step_epsilon:
        :param step_delta:
        :param dataset_distance: group size
        :param MI: HammingDistance or SymmetricDistance
        """

        self.dataset_distance = dataset_distance
        MI = RuntimeType.parse(MI)
        if not isinstance(MI, DatasetMetric):
            raise ValueError(f"MI must be a dataset metric")
        self.MI = MI

        self.step_epsilon = step_epsilon
        self.step_delta = step_delta

        # for budget tracking
        self._epochs = []
        self.steps = 0

        assert reduction in ('sum', 'mean')
        self.reduction = reduction
        self.clipping_norm = clipping_norm

        self.is_tracking = False

    def make_tracked_view(
            self,
            model: nn.Module,
            replacement_modules: Dict[Union[nn.Module, str], nn.Module] = None,
            whitelist: Optional[List[nn.Module]] = None
    ) -> nn.Module:
        """Returns an equivalent model that shares the input model's parameters.
        When .backward() is called on the returned model, .grad on parameters is differentially private.
        The related budget costs are tracked in this odometer class.
        The original model can still be used to train the same parameters with public data.

        :param model:
        :param replacement_modules: dict of custom module replacement classes
        :param whitelist: if defined, only hook these modules
        :return: a copy of the model that shares parameters and has DP backprop
        """

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

        # copy network architecture, but share parameters
        for param in model.parameters():
            # temporarily mutates the original model to prevent parameters from being copied
            _SharedParameter.mark(param)
        model = copy.deepcopy(model)
        # param is shared in both the original and copied model, so the original model is restored
        for param in model.parameters():
            param.unmark()

        self.track_(model, replacement_modules, whitelist)

        return model

    def track_(
            self,
            model: nn.Module,
            replacement_modules: Dict[Union[nn.Module, str], nn.Module] = None,
            whitelist: Optional[List[nn.Module]] = None
    ) -> None:
        """When .backward() is called on model, .grad on each parameter is differentially private.
        The related budget costs are tracked in this odometer class.

        Adds hooks throughout the module tree to save activations and backprop values.
        Adds hooks to parameters to overwrite .grad.
        May replace modules in the module tree with equivalent modules that have DP capabilities.

        The module hooks will
        1. save activations into module.activations during forward pass
        2. save backprops into module.backprops during backward pass.
        """
        if self.is_tracking:
            raise ValueError("An odometer can only track one model.")
        self.is_tracking = True

        # 1. MODULE REWRITING
        replacement_modules = {**REPLACEMENT_MODULES, **(replacement_modules or {})}

        if model in replacement_modules:
            raise ValueError("Root module cannot be replaced. Try wrapping the model in a module.")

        def replace_modules(module_):
            """
            replaces modules with DP-capable versions of modules throughout a network
            """

            for attr_str in dir(module_):
                target_attr = getattr(module_, attr_str)
                # ignore anything that isn't a module
                if not issubclass(type(target_attr), nn.Module):
                    continue

                # only replace modules in the module whitelist
                if whitelist is not None and id(target_attr) not in whitelist:
                    continue

                replacement_module = replacement_modules.get(type(target_attr))
                if not replacement_module:
                    replacement_module = replacement_modules.get(target_attr.__class__.__name__)
                if replacement_module:
                    replacement_attr = replacement_module.replace(target_attr)
                    setattr(module_, attr_str, replacement_attr)

            # recurse down child modules
            for name, child_module in module_.named_children():
                replace_modules(child_module)

        # restructure the copied network architecture in-place, without breaking references to original parameters
        replace_modules(model)

        # overwrite forward on the first module to set requires_grad=True
        if FULL_BACKWARD_HOOK:
            self._patch_first_hook(model)

        # 2. DEFINE HOOKS
        def hook_module_forward(module_: nn.Module, input_: List[torch.Tensor], _output):
            """Save activations into module.activations in forward pass"""
            if not hasattr(module_, 'activations'):
                module_.activations = []
            # NOTE: clone is required to prevent in-place-overwrite of stored activations
            module_.activations.append(tuple(in_arg.detach().clone() for in_arg in input_))

        def hook_module_backward(module_: nn.Module, _input: List[torch.Tensor], output: List[torch.Tensor]):
            """Save backprops into module.backprops in backward pass"""
            if not hasattr(module_, 'backprops'):
                module_.backprops = []
            # NOTE: clone is required to prevent in-place-overwrite of stored backprops
            module_.backprops.append(tuple(out_arg.detach().clone() for out_arg in output))

        def hook_param_grad(param_, module_, get_instance_grad, grad):
            """
            Privatization hook for parameters
            :param param_: the parameter being hooked
            :param module_: the module of the parameter being hooked
            :param get_instance_grad: function that returns a generator containing instance grad chunks
            :param grad: non-private gradient. Only useful for debugging grad correctness.
            :return: private grad
            """

            # ignore leading activations from evaluations outside of the training loop
            module_.activations = module_.activations[-len(module_.backprops):]

            def norm_sum(chunk):
                chunk = chunk.detach()
                return torch.sum(chunk.reshape(chunk.size()[0], -1) ** 2, axis=1)

            def sum_pass(A, B, mapper):
                """sum the gradients associated with one forward/backward pass"""
                return sum(map(mapper, get_instance_grad(A, B)))

            def sum_all_passes(mapper):
                # backprops are in reverse-order
                return sum(sum_pass(A, B, mapper) for A, B in zip(module_.activations, module_.backprops[::-1]))

            actual_norm = torch.sqrt(sum_all_passes(norm_sum))

            # if CHECK_CORRECTNESS:
            #     print('checking:', module_, param_.shape)
            #     if self.reduction == 'mean':
            #         for backprop in module_.backprops:
            #             backprop *= self._get_batch_size(module_)
            #     self._check_grad(grad, param_.grad_instance, self.reduction)
            # actual_norm = norm(
            #     param_.grad_instance.reshape(param_.grad_instance.shape[0], -1),
            #     dim=1)

            def clip_chunk(chunk):
                """clip and then sum all leading axes of chunk, leaving a tensor of shape _param.shape"""
                chunk = chunk.detach()
                self.clip_grad_(chunk, actual_norm)
                return torch.sum(chunk.reshape(-1, *param_.shape), axis=0)

            clipped_grad = sum_all_passes(clip_chunk)

            private_grad = self._noise_grad(
                clipped_grad, self.clipping_norm, self.reduction, self._get_batch_size(module_))

            param_.is_grad_dp = True
            # clear module hook data once all param grads are dp
            if all(hasattr(par, 'is_grad_dp') and par.is_grad_dp for par in module_.parameters(recurse=False)):
                del module_.activations
                del module_.backprops
                for par in module_.parameters(recurse=False):
                    del par.is_grad_dp

            return private_grad

        # 3. SET HOOKS
        model.autograd_hooks = []
        for module in self._filter_modules(model, whitelist):

            module_register_backward_hook = module.register_full_backward_hook \
                if FULL_BACKWARD_HOOK else module.register_backward_hook

            # hooks on the module
            model.autograd_hooks.extend([
                module.register_forward_hook(hook_module_forward),
                module_register_backward_hook(hook_module_backward)
            ])

            # hooks on each of the params. First need to define how to build instance grads for each param
            if isinstance(module, InstanceGrad):
                # Dict[Parameter, Callable[[A, B], G]], where A is activations, B is backprops, G is instance grad
                # A: tuple of activations, one for each argument
                # B: tuple of backprops, one for each output from the network. Implicitly upgraded to a singleton
                # G: instance gradient of shape- n x (*param.shape)
                instance_grads = module.get_instance_grad_functions()

            elif isinstance(module, nn.Embedding):
                def embedding_grad_generator(module_, A, B):
                    # only take the first argument to embedding forward
                    A, B = A[0], B[0]
                    batch_size = A.shape[0]
                    A = A.unsqueeze(-1).expand(*A.shape, module_.embedding_dim)
                    shape = batch_size, -1, module_.embedding_dim

                    # massive... empty... tensor, because clip doesn't distribute
                    grad_instance = torch.zeros([batch_size, *module_.weight.shape])
                    grad_instance.scatter_add_(1, A.reshape(*shape), B.reshape(*shape))
                    yield grad_instance

                instance_grads = {module.weight: partial(embedding_grad_generator, module)}

                # # reconstructs exact grad
                # grad = torch.zeros_like(module.weight.grad)
                # grad.index_add_(0, A.reshape(-1), B.reshape(-1, module.embedding_dim))
                # self._accumulate_grad(module.weight, grad)

            elif isinstance(module, nn.Linear):
                def weight_grad_generator(A, B):
                    # linear is unary; there are only activations/backprops for one argument
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

                def bias_grad_generator(module_, A, B):
                    if module_.bias is None:
                        return
                    A, B = A[0], B[0]

                    if len(A.shape) > 2:
                        for A, B in zip(torch.chunk(A, chunks=10, dim=1), torch.chunk(B, chunks=10, dim=1)):
                            yield torch.einsum('n...i->ni', B)
                    else:
                        yield torch.einsum('n...i->ni', B)

                instance_grads = {
                    module.weight: weight_grad_generator,
                    module.bias: partial(bias_grad_generator, module),
                }

            else:
                raise NotImplementedError(f"Gradient reconstruction is not implemented for {module}")

            # hooks on each of the params of the module
            for param in instance_grads:
                hook = param.register_hook(partial(hook_param_grad, param, module, instance_grads[param]))
                model.autograd_hooks.append(hook)

    def _find_suitable_step_measure(self, reduction, clipping_norm, n):
        mechanism_name = 'gaussian' if self.step_delta else 'laplace'
        # find the tightest scale between 0. and 10k that satisfies the stepwise budget
        scale = _binary_search(lambda s: self._check_noise_scale(reduction, clipping_norm, n, mechanism_name, s),
                               0., 10_000.)
        return self._make_base_mechanism_vec(mechanism_name, scale)

    def _check_noise_scale(self, reduction, clipping_norm, n, mechanism_name, scale):
        if reduction == 'mean':
            aggregator = trans.make_bounded_mean(
                -clipping_norm, clipping_norm, n, MI=self.MI)
        elif reduction == 'sum':
            aggregator = trans.make_bounded_sum(
                -clipping_norm, clipping_norm, MI=self.MI)
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
    def _make_base_mechanism_vec(mechanism_name, scale):
        if mechanism_name == 'laplace':
            return meas.make_base_vector_laplace(scale)
        if mechanism_name == 'gaussian':
            return meas.make_base_vector_gaussian(scale)

    @staticmethod
    def _get_batch_size(module_):
        # first activation, first arg, first axis shape
        # TODO: this could be flaky, especially around lstms
        return module_.activations[0][0].shape[0]

    @staticmethod
    def _patch_first_hook(model):
        def set_requires_grad(arg):
            if torch.is_grad_enabled() and isinstance(arg, torch.Tensor) and arg.dtype.is_floating_point:
                arg.requires_grad = True
            return arg

        old_forward = model.forward

        @wraps(old_forward)
        def wrapper(*args, **kwargs):
            return old_forward(
                *(set_requires_grad(arg) for arg in args),
                **{kw: set_requires_grad(arg) for kw, arg in kwargs.items()})

        setattr(model, 'forward', wrapper)

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

    @staticmethod
    def _unhook(model):
        """
        Remove hooks added to `model`. Does not reverse module replacement.
        """

        # This issue indicates that hooks are not actually removed if the forward pass is run
        # https://github.com/pytorch/pytorch/issues/25723
        # Based on testing, the hooks are actually removed

        if not hasattr(model, 'autograd_hooks'):
            print("Warning, asked to remove hooks, but no hooks found")
        else:
            for handle in model.autograd_hooks:
                handle.remove()
            del model.autograd_hooks

    @staticmethod
    def _check_grad(grad, instance_grad, reduction):
        grad_2 = PrivacyOdometer._reduce_grad(instance_grad, reduction)
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

    def clip_grad_(self, grad_instance, actual_norm):
        singletons = (1,) * (grad_instance.ndim - 1)
        grad_instance /= torch.max(torch.ones_like(actual_norm), actual_norm / self.clipping_norm) \
            .reshape(-1, *singletons) \
            .expand_as(grad_instance)

    def _noise_grad(self, grad, clipping_norm, reduction, n):
        device = grad.device
        if device != 'cpu':
            grad = grad.to('cpu')

        measurement = self._find_suitable_step_measure(reduction, clipping_norm, n)
        grad = torch.FloatTensor(measurement(grad)).reshape(grad.shape)

        # fill gradient with a constant, if a _fill value is set. Useful for validating DDP
        if hasattr(self, '_fill') and self._fill is not None:
            grad.fill_(self._fill)

        if device != 'cpu':
            grad = grad.to(device)
        return grad

    @staticmethod
    def _filter_modules(model, whitelist=None) -> List[nn.Module]:
        def has_params(module):
            return next(module.parameters(recurse=False), None) is not None
        return [m for m in whitelist or model.modules() if has_params(m)]

    def _set_fill(self, constant):
        self._fill = constant

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
                    num_steps=batch_len,
                    MI=self.MI
                ).check(self.dataset_distance, (batch_epsilon, batch_delta))

            epsilon += _binary_search(check_epsilon, 0., 10_000.)
            delta += batch_delta

        return epsilon, delta


def _binary_search(predicate, start, end):
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
