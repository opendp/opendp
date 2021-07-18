import math
from typing import List, Optional

import torch
import torch.nn as nn
from opendp.v1.typing import RuntimeType, DatasetMetric, HammingDistance

import opendp.v1.trans as trans
import opendp.v1.meas as meas

from functools import partial as functools_partial


# hack for pytorch 1.4
def partial(func, *args, **keywords):
    result = functools_partial(func, *args, **keywords)
    result.__name__ = func.__name__
    return result


class StochasticPrivacyOdometer(object):
    def __init__(
            self,
            step_epsilon, step_delta=0.,
            clipping_norm=1.,
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
        self.clipping_norm = clipping_norm

        self.is_tracking = False

    def track_(
            self,
            model: nn.Module,
            whitelist: Optional[List[nn.Module]] = None
    ) -> None:
        """When .backward() is called on model, .grad on each parameter is differentially private.
        The related budget costs are tracked in this odometer class.

        Adds hooks to parameters to overwrite .grad.
        """
        if self.is_tracking:
            raise ValueError("An odometer can only track one model.")
        self.is_tracking = True

        def hook_param_grad(measurement, grad):
            """
            Privatization hook for parameters
            :param param_: the parameter being hooked
            :param module_: the module of the parameter being hooked
            :param get_instance_grad: function that returns a generator containing instance grad chunks
            :param grad: non-private gradient. Only useful for debugging grad correctness.
            :return: private grad
            """
            return self._noise_grad(measurement, self.clip_grad_(grad))

        model.autograd_hooks = []
        num_params = sum(p.numel() for p in model.parameters() if p.requires_grad)
        for module in self._filter_modules(model, whitelist):
            for param in module.parameters():
                if param.requires_grad:
                    measurement = self._find_suitable_step_measure(prop=param.numel() / num_params)
                    model.autograd_hooks.append(param.register_hook(partial(hook_param_grad, measurement)))

    def _find_suitable_step_measure(self, prop):
        mechanism_name = 'gaussian' if self.step_delta else 'laplace'
        # find the tightest scale between 0. and 10k that satisfies the stepwise budget
        scale = _binary_search(lambda s: self._check_noise_scale(prop, mechanism_name, s),
                               0., 10_000.)
        return self._make_base_mechanism_vec(mechanism_name, scale)

    def _check_noise_scale(self, prop, mechanism_name, scale):
        aggregator = trans.make_bounded_sum_n(
            lower=-self.clipping_norm, upper=self.clipping_norm, n=1)

        chained = aggregator >> self._make_base_mechanism(mechanism_name, scale)

        budget = (prop * self.step_epsilon, prop * self.step_delta) if self.step_delta else prop * self.step_epsilon
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

    def clip_grad_(self, grad_instance):
        grad_instance /= torch.max(1, torch.norm(grad_instance, p=2) / self.clipping_norm)

    @staticmethod
    def _noise_grad(measurement, grad):
        device = grad.device
        if device != 'cpu':
            grad = grad.to('cpu')

        grad = torch.FloatTensor(measurement(grad)).reshape(grad.shape)

        if device != 'cpu':
            grad = grad.to(device)
        return grad

    @staticmethod
    def _filter_modules(model, whitelist=None) -> List[nn.Module]:
        def has_params(module):
            return next(module.parameters(recurse=False), None) is not None
        return [m for m in whitelist or model.modules() if has_params(m)]

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
