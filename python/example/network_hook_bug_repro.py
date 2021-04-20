import torch
import torch.nn as nn
import torch.nn.functional as F

from functools import wraps

# https://github.com/pytorch/pytorch/issues/56380


class TestModule(nn.Module):
    def __init__(self, input_size, output_size):
        super().__init__()
        internal_size = 5
        self.linear1 = nn.Linear(input_size, internal_size)
        self.linear2 = nn.Linear(internal_size, output_size)

    def forward(self, x: torch.Tensor):
        x = self.linear1(x)
        x = F.relu(x)
        x = self.linear2(x)
        return x


model = TestModule(4, 3)


def print_hook(module: nn.Module, _inputs, _outputs):
    print('hook triggered on', module)


for module in model.modules():
    if isinstance(module, nn.Linear):
        module.register_full_backward_hook(print_hook)
        print('added hook to', module)

def patch_first_hook(model):
    def set_requires_grad(arg):
        if isinstance(arg, torch.Tensor):
            arg.requires_grad = True
        return arg

    old_forward = model.forward

    @wraps(old_forward)
    def wrapper(*args, **kwargs):
        return old_forward(
            *(set_requires_grad(arg) for arg in args),
            **{kw: set_requires_grad(arg) for kw, arg in kwargs.items()})

    setattr(model, 'forward', wrapper)

from inspect import signature
print(signature(model.forward))

patch_first_hook(model)

x, y = torch.rand(size=(20, 4)), torch.randint(low=0, high=3, size=(20,))
loss = F.cross_entropy(model(x), y)

print('backward started')
loss.backward()
print('backward complete')
