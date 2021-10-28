from opendp._convert import set_return_mode
from opendp._data import slice_as_object, object_as_slice, object_free, slice_free
from opendp.meas import make_base_laplace
from opendp.mod import enable_features
import numpy as np
import torch

enable_features("contrib", "floating-point")

DATA_LEN = 100_000
DATA = list(map(float, range(DATA_LEN)))
# DATA = 1.


def test_non_leak():
    # what should the memory usage look like?
    # hovers at around 80mb memory
    while True:
        list(map(float, range(DATA_LEN)))


def test_intentional_leak():
    # valgrind can't find this, nor anything else
    import ctypes
    while True:
        ctypes.c_char_p(b"abc")


def test_slice_as_object():
    # is there a leak when a python array is passed into the library?
    while True:
        obj = slice_as_object(DATA)
        object_free(obj)


def test_object_as_slice():
    # is there a leak when a data is returned from the library?
    obj = slice_as_object(DATA)
    while True:
        sl = object_as_slice(obj)
        slice_free(sl)


def test_list():
    meas = make_base_laplace(0., D="VectorDomain<AllDomain<f64>>")

    while True:
        meas.invoke(DATA)


def test_numpy():
    set_return_mode('numpy')
    meas = make_base_laplace(0., D="VectorDomain<AllDomain<f64>>")

    # convert to numpy
    data = np.array(DATA)

    while True:
        meas.invoke(data)


def test_torch():
    set_return_mode('torch')
    meas = make_base_laplace(0., D="VectorDomain<AllDomain<f64>>")

    # convert data to torch
    data = torch.from_numpy(np.array(DATA))
    # this approach segfaults
    # data = torch.tensor(DATA)
    while True:
        meas.invoke(data)


test_torch()