"""
Tools for building differentially private models.
Thanks to https://github.com/cybertronai/autograd-hacks for demonstrating gradient hacks.

A, activations: inputs into current module
B, backprops: backprop values (aka Jacobian-vector product) observed at current module

"""
import copy
from functools import partial as functools_partial
from typing import List, Optional, Union, Dict, Callable, Tuple, Iterable, TypeVar

import numpy as np
import torch
import torch.nn as nn
from torch.nn.parameter import Parameter

from opendp.mod import enable_features

from opendp.network.odometer import BasePrivacyOdometer
from opendp.typing import DatasetMetric, SymmetricDistance

enable_features("contrib", "floating-point")


# If True, then computationally expensive tests are run to check that gradients are correct
CHECK_CORRECTNESS = False

# see https://github.com/pytorch/pytorch/issues/56380
FULL_BACKWARD_HOOK = False


# tuple of activations, one for each argument
Activations = Tuple[torch.Tensor]
# tuple of backprops, one for each output from the network. Implicitly upgraded to a singleton
Backprops = Tuple[torch.Tensor]
# an iterator over chunks of instance gradients of shape- n x (*param.shape)
InstanceGradIterator = Iterable[torch.Tensor]

# a function that:
# - takes in one pass of forward activations and backprops
# - returns an instance grad chunk iterable
InstanceGradConstructor = Callable[[Activations, Backprops], InstanceGradIterator]

# each module has an associated InstanceGradDict
InstanceGradDict = Dict[Parameter, InstanceGradConstructor]


class InstanceGrad(object):
    def get_instance_grad_functions(self) -> InstanceGradDict:
        """Returns a function for each parameter.
        Each function is provided activations and backprops,
            and returns an iterator that contains chunks of instance gradients
        """
        raise NotImplementedError("get_instance_grad_functions is not defined")


A = TypeVar('A')
B = TypeVar('B')
C = TypeVar('C')


# explicitly catch and show errors so they don't get swallowed in multiprocessing environments
def partial(func: Callable[..., C], *args, **kwargs) -> Callable[[A, B], C]:
    def func_warn(*args_, **kwargs_):
        # noinspection PyBroadException
        try:
            return func(*args_, **kwargs_)
        except:
            import traceback
            traceback.print_exc()

    # def func_debug(*args, **kwargs):
    #     print("invoking", func.__name__)
    #     result = func_warn(*args, **kwargs)
    #     print("done invoking", func.__name__)
    #     return result
    # result = functools_partial(func_debug, *args, **kwargs)

    result = functools_partial(func_warn, *args, **kwargs)
    # hack for pytorch 1.4
    result.__name__ = func.__name__
    return result


class ReconstructionPrivacyOdometer(BasePrivacyOdometer):
    def __init__(
            self,
            step_epsilon, step_delta=1e-6,
            clipping_norm=1.,
            reduction='mean',
            dataset_distance: int = 1,
            MI: DatasetMetric = SymmetricDistance):
        """
        Utility for making private neural networks and tracking privacy usage
        :param step_epsilon:
        :param step_delta:
        :param dataset_distance: group size
        :param MI: HammingDistance or SymmetricDistance
        """

        super().__init__(step_epsilon, step_delta, clipping_norm, reduction, dataset_distance, MI)
        self.is_tracking = False
        self.num_params = None

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
        # When a key is recognized in a model, replace it with the associated value
        from opendp.network.layers.bahdanau import DPBahdanauAttention
        from opendp.network.layers.lstm import DPLSTM, DPLSTMCell
        replacement_modules = {
            nn.LSTM: DPLSTM,
            nn.LSTMCell: DPLSTMCell,
            'BahdanauAttention': DPBahdanauAttention,
            **(replacement_modules or {})
        }

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
        self.num_params = sum(p.numel() for p in model.parameters() if p.requires_grad)

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
                # it is not necessary to compute gradients for gradient chunks; don't make a computation subgraph
                chunk = chunk.detach()
                # reshape [b, ...] into shape [b, -1], return sum of squares of shape [b]
                return torch.sum(chunk.reshape(chunk.size()[0], -1) ** 2, axis=1)

            def sum_pass(A, B, mapper):
                """sum the gradients associated with one forward/backward pass"""

                # When you evaluate get_instance_grad, you get a generator
                # The generator yields instance grad chunks of shape [b, ...]
                generator = get_instance_grad(A, B)

                # When mapper is `norm_sum`, this evaluates to a sum of squares reduction [b]
                # When mapper is `clip_chunk`, this evaluates to the clipped sum of each partial [b, ...]
                return sum(map(mapper, generator))

            def sum_all_passes(mapper):
                # Each weight can be referenced k times in a single forward pass
                # The backprops are of length k, and activations of length at least k
                #      ("at least" because scoring evaluations don't have accompanying backprops)
                # backpropagation happens in the opposite order from forward propagation, so we must reverse
                return sum(sum_pass(A, B, mapper) for A, B in zip(module_.activations, module_.backprops[::-1]))

            actual_norm = torch.sqrt(sum_all_passes(norm_sum))

            if CHECK_CORRECTNESS:
                print('checking:', module_, param_.shape)
                if self.reduction == 'mean':
                    for backprop in module_.backprops:
                        backprop *= self._get_batch_size(module_)
                self._check_grad(grad, param_.grad_instance, self.reduction)
            # actual_norm = norm(
            #     param_.grad_instance.reshape(param_.grad_instance.shape[0], -1),
            #     dim=1)

            def clip_chunk(chunk):
                """clip and then sum all leading axes of chunk, leaving a tensor of shape _param.shape"""
                chunk = chunk.detach()
                self.clip_grad_(chunk, actual_norm)
                return torch.sum(chunk.reshape(-1, *param_.shape), axis=0)

            clipped_grad = sum_all_passes(clip_chunk)

            measurement = self._find_suitable_step_measurement(
                param.numel() / self.num_params,
                self.reduction, self.clipping_norm, self._get_batch_size(module_))

            private_grad = self._noise_grad(clipped_grad, measurement)

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
                instance_grads: InstanceGradDict = module.get_instance_grad_functions()

            elif isinstance(module, nn.Embedding):
                def embedding_grad_generator(
                        module_, A: Activations, B: Tuple[torch.Tensor]
                ) -> Iterable[torch.Tensor]:
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
                            # grad_instance = torch.einsum('n...i,n...j->n...ij', B, A)
                            # yield torch.einsum('n...ij->nij', grad_instance)
                            yield torch.einsum('n...i,n...j->nij', B, A)
                    else:
                        # grad_instance = torch.einsum('n...i,n...j->n...ij', B, A)
                        # yield torch.einsum('n...ij->nij', grad_instance)
                        yield torch.einsum('n...i,n...j->n...ij', B, A)

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

            elif isinstance(module, nn.Conv2d):
                pass
            else:
                raise NotImplementedError(f"Gradient reconstruction is not implemented for {module}")

            # hooks on each of the params of the module
            for param in instance_grads:
                hook = param.register_hook(partial(hook_param_grad, param, module, instance_grads[param]))
                model.autograd_hooks.append(hook)

    @staticmethod
    def _get_batch_size(module_):
        # first activation, first arg, first axis shape
        # TODO: this could be flaky, especially around lstms
        return module_.activations[0][0].shape[0]

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
        raise NotImplementedError("this implementation is out-of-date")
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

    def _set_fill(self, constant):
        self._fill = constant
