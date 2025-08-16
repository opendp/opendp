Gaussian Noise
==============

Gaussian noise has several benefits over Laplace noise, and is commonly
used for differentially private data releases. OpenDP automatically
chooses the noise distribution based on the definition of privacy.

========== ============
Definition Distribution
========== ============
Pure-DP    Laplace
zCDP       Gaussian
========== ============

While the documentation is generally written under pure-DP (or
approximate-DP), you can easily switch to zCDP (or approximate-zCDP) by
simply changing the privacy loss:

``dp.loss_of(epsilon=1.0)`` → ``dp.loss_of(rho=0.1)``

The following code repeats the same initial release in the `Essential
Statistics <../../../getting-started/tabular-data/essential-statistics.ipynb>`__
documentation on the Labour Force Survey, but under zero-concentrated
differential privacy, resulting in Gaussian noise perturbation instead
of Laplace noise perturbation.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import polars as pl 
            >>> import opendp.prelude as dp
            
            >>> dp.enable_features("contrib")
            
            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(rho=0.1),
            ...     split_evenly_over=5,
            ... )
            
            >>> query_num_responses = context.query().select(dp.len())
            >>> query_num_responses.summarize(alpha=0.05)
            shape: (1, 5)
            ┌────────┬──────────────┬──────────────────┬───────┬──────────┐
            │ column ┆ aggregate    ┆ distribution     ┆ scale ┆ accuracy │
            │ ---    ┆ ---          ┆ ---              ┆ ---   ┆ ---      │
            │ str    ┆ str          ┆ str              ┆ f64   ┆ f64      │
            ╞════════╪══════════════╪══════════════════╪═══════╪══════════╡
            │ len    ┆ Frame Length ┆ Integer Gaussian ┆ 180.0 ┆ 354.0    │
            └────────┴──────────────┴──────────────────┴───────┴──────────┘


Any other code example will switch and work in the same way (so long as
the noise distribution isn’t explicitly specified in the query).

Distribution Comparison
-----------------------

- Adding Gaussian noise can simplify further statistical analysis of the
  release that relies on a normality assumption.
- The Gaussian mechanism cannot satisfy pure differential privacy,
  instead satisfying the weaker definition of approximate differential
  privacy.
- The Gaussian mechanism affords greater utility (adds less overall
  noise) for a similar privacy guarantee when answering many queries.
- The Laplace mechanism adds noise proportional to sensitivity based on
  the :math:`L_1` distance, whereas the Gaussian mechanism adds noise
  proportional to sensitivity based on the :math:`L_2` distance.

Lets take a closer look at how the difference in the sensitivity’s
metric can translate to significantly less noise.

Sensitivity
-----------

The :math:`L_2` distance (euclidean distance) is not as sensitive to
small changes along many different axes as the :math:`L_1` distance
(taxicab distance) is. This makes intuitive sense: when traveling
between two corners of a unit square, the taxicab distance is :math:`2`,
whereas the distance as the crow flies is just :math:`\sqrt{2}`. Better
yet, as the dimensionality :math:`d` increases, the :math:`L_2` distance
between opposite corners grows more slowly (:math:`d` vs
:math:`\sqrt{d}`).

Since the amount of noise added is proportional to the worst-case
distance (to hide individual effects), a mechanism that can calibrate
its noise according to the :math:`L_2` distance is very attractive. In
the context of differentially private marginal queries, as is common in
OpenDP Polars, the greatest improvements occur when an individual has
small influence on a large number of answers.

Bound Contributions Per Group
-----------------------------

In the Labour Force survey, one record is collected from each respondent
on a quarterly cadence. This means an individual has very little
influence on the data in any one quarter; in the worst case, they can
only ever contribute one record per quarter. With this knowledge the
amount of noise necessary to release time-series queries at a given
privacy loss under zCDP becomes much smaller.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> context_margin = dp.Context.compositor(
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     # tells OpenDP that individuals contribute...
            ...     privacy_unit=dp.unit_of(contributions=[
            ...         # ...at most 36 records overall...
            ...         dp.polars.Bound(by=[], per_group=36),
            ...         # ...and at most 1 record in each year-quarter.
            ...         dp.polars.Bound(by=["YEAR", "QUARTER"], per_group=1),
            ...     ]),
            ...     privacy_loss=dp.loss_of(rho=0.1, delta=1e-7),
            ...     split_evenly_over=5,
            ... )
            
            >>> query_num_responses = context_margin.query().group_by("YEAR", "QUARTER").agg(dp.len())
            >>> query_num_responses.summarize(alpha=0.05)
            shape: (1, 6)
            ┌────────┬──────────────┬──────────────────┬───────┬──────────┬───────────┐
            │ column ┆ aggregate    ┆ distribution     ┆ scale ┆ accuracy ┆ threshold │
            │ ---    ┆ ---          ┆ ---              ┆ ---   ┆ ---      ┆ ---       │
            │ str    ┆ str          ┆ str              ┆ f64   ┆ f64      ┆ u32       │
            ╞════════╪══════════════╪══════════════════╪═══════╪══════════╪═══════════╡
            │ len    ┆ Frame Length ┆ Integer Gaussian ┆ 30.0  ┆ 60.0     ┆ 184       │
            └────────┴──────────────┴──────────────────┴───────┴──────────┴───────────┘


Now contrast this to the same query, but when the library isn’t made
aware of this data descriptor.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> context = dp.Context.compositor(
            ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(rho=0.1, delta=1e-7),
            ...     split_evenly_over=5,
            ... )
            
            >>> query_num_responses = context.query().group_by("YEAR", "QUARTER").agg(dp.len())
            >>> query_num_responses.summarize(alpha=0.05)
            shape: (1, 6)
            ┌────────┬──────────────┬──────────────────┬───────┬──────────┬───────────┐
            │ column ┆ aggregate    ┆ distribution     ┆ scale ┆ accuracy ┆ threshold │
            │ ---    ┆ ---          ┆ ---              ┆ ---   ┆ ---      ┆ ---       │
            │ str    ┆ str          ┆ str              ┆ f64   ┆ f64      ┆ u32       │
            ╞════════╪══════════════╪══════════════════╪═══════╪══════════╪═══════════╡
            │ len    ┆ Frame Length ┆ Integer Gaussian ┆ 180.0 ┆ 354.0    ┆ 1133      │
            └────────┴──────────────┴──────────────────┴───────┴──────────┴───────────┘


The presence of the margin descriptor reduces the scale from 180 to 30,
a *six*-fold reduction in noise!

When the margin descriptor is present together with the bound of 36
contributions, then in the worst case an individual influences 36
different dimensions by one. The :math:`L_2` distance between two count
vectors that differ by one in :math:`36` positions is
:math:`\sqrt{36} = 6`.

Whereas when the margin descriptor is not present, then in the worst
case an individual makes 36 contributions to the same dimension. The
:math:`L_2` distance between two count vectors that differ by :math:`36`
in one position is :math:`\sqrt{36^2} = 36`.

This explains the factor of six reduction in the noise (:math:`36 / 6`).
For comparison, the Laplace mechanism will always result in a
sensitivity of :math:`36`, even in the presence of this domain
descriptor.

Try to be mindful of the structure of your data when preparing your
analysis, because settings where an individual’s contributions are
distributed over many different dimensions can be used to answer queries
that have much lower sensitivity, and therefore can be estimated with
less noise.
