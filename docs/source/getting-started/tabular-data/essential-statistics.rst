Essential Statistics
====================

This section demonstrates how to use the OpenDP Library to compute
essential statistical measures with `Polars <https://docs.pola.rs/>`__.

-  Count

   -  of rows in frame, including nulls (``len()``)
   -  of rows in column, including nulls (``.len()``)
   -  of rows in column, excluding nulls (``.count()``)
   -  of rows in column, exclusively nulls (``.null_count()``)
   -  of *unique* rows in column, including null (``.n_unique()``)

-  Sum (``.sum(bounds)``)
-  Mean (``.mean(bounds)``)
-  Quantile (``.quantile(alpha, candidates)``)

   -  Median (``.median(candidates)``)

We will use `sample
data <https://github.com/opendp/dp-test-datasets/blob/main/data/eurostat/README.ipynb>`__
from the Labour Force Survey in France.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import polars as pl 
            >>> import opendp.prelude as dp
            
            >>> dp.enable_features("contrib")
            

To get started, we’ll recreate the Context from the `tabular data
introduction <index.rst>`__.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=5,
            ... )

.. note::

    In practice, it is recommended to only ever create one Context that spans all queries you may make on your data.

    However, to more clearly explain the functionality of the library, the following examples do not follow this recommendation.
            

Count
-----

The simplest query is a count of the number of records in a dataset.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_num_responses = context.query().select(dp.len())
            

If you have not used Polars before, please familiarize yourself with the
query syntax by reading `Polars’ Getting
Started <https://docs.pola.rs/user-guide/getting-started/>`__. OpenDP
specifically targets the `lazy API, not the eager
API <https://docs.pola.rs/user-guide/concepts/lazy-vs-eager/>`__.

You can retrieve information about the noise scale and mechanism before
committing to a release:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_num_responses.summarize(alpha=0.05)
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (1, 5)</small><table border="1" class="dataframe"><thead><tr><th>column</th><th>aggregate</th><th>distribution</th><th>scale</th><th>accuracy</th></tr><tr><td>str</td><td>str</td><td>str</td><td>f64</td><td>f64</td></tr></thead><tbody><tr><td>&quot;len&quot;</td><td>&quot;Frame Length&quot;</td><td>&quot;Integer Laplace&quot;</td><td>180.0</td><td>539.731115</td></tr></tbody></table></div>



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

        .. code:: python

            >>> query_num_responses.summarize(alpha=0.01)
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (1, 5)</small><table border="1" class="dataframe"><thead><tr><th>column</th><th>aggregate</th><th>distribution</th><th>scale</th><th>accuracy</th></tr><tr><td>str</td><td>str</td><td>str</td><td>f64</td><td>f64</td></tr></thead><tbody><tr><td>&quot;len&quot;</td><td>&quot;Frame Length&quot;</td><td>&quot;Integer Laplace&quot;</td><td>180.0</td><td>829.429939</td></tr></tbody></table></div>



The DP ``len`` estimate will differ from the true ``len`` by no more
than the given accuracy with 1 - alpha = 99% confidence.

Assuming this level of utility justifies the loss of privacy (ε = 0.2),
release the query:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_num_responses.release().collect().item()
            200215

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

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=5,
            ...     # NEW CODE STARTING HERE
            ...     margins={
            ...         # when data is not grouped (empty tuple)...
            ...         (): dp.polars.Margin(
            ...             # ...the biggest (and only) partition is no larger than
            ...             #    France population * number of quarters
            ...             max_partition_length=60_000_000 * 36
            ...         ),
            ...     },
            ... )
            

Each ``dp.polars.Margin`` contains descriptors about the dataset when
grouped by columns. Since we’re not yet grouping, the tuple of grouping
columns is empty: ``()``. The OpenDP Library references this margin when
you use ``.select`` in a query.

This margin provides an upper bound on how large any partition can be
(``max_partition_length``). An adversary could very reasonably surmise
that there are no more responses in each quarter than the population of
France. The population of France was about 60 million in 2004 so we’ll
use that as our maximum partition length. Source: `World
Bank <https://datatopics.worldbank.org/world-development-indicators/>`__.
By giving up this relatively inconsequential fact about the data to a
potential adversary, the library is able to ensure that overflow and/or
numerical instability won’t undermine privacy guarantees.

Now that you’ve become acquainted with margins, lets release some
queries that make use of it. We start by releasing the total number of
work hours across responses.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_work_hours = (
            ...     # 99 represents "Not applicable"
            ...     context.query().filter(pl.col("HWUSUAL") != 99.0)
            ...     # compute the DP sum
            ...     .select(pl.col.HWUSUAL.cast(int).fill_null(35).dp.sum(bounds=(0, 80)))
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

   Do not use private data to calculate imputed values or bounds: This
   could leak private information, reducing the integrity of the privacy
   guarantee. Instead, choose bounds and imputed values based on prior
   domain knowledge.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_work_hours.summarize(alpha=0.05)
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (1, 5)</small><table border="1" class="dataframe"><thead><tr><th>column</th><th>aggregate</th><th>distribution</th><th>scale</th><th>accuracy</th></tr><tr><td>str</td><td>str</td><td>str</td><td>f64</td><td>f64</td></tr></thead><tbody><tr><td>&quot;HWUSUAL&quot;</td><td>&quot;Sum&quot;</td><td>&quot;Integer Laplace&quot;</td><td>14400.0</td><td>43139.04473</td></tr></tbody></table></div>



The noise scale 1440 comes from the product of 36 (number of
contributions), 80 (max number of work hours) and 5 (number of queries).

If you were to release ``query_work_hours``, then the DP sum estimate
will differ from the *clipped* sum by no more than the given accuracy
with 1 - alpha = 95% confidence. Notice that the accuracy estimate does
not take into account bias introduced by clipping responses.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_work_hours.release().collect()
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (1, 1)</small><table border="1" class="dataframe"><thead><tr><th>HWUSUAL</th></tr><tr><td>i64</td></tr></thead><tbody><tr><td>2964398</td></tr></tbody></table></div>



Even though the accuracy estimate may have seemed large, in retrospect
we see it is actually quite tight. Our noisy release of nearly 3 million
work hours likely only differs from total clipped work hours by no more
than 43k.

One adjustment made to get better utility was to change the data type we
are summing to an integer. When the ``max_partition_length`` is very
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

        .. code:: python

            >>> query_work_hours = (
            ...     context.query().filter(pl.col.HWUSUAL != 99.0)
            ...     # release both the sum and length in one query
            ...     .select(pl.col.HWUSUAL.cast(int).fill_null(35).dp.sum(bounds=(0, 80)), dp.len())
            ... )
            
            >>> query_work_hours.summarize(alpha=0.05)
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (2, 5)</small><table border="1" class="dataframe"><thead><tr><th>column</th><th>aggregate</th><th>distribution</th><th>scale</th><th>accuracy</th></tr><tr><td>str</td><td>str</td><td>str</td><td>f64</td><td>f64</td></tr></thead><tbody><tr><td>&quot;HWUSUAL&quot;</td><td>&quot;Sum&quot;</td><td>&quot;Integer Laplace&quot;</td><td>28800.0</td><td>86277.589474</td></tr><tr><td>&quot;len&quot;</td><td>&quot;Frame Length&quot;</td><td>&quot;Integer Laplace&quot;</td><td>360.0</td><td>1078.963271</td></tr></tbody></table></div>



This joint query satisfies the same privacy guarantee as each of the
previous individual queries, by adding twice as much noise to each
query.

You can also reuse the same noisy count estimate to estimate several
means on different columns.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # release and create mean column
            >>> query_work_hours.release().collect().with_columns(mean=pl.col.HWUSUAL / pl.col.len)
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (1, 3)</small><table border="1" class="dataframe"><thead><tr><th>HWUSUAL</th><th>len</th><th>mean</th></tr><tr><td>i64</td><td>u32</td><td>f64</td></tr></thead><tbody><tr><td>2974264</td><td>78140</td><td>38.063271</td></tr></tbody></table></div>



If the dataset size is an invariant (bounded-DP), then only the sums
need to be released, so we recommend using ``.dp.mean``. Specify this
data invariant in the margin: ``public_info="lengths"``.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # apply some preprocessing outside of OpenDP (see note below)
            >>> # drops "Not applicable" values
            >>> data = pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True).filter(pl.col.HWUSUAL != 99)
            
            >>> # apply domain descriptors (margins) to preprocessed data
            >>> context_bounded_dp = dp.Context.compositor(
            ...     data=data,
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=5,
            ...     margins={
            ...         (): dp.polars.Margin(
            ...             max_partition_length=60_000_000 * 36,
            ...             # ADDITIONAL CODE STARTING HERE
            ...             # make partition size public (bounded-DP)
            ...             public_info="lengths",
            ...         ),
            ...     },
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

        .. code:: python

            >>> query_mean_work_hours = context_bounded_dp.query().select(
            ...     pl.col.HWUSUAL.cast(int).fill_null(35).dp.mean(bounds=(0, 80))
            ... )
            

When ``public_info="lengths"`` is set, the number of records in the data
is not protected (for those familiar with DP terminology, this is
equivalent to bounded-DP). Therefore when computing the mean, a noisy
sum is released and subsequently divided by the exact length. This
behavior can be observed in the query summary:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_mean_work_hours.summarize(alpha=0.05)
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (2, 5)</small><table border="1" class="dataframe"><thead><tr><th>column</th><th>aggregate</th><th>distribution</th><th>scale</th><th>accuracy</th></tr><tr><td>str</td><td>str</td><td>str</td><td>f64</td><td>f64</td></tr></thead><tbody><tr><td>&quot;HWUSUAL&quot;</td><td>&quot;Sum&quot;</td><td>&quot;Integer Laplace&quot;</td><td>7200.0</td><td>21569.772352</td></tr><tr><td>&quot;HWUSUAL&quot;</td><td>&quot;Length&quot;</td><td>&quot;Integer Laplace&quot;</td><td>0.0</td><td>NaN</td></tr></tbody></table></div>



.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_mean_work_hours.release().collect()
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (1, 1)</small><table border="1" class="dataframe"><thead><tr><th>HWUSUAL</th></tr><tr><td>f64</td></tr></thead><tbody><tr><td>37.642692</td></tr></tbody></table></div>



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

        .. code:: python

            >>> candidates = list(range(20, 60))
            
            >>> query_median_hours = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99.0)
            ...     .select(pl.col.HWUSUAL.fill_null(35).dp.median(candidates))
            ... )
            >>> query_median_hours.summarize(alpha=0.05)
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (1, 5)</small><table border="1" class="dataframe"><thead><tr><th>column</th><th>aggregate</th><th>distribution</th><th>scale</th><th>accuracy</th></tr><tr><td>str</td><td>str</td><td>str</td><td>f64</td><td>f64</td></tr></thead><tbody><tr><td>&quot;HWUSUAL&quot;</td><td>&quot;0.5-Quantile&quot;</td><td>&quot;GumbelMin&quot;</td><td>360.0</td><td>null</td></tr></tbody></table></div>



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

        .. code:: python

            >>> query_median_hours.release().collect()
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (1, 1)</small><table border="1" class="dataframe"><thead><tr><th>HWUSUAL</th></tr><tr><td>i64</td></tr></thead><tbody><tr><td>37</td></tr></tbody></table></div>



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

        .. code:: python

            >>> query_multi_quantiles = (
            ...     context.query()
            ...     .filter(pl.col.HWUSUAL != 99.0)
            ...     .select(
            ...         pl.col.HWUSUAL.fill_null(35).dp.quantile(a, candidates).alias(f"{a}-Quantile")
            ...         for a in [0.25, 0.5, 0.75]
            ...     )
            ... )
            >>> query_multi_quantiles.summarize()
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (3, 4)</small><table border="1" class="dataframe"><thead><tr><th>column</th><th>aggregate</th><th>distribution</th><th>scale</th></tr><tr><td>str</td><td>str</td><td>str</td><td>f64</td></tr></thead><tbody><tr><td>&quot;0.25-Quantile&quot;</td><td>&quot;0.25-Quantile&quot;</td><td>&quot;GumbelMin&quot;</td><td>3240.0</td></tr><tr><td>&quot;0.5-Quantile&quot;</td><td>&quot;0.5-Quantile&quot;</td><td>&quot;GumbelMin&quot;</td><td>1080.0</td></tr><tr><td>&quot;0.75-Quantile&quot;</td><td>&quot;0.75-Quantile&quot;</td><td>&quot;GumbelMin&quot;</td><td>3240.0</td></tr></tbody></table></div>



When you do not set the scale parameter yourself, the privacy budget is
distributed evenly across each statistic. Judging from the scale
parameters in the summary table, it may seem that more of the privacy
budget was allocated for the median, but this is only due to internal
implementation details.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> query_multi_quantiles.release().collect()
            

.. raw:: html

    <div><style>
    .dataframe > thead > tr,
    .dataframe > tbody > tr {
      text-align: right;
      white-space: pre-wrap;
    }
    </style>
    <small>shape: (1, 3)</small><table border="1" class="dataframe"><thead><tr><th>0.25-Quantile</th><th>0.5-Quantile</th><th>0.75-Quantile</th></tr><tr><td>i64</td><td>i64</td><td>i64</td></tr></thead><tbody><tr><td>35</td><td>37</td><td>40</td></tr></tbody></table></div>



Since work hours tend to be concentrated a little less than 40, this
release seems reasonable.

Throughout this notebook, all ``.dp`` expressions take an optional scale
parameter that can be used to more finely control how much noise is
added to queries. The library then rescales all of these parameters up
or down to satisfy a global privacy guarantee.

Now that you have a handle on the essential statistics, the next section
will introduce you to applying these statistics over groupings of your
data.
