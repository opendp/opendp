'''
The ``prelude`` module provides shortcuts that reduce the number of ``import`` statements needed to get started.
In most of our notebooks we begin with:

.. code:: python

    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")

After that we can refer to members of 
:py:mod:`mod <opendp.mod>`,
:py:mod:`domains <opendp.domains>`,
:py:mod:`metrics <opendp.metrics>`,
:py:mod:`measures <opendp.measures>`,
:py:mod:`typing <opendp.typing>`,
:py:mod:`accuracy <opendp.accuracy>`, and
:py:mod:`context <opendp.context>`
using the shortcut.
Above, ``dp.enable_features`` is an example:
Its full path is :py:func:`opendp.mod.enable_features`.

In addition, three modules are distinctive and have shortcut submodules:
:py:mod:`transformations <opendp.transformations>` as ``t``,
:py:mod:`measurements <opendp.measurements>` as ``m``, and
:py:mod:`combinators <opendp.combinators>` as ``c``.
For example:

.. code:: python

    >>> type(dp.t.then_sum)
    <class 'function'>
    >>> type(dp.m.then_laplace)
    <class 'function'>
    >>> type(dp.c.make_basic_composition)
    <class 'function'>
'''
from opendp.mod import *
from opendp.extras import sklearn, numpy, polars, examples
from opendp.extras.polars import dp_len as len
import opendp.transformations as t
import opendp.measurements as m
import opendp.combinators as c
from opendp.domains import *
from opendp.metrics import *
from opendp.measures import *
from opendp.typing import *
from opendp.accuracy import *
from opendp.core import new_function, new_queryable
from opendp.context import *

__all__ = ["t", "m", "c"]
