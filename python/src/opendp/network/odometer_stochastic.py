from typing import List, Optional

import torch
import torch.nn as nn

from opendp.network.odometer import BasePrivacyOdometer
from opendp.network.odometer_reconstruction import partial
from opendp.typing import DatasetMetric, SymmetricDistance
from opendp.mod import enable_features

enable_features("contrib", "floating-point")


class StochasticPrivacyOdometer(BasePrivacyOdometer):
    def __init__(
            self,
            step_epsilon, step_delta=1e-6,
            clipping_norm=1.,
            dataset_distance: int = 1,
            MI: DatasetMetric = SymmetricDistance):
        """
        Utility for tracking privacy usage
        :param step_epsilon:
        :param step_delta:
        :param dataset_distance: group size
        :param MI: HammingDistance or SymmetricDistance
        """

        super().__init__(step_epsilon, step_delta, clipping_norm, "sum", dataset_distance, MI)
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
            :param measurement: tool used to privatize the gradient
            :param grad: non-private gradient. Only useful for debugging grad correctness.
            :return: private grad
            """
            grad = grad.clone()
            grad /= torch.clamp(torch.norm(grad, p=2) / self.clipping_norm, min=1)
            return self._noise_grad(grad, measurement)

        model.autograd_hooks = []
        num_params = sum(p.numel() for p in model.parameters() if p.requires_grad)
        for module in self._filter_modules(model, whitelist):
            for param in module.parameters():
                if param.requires_grad:
                    measurement = self._find_suitable_step_measurement(
                        param.numel() / num_params, self.reduction, self.clipping_norm, 1)
                    model.autograd_hooks.append(param.register_hook(partial(hook_param_grad, measurement)))
