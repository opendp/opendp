"""
This module requires extra installs: ``pip install 'opendp[polars]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.polars.contingency_table``.
"""

from typing import Optional, Type, Union
from opendp.context import Query
from opendp.extras.mbi._table import ContingencyTable


class ContingencyTableQuery(Query):
    """Extends the Query class to include special attributes for contingency tables.

    :param scale: noise scale to be added to one-way marginals
    :param threshold: cutoff for small groups to be merged into a null group
    :param kwargs: arguments for parent class
    """

    def __init__(
        self,
        *,
        oneway_scale: Optional[float],
        oneway_threshold: Optional[int],
        **kwargs
    ):
        super().__init__(**kwargs)
        self.oneway_scale = oneway_scale
        self.oneway_threshold = oneway_threshold

    def release(
        self,
        data=None,
        bounds: Optional[tuple[float, float]] = None,
        T: Optional[Union[Type[float], Type[int]]] = None,
    ) -> ContingencyTable:
        """Release the ContingencyTable."""
        return super().release(data, bounds, T)
