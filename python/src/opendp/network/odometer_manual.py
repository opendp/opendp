import math
from typing import List, Optional

import torch
import torch.nn as nn
from opendp.v1.typing import RuntimeType, DatasetMetric, HammingDistance
from opendp.v1._convert import set_return_mode

import opendp.v1.trans as trans
import opendp.v1.meas as meas

set_return_mode('torch')


class ManualPrivacyOdometer(object):
    def __init__(
            self,
            model,
            step_epsilon, step_delta=0.,
            clipping_norm=1.,
            dataset_distance: int = 1,
            MI: DatasetMetric = HammingDistance):
        """
        Utility for tracking privacy usage
        :param model:
        :param step_epsilon:
        :param step_delta:
        :param dataset_distance: group size
        :param MI: HammingDistance or SymmetricDistance
        """

        self.model = model
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
        self.measurement = self._find_suitable_measurement()

    def privatize_grad(self):
        torch.nn.utils.clip_grad_norm_(self.model.parameters(), self.clipping_norm)
        for param in self.model.parameters():
            param.grad = self.noise_grads(param.grad)

    def _find_suitable_measurement(self):
        mechanism_name = 'gaussian' if self.step_delta else 'laplace'
        # find the tightest scale between 0. and 10k that satisfies the stepwise budget
        scale = _binary_search(lambda s: self._check_noise_scale(mechanism_name, s),
                               0., 10_000.)
        return self._make_base_mechanism_vec(mechanism_name, scale)

    def _check_noise_scale(self, mechanism_name, scale):
        aggregator = trans.make_bounded_sum_n(
            lower=-self.clipping_norm, upper=self.clipping_norm, n=1)

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
            return meas.make_base_vector_laplace(scale, T='f32')
        if mechanism_name == 'gaussian':
            return meas.make_base_vector_gaussian(scale, T='f32')

    def noise_grads(self, grad):
        device = grad.device
        if device != 'cpu':
            grad = grad.to('cpu')

        grad = self.measurement(grad).reshape(grad.shape)

        if device != 'cpu':
            grad = grad.to(device)
        return grad

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
