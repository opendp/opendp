import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.nn.parallel import DistributedDataParallel as DDP
import os
import torch.distributed as dist
import torch.multiprocessing as mp


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


def setup(rank, world_size):
    os.environ['MASTER_ADDR'] = '127.0.0.1'
    os.environ['MASTER_PORT'] = '12355'
    os.environ['GLOO_SOCKET_IFNAME'] = 'lo0'

    # initialize the process group
    dist.init_process_group("gloo", rank=rank, world_size=world_size)


def cleanup():
    dist.destroy_process_group()


def run_worker(rank, world_size):
    setup(rank, world_size)
    model = TestModule(4, 3)
    model = DDP(model)

    def print_hook(module: nn.Module, _inputs, _outputs):
        print('hook triggered on', module)

    for module in model.modules():
        if isinstance(module, nn.Linear):
            module.register_full_backward_hook(print_hook)
            print('added hook to', module)

    from inspect import signature
    print(signature(model.forward))

    for i in range(3):
        x, y = torch.rand(size=(20, 4)), torch.randint(low=0, high=3, size=(20,))
        loss = F.cross_entropy(model(x), y)

        print('backward started')
        loss.backward()
        model.zero_grad()
    print('backward complete')
    cleanup()


if __name__ == "__main__":
    for world_size in range(3, 6):
        mp.spawn(run_worker,
                 args=(world_size,),
                 nprocs=world_size,
                 join=True)
        print(f"DDP synchronized tensor grads successfully with {world_size} workers")
