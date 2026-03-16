.. _private-selection:

Private Selection From Private Candidates
-----------------------------------------

:func:`~opendp.combinators.make_select_private_candidate` turns a measurement
that releases scored candidates into a new measurement that privately chooses a
single candidate.

The input measurement is expected to release a 2-tuple of ``(score,
candidate)``. The constructor then repeatedly invokes that measurement and
chooses a candidate according to one of two modes:

* If ``threshold`` is set, return the first candidate whose score is at least
  the threshold.
* If ``threshold`` is not set, sample a repetition count and return the best
  candidate among those repetitions.

This constructor is temporarily behind the ``private-selection-v2`` feature flag,
due to backwards-incompatible API changes.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib", "private-selection-v2")

    .. tab-item:: R
        :sync: r

        .. code:: r

            library(opendp)
            enable_features("contrib", "private-selection-v2")

Thresholded Selection
~~~~~~~~~~~~~~~~~~~~~

In the thresholded setting, the constructor returns the first private candidate
whose score exceeds the threshold. This is useful when the score has a natural
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
            >>> meas_threshold = dp.c.make_select_private_candidate(
            ...     meas_scored_candidate,
            ...     mean=100.0,
            ...     threshold=threshold,
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
            meas_threshold <- make_select_private_candidate(
              meas_scored_candidate,
              mean = 100.0,
              threshold = threshold
            )
            meas_threshold(d_in = 1L)

For pure-DP measurements, thresholded private selection has a privacy loss of
``2ε`` when the base candidate mechanism has privacy loss ``ε``.

Best-Of-K Selection
~~~~~~~~~~~~~~~~~~~

If ``threshold`` is omitted, the selector samples a repetition count and returns
the best candidate among those repetitions. 
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
            >>> meas_candidate = space >> dp.m.then_user_measurement(
            ...     dp.renyi_divergence(),
            ...     lambda x: (x, x),
            ...     lambda d_in: (lambda alpha: d_in * alpha / 2.0),
            ...     TO="(f64, ExtrinsicObject)",
            ... )
            >>> meas_best = dp.c.make_select_private_candidate(
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
                function = function(x) list(x, x),
                privacy_map = function(d_in) function(alpha) d_in * alpha / 2.0,
                TO = "(f64, ExtrinsicObject)"
              )
            meas_best <- make_select_private_candidate(
              meas_candidate,
              mean = 2.5,
              distribution = "negative_binomial",
              eta = 0.5
            )
            curve <- meas_best(d_in = 1.0)
            curve(4.0)

The Poisson distribution only works with Renyi-DP.
