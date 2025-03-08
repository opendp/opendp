Compositors
===========

Any constructors that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")


Define a few queries you might want to run up-front:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # define the dataset space and how distances are measured
            >>> input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()

            >>> count_meas = input_space >> dp.t.then_count() >> dp.m.then_laplace(scale=1.0)
            >>> sum_meas = (
            ...     input_space
            ...     >> dp.t.then_clamp((0, 10))
            ...     >> dp.t.then_sum()
            ...     >> dp.m.then_laplace(scale=5.0)
            ... )


Notice that both of these measurements share the same input domain,
input metric, and output measure:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> print("count:", count_meas)
            count: Measurement(
                input_domain   = VectorDomain(AtomDomain(T=i32)),
                input_metric   = SymmetricDistance(),
                output_measure = MaxDivergence)

            >>> print("sum:", sum_meas)
            sum: Measurement(
                input_domain   = VectorDomain(AtomDomain(T=i32)),
                input_metric   = SymmetricDistance(),
                output_measure = MaxDivergence)

This is important, because compositors require these three supporting
elements to match for all queries.

Basic Composition
-----------------

The basic composition compositor is non-interactive: it takes a
collection of queries to execute on the dataset all-at-once. When the
data is passed in, all queries are evaluated together, in a single
batch.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> mean_fraction_meas = dp.c.make_basic_composition([sum_meas, count_meas])

            >>> int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            >>> dp_sum, dp_count = mean_fraction_meas(int_dataset)
            >>> print("dp sum:", dp_sum)
            dp sum: ...
            >>> print("dp count:", dp_count)
            dp count: ...

The privacy map simply sums the constituent output distances.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> mean_fraction_meas.map(1)
            3.0

.. _sequential-composition:

Sequential Composition
----------------------

Sequential composition relaxes the basic compositor, allowing for
queries to be submitted interactively. That is, you can make submit a
query, view the output, and then submit another query that uses the
information gained from the prior release. However, this API still
requires `sequentiality`, which we’ll discuss in more detail below.

The API for interactive compositors is more verbose than in the
non-interactive case because you must explicitly pass the input domain,
input metric, and output measure, as well as an upper bound on input
distances (``d_in``), and the privacy consumption allowed for each query
(``d_mids``).

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> sc_meas = dp.c.make_sequential_composition(
            ...     input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            ...     input_metric=dp.symmetric_distance(),
            ...     output_measure=dp.max_divergence(),
            ...     d_in=1,
            ...     d_mids=[2., 1.]
            ... )


Given this information, we know the privacy consumption of the entire
composition:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> sc_meas.map(1)
            3.0

When the sequential composition measurement (``sc_meas``) is invoked, it
returns a *queryable*.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            >>> sc_queryable = sc_meas(int_dataset)


A queryable is like a state machine: it takes an input query, updates
its internal state, and returns an answer. For sequential compositors,
the input query is a measurement, the internal state is the dataset and
privacy consumption, and the answer is the differentially private
release from the measurement.

Similarly as before, we now interactively submit queries to estimate the
sum and count:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> print("dp sum:", sc_queryable(sum_meas))
            dp sum: ...
            >>> print("dp count:", sc_queryable(count_meas))
            dp count: ...

The compositor is *sequential* in the sense that multiple queries are sequentially released on the same dataset.

.. note::

    The sequential compositor API makes another distinction
    between adaptive composition and concurrent composition,
    which varies based on the choice of privacy measure.

    Adaptive composition is subject to the limitation that
    only one queryable is active at any point in time.
    To satisfy adaptive composition, the compositor locks, or freezes,
    any queryable it has previously spawned when a new query arrives.
    This is because the postprocessing argument doesn't necessarily
    hold when the analyst may still interact with earlier queryables.

    Concurrent composition lifts this limitation for measures of privacy
    where we have been able to prove that postprocessing still holds.
    In OpenDP, all privacy measures support concurrent composition,
    except for approximate zCDP and approximate Renyi-DP.


Chaining
--------

Since all compositors are just “plain-old-measurements” they also
support chaining.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> str_space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
            >>> str_sc_meas = str_space >> dp.t.then_cast_default(int) >> sc_meas

            >>> str_sc_queryable = str_sc_meas(["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"])
            >>> str_sc_queryable(sum_meas), str_sc_queryable(count_meas)
            (..., ...)

``str_sc_meas`` is invoked with a string dataset, but returns a
queryable that takes queries over integer datasets. Chaining compositors
can be used to avoid repeating the same transformations for each query.

Keep in mind that the ``d_in`` on the interactive compositor must match
the output distance from the previous transformation:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> max_contributions = 1
            >>> sum_trans = input_space >> dp.t.then_clamp((0, 10)) >> dp.t.then_sum()
            >>> sc_meas = sum_trans >> dp.c.make_sequential_composition(
            ...     input_domain=sum_trans.output_domain,
            ...     input_metric=sum_trans.output_metric,
            ...     output_measure=dp.max_divergence(),
            ...     d_in=sum_trans.map(max_contributions),
            ...     d_mids=[2., 1.]
            ... )


In this code snip, we used the supporting elements and map from the
transformation to fill in arguments to the sequential compositor
constructor, and to derive a suitable ``d_in`` for the compositor, based
on a known ``d_in`` for the sum transformation.

Nesting
-------

Just like in chaining, since all compositors are
“plain-old-measurements” they can also be used as arguments to
interactive compositors. In this example, we nest a zCDP sequential
compositor inside an approximate-DP sequential compositor.

We first make the approximate-DP sequential compositor, accepting two
queries. The first query must be $(2, 10^{-6})$-DP, and the
second (1, 0)-DP.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> sc_meas = dp.c.make_sequential_composition(
            ...     input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            ...     input_metric=dp.symmetric_distance(),
            ...     output_measure=dp.approximate(dp.max_divergence()),
            ...     d_in=1,
            ...     d_mids=[(2., 1e-6), (1., 0.)]
            ... )
            >>> adp_sc_queryable = sc_meas(int_dataset)


The first query to the approximate-DP sequential compositor must be an
approximate-DP measurement that satisfies $(2, 10^{-6})$-DP.
We will now use the library to find a set of :math:`\rho` parameters
that will satisfy this level of privacy, under a given set of weights.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # find ρ_1, ρ_2 such that ρ_1 + ρ_2 = ρ <= (2, 1e-6),
            >>> #    and ρ_1 is 5 times larger than ρ_2
            >>> weights = [5., 1.]


            >>> def scale_weights(scale, weights):
            ...     return [scale * w for w in weights]

            >>> def make_zcdp_sc(scale):
            ...     return dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(dp.c.make_sequential_composition(
            ...         input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            ...         input_metric=dp.symmetric_distance(),
            ...         output_measure=dp.zero_concentrated_divergence(),
            ...         d_in=1,
            ...         d_mids=scale_weights(scale, weights)
            ...     )), delta=1e-6)

            >>> # find a scale parameter for the d_mids that makes the overall compositor satisfy (2., 1e-6)-approxDP
            >>> zcdp_compositor_scale = dp.binary_search_param(make_zcdp_sc, d_in=1, d_out=(2., 1e-6), T=float)

            >>> # construct a zCDP sequential compositor that satisfies (2., 1e-6)-approxDP
            >>> zcdp_compositor = make_zcdp_sc(zcdp_compositor_scale)

            >>> # query the root approx-DP compositor queryable to get a child zCDP queryable
            >>> zcdp_sc_queryable = adp_sc_queryable(zcdp_compositor)

            >>> rho_1, rho_2 = scale_weights(zcdp_compositor_scale, weights)
            >>> rho_1, rho_2
            (0.0734..., 0.0146...)

Now that we’ve determined :math:`\rho_1` and :math:`\rho_2`, make a
release:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> def make_zcdp_sum_query(scale):
            ...     return (
            ...         input_space
            ...         >> dp.t.then_clamp((0, 10))
            ...         >> dp.t.then_sum()
            ...         >> dp.m.then_gaussian(scale)
            ...     )


            >>> dg_scale = dp.binary_search_param(make_zcdp_sum_query, d_in=1, d_out=rho_1)
            >>> print('zcdp:', zcdp_sc_queryable(make_zcdp_sum_query(dg_scale)))
            zcdp: ...

At this point, we can either submit a second query to the root approx-DP
compositor queryable (``adp_sc_queryable``), or to the child zCDP compositor
queryable (``zcdp_sc_queryable``).

However, if you submit a query to ``adp_sc_queryable`` first, then to preserve
sequentiality, ``zcdp_sc_queryable`` becomes locked.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # convert the pure-DP count measurement to a approx-DP count measurement (where δ=0.)
            >>> adp_count_meas = dp.c.make_approximate(count_meas)

            >>> # submit the count measurement to the root approx-DP compositor queryable
            >>> print('adp:', adp_sc_queryable(adp_count_meas))
            adp: ...

We’ve now exhausted the privacy budget of the root approx-DP queryable,
and locked the zCDP queryable, so all queryables will refuse to answer
any more queries.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> zcdp_sc_queryable(make_zcdp_sum_query(dg_scale))
            Traceback (most recent call last):
            ...
            opendp.mod.OpenDPException:
              FailedFunction("insufficient budget for query: 0.0734... > 0.0146...")

            >>> adp_sc_queryable(adp_count_meas)
            Traceback (most recent call last):
            ...
            opendp.mod.OpenDPException:
              FailedFunction("out of queries")

In conclusion, OpenDP provides several compositors with different
trade-offs, and interactive compositors (like sequential composition)
provide a protective, differentially private interface for accessing any
dataset stored within the queryable.
