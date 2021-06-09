from typing import Callable, Dict, Iterable

import torch
import torch.nn as nn


class InstanceGrad(object):
    def get_instance_grad_functions(self) -> Dict[nn.Parameter, Callable[[torch.Tensor, torch.Tensor], Iterable[torch.Tensor]]]:
        """
        Returns a function for each parameter.
        Each function is provided activations and backprops,
            and returns an iterator that contains chunks of instance gradients
        """
        raise NotImplementedError("get_instance_grad_functions is not defined")
