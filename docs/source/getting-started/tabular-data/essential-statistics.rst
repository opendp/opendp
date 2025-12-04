Essential Statistics
====================

This section demonstrates how to use the OpenDP Library to compute
essential statistical measures with `Polars <https://docs.pola.rs/>`__.

- Count

  - of rows in frame, including nulls (``len()``)
  - of rows in column, including nulls (``.len()``)
  - of rows in column, excluding nulls (``.count()``)
  - of rows in column, exclusively nulls (``.null_count()``)
  - of *unique* rows in column, including null (``.n_unique()``)

- Sum (``.sum(bounds)``)
- Mean (``.mean(bounds)``)
- Quantile (``.quantile(alpha, candidates)``)

  - Median (``.median(candidates)``)

We will use `sample
data <https://github.com/opendp/dp-test-datasets/blob/main/data/eurostat/README.ipynb>`__
from the Labour Force Survey in France.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import polars as pl
            >>> import opendp.prelude as dp

            >>> dp.enable_features("contrib")


To get started, we’ll recreate the Context from the `tabular data
introduction <index.rst>`__.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(
            ...         dp.examples.get_france_lfs_path(),
            ...         ignore_errors=True,
            ...     ),
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=5,
            ... )

.. note::
    We'll consistently use ``ignore_errors=True``:
    Polars infers types from the first rows of a CSV, but without this setting,
    if a later value can not be parsed as the inferred type it will error.
    That's not the behavior we want:
    Runtime errors would leak information and violate the DP guarantee.
            

Count
-----

The simplest query is a count of the number of records in a dataset.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query_num_responses = context.query().select(dp.len())


If you have not used Polars before, please familiarize yourself with the
query syntax by reading `Polars’ Getting
Started <https://docs.pola.rs/user-guide/getting-started/>`__. OpenDP
specifically targets the `lazy API, not the eager
API <https://docs.pola.rs/user-guide/concepts/lazy-api/>`__.

You can retrieve information about the noise scale and mechanism before
committing to a release:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query_num_responses.summarize(alpha=0.05)
            shape: (1, 5)
            ┌────────┬──────────────┬─────────────────┬───────┬────────────┐
            │ column ┆ aggregate    ┆ distribution    ┆ scale ┆ accuracy   │
            │ ---    ┆ ---          ┆ ---             ┆ ---   ┆ ---        │
            │ str    ┆ str          ┆ str             ┆ f64   ┆ f64        │
            ╞════════╪══════════════╪═════════════════╪═══════╪════════════╡
            │ len    ┆ Frame Length ┆ Integer Laplace ┆ 180.0 ┆ 539.731115 │
            └────────┴──────────────┴─────────────────┴───────┴────────────┘


When this query is released, Laplacian noise is added with a scale
parameter of 180 (for those interested in the math, the scale in this
case is the sensitivity divided by epsilon, where sensitivity is 36 and
ε is 0.2).

Since alpha was specified, if you were to release
``query_num_responses``, then the DP ``len`` estimate will differ from
the true ``len`` by no more than the given accuracy with 1 - alpha = 95%
confidence.

For comparison, the accuracy interval becomes *larger* when the level of
significance becomes smaller:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query_num_responses.summarize(alpha=0.01)
            shape: (1, 5)
            ┌────────┬──────────────┬─────────────────┬───────┬────────────┐
            │ column ┆ aggregate    ┆ distribution    ┆ scale ┆ accuracy   │
            │ ---    ┆ ---          ┆ ---             ┆ ---   ┆ ---        │
            │ str    ┆ str          ┆ str             ┆ f64   ┆ f64        │
            ╞════════╪══════════════╪═════════════════╪═══════╪════════════╡
            │ len    ┆ Frame Length ┆ Integer Laplace ┆ 180.0 ┆ 829.429939 │
            └────────┴──────────────┴─────────────────┴───────┴────────────┘


The DP ``len`` estimate will differ from the true ``len`` by no more
than the given accuracy with 1 - alpha = 99% confidence.

Assuming this level of utility justifies the loss of privacy (ε = 0.2),
release the query:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> print(
            ...     "len:", query_num_responses.release().collect().item()
            ... )  # doctest: +ELLIPSIS
            len: ...

Other variations of counting queries are discussed in the `Aggregation
section <../../api/user-guide/polars/expressions/aggregation.ipynb>`__.

Sum
---

In this section we compute a privacy-preserving total of work hours
across all responses.

The OpenDP Library ensures that privacy guarantees take into account the
potential for overflow and/or numerical instability. For this reason,
many statistics require a known upper bound on how many records can be
present in the data. This descriptor will need to be provided when you
first construct the Context, in the form of a *margin*. A margin is used
to describe certain properties that a potential adversary would already
know about the data.

.. note::
    We recommend creating just one Context for all your queries.
    However, to help explain the functionality of the library,
    the following examples create new Contexts with different settings.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(
            ...         dp.examples.get_france_lfs_path(),
            ...         ignore_errors=True,
            ...     ),
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=5,
            ...     # NEW CODE STARTING HERE
            ...     margins=[
            ...         dp.polars.Margin(
            ...             # The length of the data is no greater than:
            ...             #    average quarterly survey size * number of quarters
            ...             # (both public)
            ...             max_length=150_000
            ...             * 36
            ...             # Remember to only use public information
            ...             # when determining max_length.
            ...         ),
            ...     ],
            ... )


Each ``dp.polars.Margin`` contains descriptors about the dataset when
grouped by columns. Since we’re not yet grouping, the grouping columns
(``by``) defaults to empty (``[]``). The OpenDP Library references this
margin when you use ``.select`` in a query.

This margin provides an upper bound on how large any group can be
(``max_length``). Since the average achieved sample size is shared
`50,000
households <https://ec.europa.eu/eurostat/documents/7870049/19469785/KS-FT-24-003-EN-N.pdf/f8f6f54b-8504-0388-f754-abb004902f45?version=1.0&t=1719410273207>`__,
and the average number of individuals in households is `less than
three <https://www.globaldata.com/data-insights/macroeconomic/average-household-size-in-france-2096123/>`__,
we can use 150,000 as a conservative upper bound on the number of
records per quarter. By giving up this relatively inconsequential fact
about the data to a potential adversary, the library is able to ensure
that overflow and/or numerical instability won’t undermine privacy
guarantees.

Now that you’ve become acquainted with margins, lets release some
queries that make use of it. We start by releasing the total number of
work hours across responses.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query_work_hours = (
            ...     # 99 represents "Not applicable"
            ...     context.query().filter(pl.col("HWUSUAL") != 99.0)
            ...     # compute the DP sum
            ...     .select(
            ...         pl.col.HWUSUAL.cast(int)
            ...         .fill_null(35)
            ...         .dp.sum(bounds=(0, 80))
            ...     )
            ... )


This query uses an expression ``.dp.sum`` that clips the range of each
response, sums, and then adds sufficient noise to satisfy the
differential privacy guarantee.

Since the sum is sensitive to null values, OpenDP also requires that
inputs are not null. ``.fill_null`` fulfills this requirement by
imputing null values with the provided expression. In this case we fill
with 35, which, based on other public information, is the average number
of weekly work hours in France. Your choice of imputation value will
vary depending on how you want to use the statistic.

.. note::
   Do not use private data to calculate imputed values or bounds: This
   could leak private information, reducing the integrity of the privacy
   guarantee. Instead, choose bounds and imputed values based on prior
   domain knowledge.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query_work_hours.summarize(alpha=0.05)
            shape: (1, 5)
            ┌─────────┬───────────┬─────────────────┬─────────┬─────────────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale   ┆ accuracy    │
            │ ---     ┆ ---       ┆ ---             ┆ ---     ┆ ---         │
            │ str     ┆ str       ┆ str             ┆ f64     ┆ f64         │
            ╞═════════╪═══════════╪═════════════════╪═════════╪═════════════╡
            │ HWUSUAL ┆ Sum       ┆ Integer Laplace ┆ 14400.0 ┆ 43139.04473 │
            └─────────┴───────────┴─────────────────┴─────────┴─────────────┘


The noise scale 14,400 comes from the product of 36 (number of
contributions), 80 (max number of work hours) and 5 (number of queries).

If you were to release ``query_work_hours``, then the DP sum estimate
will differ from the *clipped* sum by no more than the given accuracy
with 1 - alpha = 95% confidence. Notice that the accuracy estimate does
not take into account bias introduced by clipping responses.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> print(
            ...     "HWUSUAL:", query_work_hours.release().collect().item()
            ... )  # doctest: +ELLIPSIS
            HWUSUAL: ...


Even though the accuracy estimate may have seemed large, in retrospect
we see it is actually quite tight. Our noisy release of nearly 3 million
work hours likely only differs from total clipped work hours by no more
than 43k.

One adjustment made to get better utility was to change the data type we
are summing to an integer. When the ``max_length`` of a group is very
large, the worst-case error from summing floating-point numbers also
becomes very large. This numerical imprecision can significantly impact
the utility of the release.

Mean
----

Under the default setting where individuals may add or remove records,
we recommended estimating means by separately releasing sum and count
estimates.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query_work_hours = (
            ...     context.query().filter(pl.col.HWUSUAL != 99.0)
            ...     # release both the sum and length in one query
            ...     .select(
            ...         # if the imputation is omitted, 
            ...         # a midpoint imputation is inserted (40)
            ...         pl.col.HWUSUAL.cast(int).dp.sum(bounds=(0, 80)),
            ...         dp.len(),
            ...     )
            ... )

            >>> query_work_hours.summarize(alpha=0.05)
            shape: (2, 5)
            ┌─────────┬──────────────┬─────────────────┬─────────┬──────────────┐
            │ column  ┆ aggregate    ┆ distribution    ┆ scale   ┆ accuracy     │
            │ ---     ┆ ---          ┆ ---             ┆ ---     ┆ ---          │
            │ str     ┆ str          ┆ str             ┆ f64     ┆ f64          │
            ╞═════════╪══════════════╪═════════════════╪═════════╪══════════════╡
            │ HWUSUAL ┆ Sum          ┆ Integer Laplace ┆ 28800.0 ┆ 86277.589474 │
            │ len     ┆ Frame Length ┆ Integer Laplace ┆ 360.0   ┆ 1078.963271  │
            └─────────┴──────────────┴─────────────────┴─────────┴──────────────┘


This joint query satisfies the same privacy guarantee as each of the
previous individual queries, by adding twice as much noise to each
query.

You can also reuse the same noisy count estimate to estimate several
means on different columns.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # release and create mean column
            >>> query_work_hours.release().collect().with_columns(
            ...     mean=pl.col.HWUSUAL / pl.col.len
            ... )  # doctest: +FUZZY_DF
            shape: (1, 3)
            ┌──────────┬─────────┬───────────┐
            │ HWUSUAL  ┆ len     ┆ mean      │
            │ ---      ┆ ---     ┆ ---       │
            │ i64      ┆ u32     ┆ f64       │
            ╞══════════╪═════════╪═══════════╡
            │ ...      ┆ ...     ┆ ...       │
            └──────────┴─────────┴───────────┘


If the dataset size is an invariant (bounded-DP), then only the sums
need to be released, so we recommend using ``.dp.mean``. Specify this
data invariant in the margin: ``invariant="lengths"``.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # apply some preprocessing outside of OpenDP (see note below)
            >>> # drops "Not applicable" values
            >>> data = pl.scan_csv(
            ...     dp.examples.get_france_lfs_path(), ignore_errors=True
            ... ).filter(pl.col.HWUSUAL != 99)

            >>> # apply domain descriptors (margins) to preprocessed data
            >>> context_bounded_dp = dp.Context.compositor(
            ...     data=data,
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=5,
            ...     margins=[
            ...         dp.polars.Margin(
            ...             max_length=150_000 * 36,
            ...             # ADDITIONAL CODE STARTING HERE
            ...             # don't protect the total number of records (bounded-DP)
            ...             invariant="lengths",
            ...         ),
            ...     ],
            ... )


OpenDP accounts for the effect of data preparation on the privacy
guarantee, so we generally recommend preparing data in OpenDP. However,
in this setting the filter makes the number of records unknown to the
adversary, dropping the ``"lengths"`` descriptor from the margin
metadata that we intended to use for the mean release.

Assuming that it is truly the number of *applicable* ``HWUSUAL``
responses that is public information, and that the filter won’t affect
the privacy guarantee, then you could initialize the context with
filtered data, as shown above.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query_mean_work_hours = context_bounded_dp.query().select(
            ...     pl.col.HWUSUAL.cast(int).dp.mean(bounds=(0, 80))
            ... )


When ``invariant="lengths"`` is set, the number of records in the data
is not protected (for those familiar with DP terminology, this is
equivalent to bounded-DP). Therefore when computing the mean, a noisy
sum is released and subsequently divided by the exact length. This
behavior can be observed in the query summary:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query_mean_work_hours.summarize(alpha=0.05)
            shape: (2, 5)
            ┌─────────┬───────────┬─────────────────┬────────┬──────────────┐
            │ column  ┆ aggregate ┆ distribution    ┆ scale  ┆ accuracy     │
            │ ---     ┆ ---       ┆ ---             ┆ ---    ┆ ---          │
            │ str     ┆ str       ┆ str             ┆ f64    ┆ f64          │
            ╞═════════╪═══════════╪═════════════════╪════════╪══════════════╡
            │ HWUSUAL ┆ Sum       ┆ Integer Laplace ┆ 7200.0 ┆ 21569.772352 │
            │ HWUSUAL ┆ Length    ┆ Integer Laplace ┆ 0.0    ┆ NaN          │
            └─────────┴───────────┴─────────────────┴────────┴──────────────┘

            >>> print(
            ...     "mean:",
            ...     query_mean_work_hours.release().collect().item(),
            ... )
            mean: ...

To recap, we’ve shown how to estimate linear statistics like counts,
sums and means. These estimates were all released via output
perturbation (adding noise to a value).

Median
------

Unfortunately, output perturbation does not work well for releasing
private medians (``.dp.median``) and quantiles (``.dp.quantile``).
Instead of passing bounds, the technique used to release these
quantities requires you specify ``candidates``, which are potential
outcomes to be selected from. The expression privately selects the
candidate that is nearest to the true median (or quantile).

For example, to privately release the median over ``HWUSUAL`` you might
set candidates to whole numbers between 20 and 60:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> candidates = list(range(20, 60))

            >>> query_median_hours = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99.0)
            ...     .select(
            ...         pl.col.HWUSUAL.cast(int).dp.median(candidates)
            ...     )
            ... )
            >>> query_median_hours.summarize(alpha=0.05)
            shape: (1, 5)
            ┌─────────┬──────────────┬────────────────┬───────┬──────────┐
            │ column  ┆ aggregate    ┆ distribution   ┆ scale ┆ accuracy │
            │ ---     ┆ ---          ┆ ---            ┆ ---   ┆ ---      │
            │ str     ┆ str          ┆ str            ┆ f64   ┆ f64      │
            ╞═════════╪══════════════╪════════════════╪═══════╪══════════╡
            │ HWUSUAL ┆ 0.5-Quantile ┆ ExponentialMin ┆ 360.0 ┆ null     │
            └─────────┴──────────────┴────────────────┴───────┴──────────┘


The ``aggregate`` value shows “0.5-Quantile” because ``.dp.median``
internally just calls ``.dp.quantile`` with an alpha parameter set to
0.5.

This time the accuracy estimate is unknown because the algorithm isn’t
directly adding noise: it’s scoring each candidate, adding noise to each
score, and then releasing the candidate with the best noisy score. While
this approach results in much better utility than output perturbation
would for this kind of query, it prevents us from providing accuracy
estimates.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> print(
            ...     "median:", query_median_hours.release().collect()
            ... )  # doctest: +ELLIPSIS
            median: ...


This median estimate is consistent with the mean estimate from the
previous section.

Quantile
--------

``.dp.quantile`` additionally requires an alpha parameter between zero
and one, designating the proportion of records less than the desired
release.

For example, the following query computes the three quartiles of work
hours:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query_multi_quantiles = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99.0)
            ...     .select(
            ...         pl.col.HWUSUAL.cast(int)
            ...         .dp.quantile(a, candidates)
            ...         .alias(f"{a}-Quantile")
            ...         for a in [0.25, 0.5, 0.75]
            ...     )
            ... )
            >>> query_multi_quantiles.summarize()
            shape: (3, 4)
            ┌───────────────┬───────────────┬────────────────┬────────┐
            │ column        ┆ aggregate     ┆ distribution   ┆ scale  │
            │ ---           ┆ ---           ┆ ---            ┆ ---    │
            │ str           ┆ str           ┆ str            ┆ f64    │
            ╞═══════════════╪═══════════════╪════════════════╪════════╡
            │ 0.25-Quantile ┆ 0.25-Quantile ┆ ExponentialMin ┆ 3240.0 │
            │ 0.5-Quantile  ┆ 0.5-Quantile  ┆ ExponentialMin ┆ 1080.0 │
            │ 0.75-Quantile ┆ 0.75-Quantile ┆ ExponentialMin ┆ 3240.0 │
            └───────────────┴───────────────┴────────────────┴────────┘

When you do not set the scale parameter yourself, the privacy budget is
distributed evenly across each statistic. Judging from the scale
parameters in the summary table, it may seem that more of the privacy
budget was allocated for the median, but this is only due to internal
implementation details.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> query_multi_quantiles.release().collect()  # doctest: +FUZZY_DF
            shape: (1, 3)
            ┌───────────────┬──────────────┬───────────────┐
            │ 0.25-Quantile ┆ 0.5-Quantile ┆ 0.75-Quantile │
            │ ---           ┆ ---          ┆ ---           │
            │ i64           ┆ i64          ┆ i64           │
            ╞═══════════════╪══════════════╪═══════════════╡
            │ ...           ┆ ...          ┆ ...           │
            └───────────────┴──────────────┴───────────────┘


Since work hours tend to be concentrated a little less than 40, this
release seems reasonable.

Now that you have a handle on the essential statistics, the next section
will introduce you to applying these statistics over groupings of your
data.
