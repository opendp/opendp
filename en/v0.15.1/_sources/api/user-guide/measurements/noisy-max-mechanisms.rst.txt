Report Noisy Max Mechanisms
===========================

The report noisy max mechanism takes as input a vector of scores, adds
noise to each score, and then releases the index of the greatest score.

The specific case of report noisy max with gumbel noise is equivalent to
the exponential mechanism with a finite support.

This notebook documents the variations on report noisy max mechanisms in
OpenDP:

- Distribution: exponential vs gumbel
- Monotonicity
- Objective: maximize vs. minimize
- Composition: max vs. top-k

--------------

Any constructors that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")


The goal of mechanisms in this notebook will be to release the argument
or index of the best score in the following vector:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> scores = [0.0, 2.0, 4.0, 6.0]


We’ll also define the sensitivity once, for all examples:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> sensitivity = 1.0


The score vectors on adjacent datasets differ by at most one in each
position.

Distribution: Exponential vs. Gumbel
------------------------------------

Based on the choice of privacy measure passed into ``report_noisy_max``,
the OpenDP Library chooses the corresponding distribution to maximize
utility:

+-------------------+------------------------------------+--------------+
| Privacy           | ``privacy_measure``                | distribution |
| Definition        |                                    |              |
+===================+====================================+==============+
| pure-DP           | ``dp.max_divergence()``            | exponential  |
+-------------------+------------------------------------+--------------+
| zCDP              | ``d                                | gumbel       |
|                   | p.zero_concentrated_divergence()`` |              |
+-------------------+------------------------------------+--------------+

In the case of pure-DP, many noise distributions have been considered.
Report noisy max was initially introduced to the differential privacy
literature with *laplace* noise, and the exponential mechanism is
equivalent to report noisy max with *gumbel* noise, but *exponential*
noise satisfies the same privacy guarantee as both with less error than
both. However, in the case of zCDP, using gumbel noise results in a
stronger zCDP guarantee than can be derived with exponential noise,
making gumbel noise competitive.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # call the constructor to produce the measurement
            >>> exponential_max = dp.m.make_noisy_max(
            ...     input_domain=dp.vector_domain(
            ...         dp.atom_domain(T=float, nan=False)
            ...     ),
            ...     input_metric=dp.linf_distance(T=float),
            ...     output_measure=dp.max_divergence(),
            ...     scale=2.0,
            ... )

            >>> print("noisy max:", exponential_max(scores))
            noisy max: ...

            >>> #                 sensitivity * 2 / scale
            >>> print("epsilon:", exponential_max.map(d_in=sensitivity))
            epsilon: 1.0

Both variations of the mechanism share very similar privacy maps between
the sensitivity and privacy parameters guarantees as the respective
additive noise mechanisms (laplace and gaussian).

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # call the constructor to produce the measurement `base_rnm_gumbel`
            >>> gumbel_max = dp.m.make_noisy_max(
            ...     input_domain=dp.vector_domain(
            ...         dp.atom_domain(T=float, nan=False)
            ...     ),
            ...     input_metric=dp.linf_distance(T=float),
            ...     output_measure=dp.zero_concentrated_divergence(),
            ...     scale=2.0,
            ... )

            >>> print("noisy max:", gumbel_max(scores))
            noisy max: ...

            >>> #             (sensitivity * 2 / scale)^2 / 8
            >>> print("rho:", gumbel_max.map(d_in=sensitivity))
            rho: 0.125

Take note that gumbel noise results in a much slower runtime on common
workloads.

Monotonicity
============

If all entries in the score vector may only differ in one direction,
then we can say that the scores are *monotonic*. When scores are
monotonic, the privacy loss is halved:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # when monotonic=True, the distance between score vectors
            >>> # that differ in different directions is defined to be infinity
            >>> input_metric = dp.linf_distance(T=float, monotonic=True)

            >>> # construct the measurement with a monotonic metric
            >>> monotonic_exponential_max = dp.m.make_noisy_max(
            ...     input_domain=dp.vector_domain(
            ...         dp.atom_domain(T=float, nan=False)
            ...     ),
            ...     input_metric=input_metric,
            ...     output_measure=dp.max_divergence(),
            ...     scale=2.0,
            ... )

            >>> # factor of 2 goes away in privacy map:
            >>> #                 sensitivity * 1 / scale
            >>> print(
            ...     "epsilon:",
            ...     monotonic_exponential_max.map(d_in=sensitivity),
            ... )
            epsilon: 0.5

Under these conditions, the privacy map now matches the laplace
mechanism. Under monotonicity, in zCDP, the noisy max mechanism differs
only by a factor of a fourth.

Objective: Maximize vs. Minimize
================================

If the mechanism should choose the smallest score, instead of the
largest, then negate the inputs:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # construct the report noisy min measurement
            >>> exponential_min = dp.m.make_noisy_max(
            ...     input_domain=dp.vector_domain(
            ...         dp.atom_domain(T=float, nan=False)
            ...     ),
            ...     input_metric=dp.linf_distance(T=float),
            ...     output_measure=dp.max_divergence(),
            ...     scale=2.0,
            ...     # negate input scores
            ...     negate=True,
            ... )

            >>> print("noisy min:", exponential_min(scores))
            noisy min: ...

The negation of scores does not affect the privacy loss.

Composition: Max vs. Top-k
==========================

Use the ``make_noisy_top_k`` constructor to select multiple indices
simultaneously:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> exponential_top_k = dp.m.make_noisy_top_k(
            ...     input_domain=dp.vector_domain(
            ...         dp.atom_domain(T=float, nan=False)
            ...     ),
            ...     input_metric=dp.linf_distance(T=float),
            ...     output_measure=dp.max_divergence(),
            ...     k=2,
            ...     scale=2.0,
            ... )

            >>> scores = [0.0, 1.0, 2.0, 3.0]
            >>> print("noisy top 2:", exponential_top_k(scores))
            noisy top 2: [...]

            >>> print("epsilon:", exponential_top_k.map(d_in=sensitivity))
            epsilon: 2.0

In the output, the ordering of indices is significant. That is, the
noisy score at the first index is greater than the noisy score at the
second index.

The privacy loss increases linearly in ``k``, by sequential composition.
Assuming the number of candidates in the score vector is ``d``, then the
time complexities of the algorithms are as follows:

================== ===================== =============================
Privacy Definition algorithm             time complexity
================== ===================== =============================
pure-DP            peel permute and flip :math:`\mathcal{O}(dk)`
zCDP               oneshot gumbel        :math:`\mathcal{O}(d \ln(k))`
================== ===================== =============================

The noisy max exponential mechanism is internally implemented via the
permute and flip algorithm, which runs in time :math:`\mathcal{O}(d)`.
To compute top-k, the permute and flip algorithm is then run k times,
where the previous selection is repeatedly “peeled” off of the score
vector.

In contrast, in the case of zCDP, gumbel noise is added to each score,
and a min-heap is used to find the top k in one shot.

Even though the oneshot gumbel has better time complexity, the permute
and flip algorithm has a much faster discrete implementation, so is
likely to still be faster for reasonable values of k.
