from random import randint
import torch
import torch.distributed as dist


class ModelCoordinator(object):
    def __init__(self, model, rank, size, step_limit, federation_scheme='shuffle', end_event=None):

        assert federation_scheme in ('shuffle', 'ring')

        self.model = model
        self.rank = rank
        self.size = size
        self.federation_scheme = federation_scheme
        self._requests = {}

        self.step = torch.tensor(0)
        self.step_limit = step_limit
        self.end_event = end_event

    def recv(self):
        # Only block on receive when not in the initial step
        # otherwise this process will wait forever for another process to communicate with it
        if self.step == 0 and self.rank == 0:
            self.step += 1
            return

        if self.federation_scheme == 'shuffle':
            prev_rank = None
        elif self.federation_scheme == 'ring':
            prev_rank = (self.rank - 1) % self.size

        dist.recv(tensor=self.step, src=prev_rank)
        if self.step == self.step_limit:
            return

        for param in self.model.parameters():
            dist.recv(tensor=param, src=prev_rank)

        self.step += 1

        # kill all other processes
        if self.step == self.step_limit:
            for rank in range(self.size):
                if rank == self.rank:
                    continue
                dist.send(tensor=self.step, dst=rank)

            if self.end_event is not None:
                self.end_event.set()

    def send(self):
        if self.federation_scheme == 'shuffle':
            next_rank = self.rank
            while next_rank == self.rank:
                next_rank = randint(0, self.size - 1)
        elif self.federation_scheme == 'ring':
            next_rank = (self.rank + 1) % self.size

        dist.send(tensor=self.step, dst=next_rank)
        for param in self.model.parameters():
            dist.send(tensor=param, dst=next_rank)