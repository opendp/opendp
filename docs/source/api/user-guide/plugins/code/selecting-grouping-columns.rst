:orphan:

.. code:: pycon

    # plugins
    >>> import opendp.prelude as dp
    >>> import pandas as pd
    >>> import random
    >>> from itertools import chain, combinations
    >>> def make_grouping_cols_score(
    ...     candidates, min_bin_contributions
    ... ):
    ...     r"""Create a transformation that assesses the utility of each candidate in `candidates`."""
    ...     dp.assert_features("contrib")
    ...     def score(x: pd.DataFrame, c):
    ...         return (
    ...             (
    ...                 x.groupby(list(c)).size()
    ...                 >= min_bin_contributions
    ...             )
    ...             .sum()
    ...             .astype(float)
    ...         )
    ...     return dp.t.make_user_transformation(
    ...         input_domain=dp.user_domain(
    ...             "PandasDomain",
    ...             member=lambda x: isinstance(x, pd.DataFrame),
    ...         ),
    ...         input_metric=dp.symmetric_distance(),
    ...         output_domain=dp.vector_domain(
    ...             dp.atom_domain(T=float, nan=False)
    ...         ),
    ...         output_metric=dp.linf_distance(
    ...             T=float, monotonic=True
    ...         ),
    ...         function=lambda x: [
    ...             score(x, c) for c in candidates
    ...         ],
    ...         stability_map=lambda d_in: float(d_in),
    ...     )
    ...
    >>> def make_select_grouping_cols(
    ...     candidates, min_bin_size, scale
    ... ):
    ...     """Create a measurement that selects a set of grouping columns from `candidates`."""
    ...     return (
    ...         make_grouping_cols_score(candidates, min_bin_size)
    ...         >> dp.m.then_noisy_max(dp.pure_dp(), scale)
    ...         >> (lambda idx: candidates[idx])
    ...     )
    ...

    # /plugins

    # dp-mechanism
    >>> row_count = 50
    >>> col_count = 4
    >>> private_data = pd.DataFrame(
    ...     {
    ...         **{
    ...             f"too_uniform_{n}": [
    ...                 random.randint(0, 1)
    ...                 for _ in range(row_count)
    ...             ]
    ...             for n in range(col_count)
    ...         },
    ...         **{
    ...             f"too_diverse_{n}": [
    ...                 random.randint(0, row_count)
    ...                 for _ in range(row_count)
    ...             ]
    ...             for n in range(col_count)
    ...         },
    ...         **{
    ...             f"just_right_{n}": [
    ...                 random.randint(0, 20)
    ...                 for _ in range(row_count)
    ...             ]
    ...             for n in range(col_count)
    ...         },
    ...     }
    ... )
    >>> def powerset(iterable):
    ...     s = list(iterable)
    ...     return chain.from_iterable(
    ...         combinations(s, r) for r in range(1, len(s) + 1)
    ...     )
    ...
    >>> candidates = list(powerset(private_data.columns))
    >>> dp.enable_features("honest-but-curious", "contrib")
    >>> m_select_gcols = make_select_grouping_cols(
    ...     candidates=candidates,
    ...     min_bin_size=89,
    ...     scale=10.0,
    ... )
    >>> print("ε =", m_select_gcols.map(d_in=1))
    ε = 0.1

    # /dp-mechanism

    # dp-release
    >>> dp_selected_grouping_columns = m_select_gcols(private_data)
    >>> dp_selected_grouping_columns  # doctest: +ELLIPSIS
    (...)

    # /dp-release
