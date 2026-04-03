"""
This module requires extra installs: ``pip install 'opendp[polars]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.polars.contingency_table``.
"""

from typing import Mapping, Optional, Sequence, Type, Union
from opendp.context import Query
from opendp.extras.mbi._table import ContingencyTable
from opendp.extras.mbi import Algorithm, make_contingency_table


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
        keys: Optional[Mapping[str, Sequence]],
        cuts: Optional[Mapping[str, Sequence[float]]],
        table: Optional[ContingencyTable],
        algorithm: Union[Algorithm],
        **kwargs
    ):
        super().__init__(**kwargs)
        self.oneway_scale = oneway_scale
        self.oneway_threshold = oneway_threshold
        self.keys = keys
        self.cuts = cuts
        self.table = table
        self.algorithm = algorithm

    def resolve(
        self,
        allow_transformations: bool = False,
        bounds: Optional[tuple[float, float]] = None,
        T: Optional[Union[Type[float], Type[int]]] = None,
        **kwargs,
    ):
        if allow_transformations or bounds is not None or T is not None:
            return super().resolve(
                allow_transformations=allow_transformations,
                bounds=bounds,
                T=T,
                **kwargs,
            )

        t_plan = self._chain
        d_in = t_plan.map(self._d_in)
        d_out = self._resolve_d_out(kwargs)

        m_table, oneway_scale, oneway_threshold = make_contingency_table(
            input_domain=t_plan.output_domain,
            input_metric=t_plan.output_metric,
            output_measure=self._output_measure,
            d_in=d_in,
            d_out=d_out,  # type: ignore[arg-type]
            keys=self.keys,
            cuts=self.cuts,
            table=self.table,
            algorithm=self.algorithm,
        )
        self.oneway_scale = oneway_scale
        self.oneway_threshold = oneway_threshold
        return t_plan >> m_table

    def release(
        self,
        data=None,
        bounds: Optional[tuple[float, float]] = None,
        T: Optional[Union[Type[float], Type[int]]] = None,
        **kwargs,
    ) -> ContingencyTable:
        """Release the ContingencyTable."""
        return super().release(data, bounds, T, **kwargs)
