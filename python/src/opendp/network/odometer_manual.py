import torch

from opendp.mod import enable_features
from opendp.network.odometer import BasePrivacyOdometer
from opendp.typing import DatasetMetric, SymmetricDistance
from opendp._convert import set_return_mode


set_return_mode('torch')
enable_features("contrib", "floating-point")


class ManualPrivacyOdometer(BasePrivacyOdometer):
    def __init__(
            self,
            model,
            step_epsilon, step_delta=0.,
            clipping_norm=1.,
            dataset_distance: int = 1,
            MI: DatasetMetric = SymmetricDistance):
        """
        Utility for tracking privacy usage
        :param model:
        :param step_epsilon:
        :param step_delta:
        :param dataset_distance: group size
        :param MI: SubstituteDistance or SymmetricDistance
        """

        super().__init__(model, step_epsilon, step_delta, clipping_norm, dataset_distance, MI)

    def privatize_grad(self, size):
        measurement = self._find_suitable_step_measurement(1., 'sum', self.clipping_norm, size)
        torch.nn.utils.clip_grad_norm_(self.model.parameters(), self.clipping_norm)
        for param in self.model.parameters():
            param.grad = self._noise_grad(param.grad, measurement)
