.. _private-selection:

Private Selection From Private Candidates
-----------------------------------------

:func:`~opendp.combinators.make_select_at_least` and
:func:`~opendp.combinators.make_select_best_candidate` select a candidate
from a measurement that releases pairs of ``(score, candidate)``.

The thresholded constructor returns the first candidate whose score meets a
public threshold. The best-candidate constructor selects the highest-scoring
candidate among a random number of repetitions. In both APIs ``mean`` is
the second argument and bounds the expected number of repetitions.

The old ``make_select_private_candidate(measurement, stop_probability, threshold)``
remains available for compatibility, but is deprecated.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")

    .. tab-item:: R
        :sync: r

        .. code:: r

            library(opendp)
            enable_features("contrib")

Thresholded Selection
~~~~~~~~~~~~~~~~~~~~~

In the thresholded setting, the constructor returns the first private candidate
whose score is at least the minimum score. This is useful when the score has a natural
accept/reject interpretation.

The following example constructs a private candidate mechanism whose score is a
private count and whose candidate is a private sum. The thresholded selector
returns the first candidate whose private count is at least ``23``.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> bounds = (0.0, 100.0)
            >>> range_ = max(abs(bounds[0]), bounds[1])
            >>> epsilon = 1.0
            >>> threshold = 23.0
            >>> input_space = (
            ...     dp.vector_domain(dp.atom_domain(T=float)),
            ...     dp.symmetric_distance(),
            ... )
            >>> meas_count = (
            ...     input_space
            ...     >> dp.t.then_count()
            ...     >> dp.m.then_laplace(scale=2.0 / epsilon)
            ... )
            >>> meas_sum = (
            ...     input_space
            ...     >> dp.t.then_impute_constant(0.0)
            ...     >> dp.t.then_clamp(bounds)
            ...     >> dp.t.then_sum()
            ...     >> dp.m.then_laplace(scale=2.0 * range_ / epsilon)
            ... )
            >>> meas_scored_candidate = dp.c.make_composition(
            ...     [meas_count, meas_sum]
            ... )
            >>> meas_threshold = dp.c.make_select_at_least(
            ...     meas_scored_candidate,
            ...     mean=100.0,
            ...     min_score=threshold,
            ... )
            >>> meas_threshold.map(1)
            2.000000009313227

    .. tab-item:: R
        :sync: r

        .. code:: r

            bounds <- c(0.0, 100.0)
            range_ <- max(abs(bounds[1]), bounds[2])
            epsilon <- 1.0
            threshold <- 23.0
            input_space <- c(
              vector_domain(atom_domain(.T = "f64")),
              symmetric_distance()
            )
            meas_count <- input_space |>
              then_count() |>
              then_laplace(scale = 2.0 / epsilon)
            meas_sum <- input_space |>
              then_impute_constant(0.0) |>
              then_clamp(bounds) |>
              then_sum() |>
              then_laplace(scale = 2.0 * range_ / epsilon)
            meas_scored_candidate <- make_composition(
              list(meas_count, meas_sum)
            )
            meas_threshold <- make_select_at_least(
              meas_scored_candidate,
              mean = 100.0,
              min_score = threshold
            )
            meas_threshold(d_in = 1L)

For pure-DP measurements, thresholded private selection has a privacy loss of
``2ε`` when the base candidate mechanism has privacy loss ``ε``.

Best-Of-K Selection
~~~~~~~~~~~~~~~~~~~

The best-candidate selector samples a repetition count and returns the
highest-scoring candidate among those repetitions.
The ``mean`` parameter controls the expected number of repetitions,
and ``distribution`` determines the sampling distribution for the number of repetitions.

The default ``distribution="geometric"`` recovers the basic geometric-style
behavior. Additional repetition laws are available for Renyi-DP measurements.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> space = (
            ...     dp.atom_domain(T=float),
            ...     dp.absolute_distance(T=float),
            ... )
            >>> import numpy as np
            >>> meas_candidate = space >> dp.m.then_user_measurement(
            ...     dp.renyi_divergence(),
            ...     lambda x: (float(np.random.normal(loc=x, scale=1.0)), "candidate"),
            ...     lambda d_in: (lambda alpha: d_in**2 * alpha / 2.0),
            ...     TO="(f64, ExtrinsicObject)",
            ... )
            >>> meas_best = dp.c.make_select_best_candidate(
            ...     meas_candidate,
            ...     mean=2.5,
            ...     distribution="negative_binomial",
            ...     eta=0.5,
            ... )
            >>> curve = meas_best.map(1.0)
            >>> curve(4.0) > 0.0
            True

    .. tab-item:: R
        :sync: r

        .. code:: r

            input_space <- c(atom_domain(.T = "f64"), absolute_distance(.T = "f64"))
            meas_candidate <- input_space |>
              then_user_measurement(
                output_measure = renyi_divergence(),
                function = function(x) list(rnorm(1, mean = x, sd = 1), "candidate"),
                privacy_map = function(d_in) function(alpha) d_in^2 * alpha / 2.0,
                TO = "(f64, ExtrinsicObject)"
              )
            meas_best <- make_select_best_candidate(
              meas_candidate,
              mean = 2.5,
              distribution = "negative_binomial",
              eta = 0.5
            )
            curve <- meas_best(d_in = 1.0)
            curve(4.0)

The Poisson distribution only works with Renyi-DP. Its privacy map uses
a generic Poisson-composition bound at all orders, taking the tighter
Papernot–Steinke bound where the latter's conditions hold. A bound may
still be infinite at some orders.

For geometric, logarithmic, and negative-binomial repetitions, ``mean=1``
selects exactly one private candidate, with the base measurement's privacy
map unchanged. The Poisson distribution retains its usual random count at
``mean=1``.

The Gaussian user-measurement examples illustrate the mathematical privacy
map; production implementations should use a cryptographically secure sampler.
