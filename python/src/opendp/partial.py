from opendp import Transformation, Measurement
from opendp.mod import binary_search
from functools import partial

import importlib

__all__ = ["enable_partial"]


class PartialChain(object):
    def __init__(self, f, *args, **kwargs):
        self.partial = partial(f, *args, **kwargs)
    
    def __call__(self, v):
        return self.partial(v)
    
    def fix(self, d_in, d_out):
        param = binary_search(lambda x: self.partial(x).check(d_in, d_out))
        chain = self.partial(param)
        chain.param = param
        return chain

    def __rshift__(self, other):
        if isinstance(other, (Transformation, Measurement)):
            return PartialChain(lambda x: self.partial(x) >> other)
        
        raise ValueError("other must be a Transformation or Measurement")


def wrap_constructor(f):
    def wrapped(*args, **kwargs):
        try:
            return f(*args, **kwargs)
        except TypeError:
            return PartialChain(f, *args, **kwargs)

    return wrapped


def enable_partial():
    for name in ["transformations", "measurements", "combinators"]:
        module = importlib.import_module(f"opendp.{name}")
        for f in dir(module):
            if f.startswith("make_"):
                module.__dict__[f] = wrap_constructor(getattr(module, f))

    # only run once
    enable_partial.__code__ = (lambda: None).__code__
