"""
Tools for building differentially private models.
Thanks to https://github.com/cybertronai/autograd-hacks for demonstrating gradient hacks.

A, activations: inputs into current module
B, backprops: backprop values (aka Jacobian-vector product) observed at current module

"""
import math
from functools import lru_cache
from typing import List

import torch
from torch import nn

import opendp.meas as meas
import opendp.trans as trans
from opendp.mod import binary_search, enable_features
from opendp.typing import RuntimeType, DatasetMetric, SymmetricDistance, AllDomain, VectorDomain, SubstituteDistance
# from opendp._convert import set_return_mode
# set_return_mode('torch')
enable_features("contrib", "floating-point")


def assert_release_binary():
    import os
    # TODO: adjust this check once the library has an initial release
    assert os.environ.get('OPENDP_TEST_RELEASE', "false") != "false", \
        "32-bit floats from torch can only be privatized by release-mode OpenDP binaries.\n" \
        "The relevant cargo command is:\n" \
        "    cargo build --release --no-default-features\n" \
        "Then enable release binaries in the python bindings before you start the script:\n" \
        "    export OPENDP_TEST_RELEASE=1"


class BasePrivacyOdometer(object):
    def __init__(
            self,
            step_epsilon, step_delta=0.,
            clipping_norm=1.,
            reduction='mean',
            dataset_distance: int = 1,
            MI: DatasetMetric = SymmetricDistance):
        """Abstract base class for neural network privatizers.
        :param step_epsilon:
        :param step_delta:
        :param dataset_distance: group size
        :param MI: SubstituteDistance or SymmetricDistance
        """
        self.model = None
        self.dataset_distance = dataset_distance
        MI = RuntimeType.parse(MI)
        if not isinstance(MI, DatasetMetric):
            raise ValueError(f"MI must be a dataset metric")
        if MI == SubstituteDistance:
            # TODO: this should be pretty straightforward
            raise NotImplementedError("SubstituteDistance is not implemented")
        self.MI = MI

        self.step_epsilon = step_epsilon
        self.step_delta = step_delta

        # for budget tracking
        self._epochs = []
        self.steps = 0

        assert reduction in ('sum', 'mean')
        self.reduction = reduction
        self.clipping_norm = clipping_norm

    def track_(self, model):
        self.model = model

    @lru_cache
    def _find_suitable_step_measurement(self, proportion, reduction, clipping_norm, size):
        mechanism_name = 'gaussian' if self.step_delta else 'laplace'
        # find the tightest scale between 0. and 10k that satisfies the stepwise budget
        scale = binary_search(
            lambda s: self._check_noise_scale(
                proportion, reduction, clipping_norm, size, mechanism_name, s),
            bounds=(0., 10_000.),
            tolerance=1.0e-4)

        return self._make_base_mechanism(mechanism_name, scale, vectorize=True)

    def _check_noise_scale(self, prop, reduction, clipping_norm, size, mechanism_name, scale):

        if reduction == 'mean':
            constructor = trans.make_sized_bounded_mean
        elif reduction == 'sum':
            constructor = trans.make_sized_bounded_sum
        else:
            raise ValueError(f'unrecognized reduction: {reduction}. Must be "mean" or "sum"')

        aggregator = constructor(size, (-clipping_norm, clipping_norm), T="f32")
        chained = aggregator >> self._make_base_mechanism(mechanism_name, scale, vectorize=False)

        budget = (prop * self.step_epsilon, prop * self.step_delta) if self.step_delta else prop * self.step_epsilon
        return chained.check(self.dataset_distance, budget)

    @staticmethod
    def _make_base_mechanism(mechanism_name: str, scale: float, vectorize: bool):
        domain = AllDomain["f32"]
        if vectorize:
            domain = VectorDomain[domain]

        if mechanism_name == 'laplace':
            return meas.make_base_laplace(scale, D=domain)
        if mechanism_name == 'gaussian':
            return meas.make_base_gaussian(scale, D=domain)

    def clip_grad_(self, grad_instance, actual_norm):
        singletons = (1,) * (grad_instance.ndim - 1)
        grad_instance /= torch.max(torch.ones_like(actual_norm), actual_norm / self.clipping_norm) \
            .reshape(-1, *singletons) \
            .expand_as(grad_instance)

    def _noise_grad(self, grad, measurement):
        device = grad.device
        if device != 'cpu':
            grad = grad.to('cpu')

        grad = torch.tensor(measurement(grad.flatten().tolist())).reshape(grad.shape)

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

            epsilon += binary_search(check_epsilon, bounds=(0., 10_000.))
            delta += batch_delta

        return epsilon, delta
