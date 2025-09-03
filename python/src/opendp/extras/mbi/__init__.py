'''
This module requires extra installs: ``pip install 'opendp[mbi]'``

``mbi`` is short for "marginal-based inference", 
and is the name of the `Private-PGM <https://github.com/ryan112358/private-pgm>`_ package.
OpenDP uses Private-PGM to postprocess DP releases made with the OpenDP Library.

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.mbi``.
'''

from ._aim import AIM
from ._fixed import Fixed
from ._mst import MST
from ._sequential import Sequential
from ._table import make_contingency_table, ContingencyTable
from ._utilities import Algorithm, Count, mirror_descent

__all__ = [
    "AIM",
    "Fixed",
    "MST",
    "Sequential",
    "make_contingency_table",
    "ContingencyTable",
    "Algorithm",
    "Count",
    "mirror_descent",
]
