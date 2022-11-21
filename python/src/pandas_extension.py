from __future__ import annotations

import opendp.combinators as comb
import opendp.measurements as meas
import opendp.mod as mod
import opendp.transformations as trans

import pandas as pd



import operator
import re
from typing import Any, Sequence

import numpy as np
import pandas as pd


@pd.api.extensions.register_extension_dtype
class OpenDPDtype(pd.core.dtypes.dtypes.PandasExtensionDtype):

    @property
    def type(self):
        return np.generic

    @property
    def name(self) -> str:
        return "OpenDPType"

    @classmethod
    def construct_array_type(cls):
        return OpenDPArray


class OpenDPArray(pd.api.extensions.ExtensionArray):
    # * _from_sequence
    # * _from_factorized
    # * __getitem__
    # * __len__
    # * __eq__
    # * dtype
    # * nbytes
    # * isna
    # * take
    # * copy
    # * _concat_same_type

    # Include `copy` param for TestInterfaceTests
    def __init__(self, data):
        self._data = data
        self._make_chain = None

    @classmethod
    def _from_sequence(cls, data, dtype=None, copy: bool=False):
        if dtype is None:
            dtype = OpenDPDtype()
        if not isinstance(dtype, OpenDPDtype):
            raise ValueError(f"'{cls.__name__}' only supports 'OpenDPDtype' dtype")
        return cls(data)

    @classmethod
    def _from_factorized(cls, uniques: np.ndarray, original: "OpenDPArray"):
        raise Exception("Not implemented!")

    def __getitem__(self, index: int) -> "OpenDPArray" | Any:
        raise Exception("Not implemented!")

    def __len__(self) -> int:
        raise Exception("Not implemented!")

    @pd.core.ops.unpack_zerodim_and_defer('__eq__')
    def __eq__(self, other):
        raise Exception("Not implemented!")

    @property
    def dtype(self):
        return OpenDPDtype()

    @property
    def nbytes(self) -> int:
        raise Exception("Not implemented!")

    def isna(self):
        raise Exception("Not implemented!")

    def take(self, indices, allow_fill=False, fill_value=None):
        raise Exception("Not implemented!")

    def copy(self):
        raise Exception("Not implemented!")

    @classmethod
    def _concat_same_type(cls, to_concat: Sequence[OpenDPArray]) -> OpenDPArray:
        raise Exception("Not implemented!")

    def bounded_sum(self):
        if self._make_chain:
            self._make_chain = None
        else:
            self._make_chain = trans.make_bounded_sum()

    def query(self, d_in, d_out):
        # chain = self._
        return None

def test_extension():
    data = [1, 2, 3]
    array = OpenDPArray(data)
