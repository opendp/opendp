from torch.tensor import Tensor


class InstanceGrad(object):
    def update_instance_grad(self, activation: Tensor, backprop: Tensor):
        raise NotImplementedError

    @staticmethod
    def accumulate_instance_grad(tensor, grad):
        if hasattr(tensor, 'grad_instance'):
            tensor.grad_instance += grad.detach()
        else:
            tensor.grad_instance = grad.detach()
