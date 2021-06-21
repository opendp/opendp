
import os
import sys
import tempfile
import torch
import torch.distributed as dist
import torch.nn as nn
import torch.optim as optim
import torch.multiprocessing as mp

from torch.nn.parallel import DistributedDataParallel as DDP

# On Windows platform, the torch.distributed package only
# supports Gloo backend, FileStore and TcpStore.
# For FileStore, set init_method parameter in init_process_group
# to a local file. Example as follow:
# init_method="file:///f:/libtmp/some_file"
# dist.init_process_group(
#    "gloo",
#    rank=rank,
#    init_method=init_method,
#    world_size=world_size)
# For TcpStore, same way as on Linux.
from opendp.network.odometer import PrivacyOdometer


def setup(rank, world_size):
    os.environ['MASTER_ADDR'] = '127.0.0.1'
    os.environ['MASTER_PORT'] = '12355'
    os.environ['GLOO_SOCKET_IFNAME'] = 'lo0'

    # initialize the process group
    dist.init_process_group("gloo", rank=rank, world_size=world_size)


def cleanup():
    dist.destroy_process_group()


class ToyModel(nn.Module):
    def __init__(self):
        super(ToyModel, self).__init__()
        self.net1 = nn.Linear(10, 10)
        self.relu = nn.ReLU()
        self.net2 = nn.Linear(10, 5)

    def forward(self, x):
        return self.net2(self.relu(self.net1(x)))


def demo_basic(rank, world_size):
    print(f"Running basic DDP example on rank {rank}.")
    setup(rank, world_size)

    torch.manual_seed(0)
    # create model and move it to GPU with id rank
    model = ToyModel()  # .to(rank)
    model = DDP(model)  # , device_ids=[rank]

    odometer = PrivacyOdometer(step_epsilon=.1)
    # fill grads with `rank`
    odometer._set_fill(constant=rank)
    model = odometer.make_tracked_view(model)

    loss_fn = nn.MSELoss()
    optimizer = optim.SGD(model.parameters(), lr=0.001)

    optimizer.zero_grad()
    outputs = model(torch.randn(20, 10))
    labels = torch.randn(20, 5)  # .to(rank)
    loss_fn(outputs, labels).backward()
    optimizer.step()

    mean = sum(range(world_size)) / world_size
    assert all(v == mean for v in model.module.net1.bias.grad.tolist())

    cleanup()


if __name__ == "__main__":
    for world_size in range(3, 6):
        mp.spawn(demo_basic,
                 args=(world_size,),
                 nprocs=world_size,
                 join=True)
        print(f"DDP synchronized tensor grads successfully with {world_size} workers")
