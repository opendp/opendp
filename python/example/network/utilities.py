import json
import os
import sys
import time
import traceback
from multiprocessing import Queue, Event
from random import randint

import matplotlib.pyplot as plt
import torch
import torch.distributed as dist
from torch.multiprocessing import Process

# enable for more verbose logging
debug = True


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
        else:
            raise ValueError

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
        else:
            raise ValueError

        dist.send(tensor=self.step, dst=next_rank)
        for param in self.model.parameters():
            dist.send(tensor=param, dst=next_rank)


def printf(x, force=False):
    """
    overkill flushing
    :param x:
    :param force:
    :return:
    """
    if debug or force:
        print(x, flush=True)
        sys.stdout.flush()


def init_process(rank, size, fn, kwargs, backend='gloo'):
    """
    Initialize the distributed environment.
    """
    # use this command to kill processes:
    # lsof -t -i tcp:29500 | xargs kill
    os.environ['MASTER_ADDR'] = '127.0.0.1'
    os.environ['MASTER_PORT'] = '29500'
    dist.init_process_group(backend, rank=rank, world_size=size)

    try:
        fn(rank, size, **kwargs)
    except Exception:
        if not kwargs['end_event'].is_set():
            traceback.print_exc()
        kwargs['end_event'].set()


def main(worker, n_workers: int, **kwargs):
    """
    Example method demonstrating ring structure running on
    multiple processes. __main__ entrypoint.
    :return:
    """
    processes = []
    queue = Queue()

    end_event = Event()

    for rank in range(n_workers):
        p = Process(target=init_process, args=(rank, n_workers, worker, {
            'queue': queue, 'end_event': end_event,
            **kwargs
        }))
        p.start()
        processes.append(p)

    end_event.wait()
    # wait for history to be queued
    time.sleep(1)

    for p in processes:
        p.terminate()

    history = []
    usage = {}
    while not queue.empty():
        rank, (epsilon, delta), batch_history = queue.get()
        usage[str(rank)] = {'epsilon': epsilon, 'delta': delta}
        history.extend(batch_history)

    print(json.dumps(usage, indent=4))

    if history:
        import pandas
        history = pandas.DataFrame.from_records(list(sorted(history, key=lambda x: x['step'])))
        plt.plot(list(range(len(history))), history['loss'], color='b', alpha=0.5)
        plt.scatter(x=list(range(len(history))), y=history['loss'], c=history['rank'])
        plt.title("Log-Loss of Federated DPSGD")
        plt.xlabel("Step")
        plt.ylabel("Log-Loss")
        plt.legend(loc='upper right')
        plt.show()
