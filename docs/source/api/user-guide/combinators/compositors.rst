Compositors
===========

Any constructors that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. code:: ipython3

    import opendp.prelude as dp
    dp.enable_features("contrib")

Define a few queries you might want to run up-front:

.. code:: ipython3

    # define the dataset space and how distances are measured
    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    
    count_meas = input_space >> dp.t.then_count() >> dp.m.then_laplace(scale=1.0)
    sum_meas = (
        input_space
        >> dp.t.then_clamp((0, 10))
        >> dp.t.then_sum()
        >> dp.m.then_laplace(scale=5.0)
    )

Notice that both of these measurements share the same input domain,
input metric, and output measure:

.. code:: ipython3

    print("count:", count_meas)
    print("sum:", sum_meas)


.. parsed-literal::

    count: Measurement(
        input_domain   = VectorDomain(AtomDomain(T=i32)), 
        input_metric   = SymmetricDistance(), 
        output_measure = MaxDivergence(f64)
    )
    sum: Measurement(
        input_domain   = VectorDomain(AtomDomain(T=i32)), 
        input_metric   = SymmetricDistance(), 
        output_measure = MaxDivergence(f64)
    )


This is important, because compositors require these three supporting
elements to match for all queries.

Basic Composition
-----------------

The basic composition compositor is non-interactive: it takes a
collection of queries to execute on the dataset all-at-once. When the
data is passed in, all queries are evaluated together, in a single
batch.

.. code:: ipython3

    mean_fraction_meas = dp.c.make_basic_composition([sum_meas, count_meas])
    
    int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    dp_sum, dp_count = mean_fraction_meas(int_dataset)
    print("dp sum:", dp_sum)
    print("dp count:", dp_count)


.. parsed-literal::

    dp sum: 54
    dp count: 12


The privacy map simply sums the constituent output distances.

.. code:: ipython3

    mean_fraction_meas.map(1)




.. parsed-literal::

    3.0



Sequential Composition
----------------------

Sequential composition relaxes the basic compositor, allowing for
queries to be submitted interactively. That is, you can make submit a
query, view the output, and then submit another query that uses the
information gained from the prior release. However, this API still
requires ``sequentiality``, which we’ll discuss in more detail later.

The API for interactive compositors is more verbose than in the
non-interactive case because you must explicitly pass the input domain,
input metric, and output measure, as well as an upper bound on input
distances (``d_in``), and the privacy consumption allowed for each query
(``d_mids``).

.. code:: ipython3

    sc_meas = dp.c.make_sequential_composition(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(T=float),
        d_in=1,
        d_mids=[2., 1.]
    )

Given this information, we know the privacy consumption of the entire
composition:

.. code:: ipython3

    sc_meas.map(1)




.. parsed-literal::

    3.0



When the sequential composition measurement (``sc_meas``) is invoked, it
returns a *queryable*.

.. code:: ipython3

    int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    sc_qbl = sc_meas(int_dataset)

A queryable is like a state machine: it takes an input query, updates
its internal state, and returns an answer. For sequential compositors,
the input query is a measurement, the internal state is the dataset and
privacy consumption, and the answer is the differentially private
release from the measurement.

Similarly as before, we now interactively submit queries to estimate the
sum and count:

.. code:: ipython3

    print("dp sum:", sc_qbl(sum_meas))
    print("dp count:", sc_qbl(count_meas))


.. parsed-literal::

    dp sum: 57
    dp count: 10


Now, why is this compositor named *sequential*? In order to prove that
the privacy properties of this compositor hold in the interactive
setting, the compositor must lock, or freeze, any queryable it has
previously spawned when a new query arrives.

This is an artifact of how non-interactive composition results have been
extended to work in the interactive setting. Namely, that the second
query can be viewed as a postprocessing of the first query.
Unfortunately, this postprocessing argument doesn’t necessarily hold
when the analyst may still interact with the first queryable. This is
the subject of a further line of research on concurrent compositors,
which we hope to make available in the next library release.

An example of this constraint is demonstrated in the “Nesting” section
below.

Chaining
--------

Since all compositors are just “plain-old-measurements” they also
support chaining.

.. code:: ipython3

    str_space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    str_sc_meas = str_space >> dp.t.then_cast_default(int) >> sc_meas
    
    str_sc_qbl = str_sc_meas(["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"])
    str_sc_qbl(sum_meas), str_sc_qbl(count_meas)




.. parsed-literal::

    (48, 8)



``str_sc_meas`` is invoked with a string dataset, but returns a
queryable that takes queries over integer datasets. Chaining compositors
can be used to avoid repeating the same transformations for each query.

Keep in mind that the ``d_in`` on the interactive compositor must match
the output distance from the previous transformation:

.. code:: ipython3

    max_contributions = 1
    sum_trans = input_space >> dp.t.then_clamp((0, 10)) >> dp.t.then_sum()
    sc_meas = sum_trans >> dp.c.make_sequential_composition(
        input_domain=sum_trans.output_domain,
        input_metric=sum_trans.output_metric,
        output_measure=dp.max_divergence(T=float),
        d_in=sum_trans.map(max_contributions),
        d_mids=[2., 1.]
    )

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
queries. The first query must be :math:`(2 ε, 10^{-6} δ)`-DP, and the
second :math:`(1 ε, 0 δ)`-DP.

.. code:: ipython3

    sc_meas = dp.c.make_sequential_composition(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.fixed_smoothed_max_divergence(T=float),
        d_in=1,
        d_mids=[(2., 1e-6), (1., 0.)]
    )
    adp_sc_qbl = sc_meas(int_dataset)

The first query to the approximate-DP sequential compositor must be an
approximate-DP measurement that satisfies :math:`(2 ε, 10^{-6} δ)`-DP.
We will now use the library to find a set of :math:`\rho` parameters
that will satisfy this level of privacy, under a given set of weights.

.. code:: ipython3

    # find ρ_1, ρ_2 such that ρ_1 + ρ_2 = ρ <= (2ε, 1e-6δ), 
    #    and ρ_1 is 5 times larger than ρ_2
    weights = [5., 1.]
    
    
    def scale_weights(scale, weights):
        return [scale * w for w in weights]
    
    def make_zcdp_sc(scale):
        return dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(dp.c.make_sequential_composition(
            input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            input_metric=dp.symmetric_distance(),
            output_measure=dp.zero_concentrated_divergence(T=float),
            d_in=1,
            d_mids=scale_weights(scale, weights)
        )), delta=1e-6)
    
    # find a scale parameter for the d_mids that makes the overall compositor satisfy (2., 1e-6)-approxDP
    zcdp_compositor_scale = dp.binary_search_param(make_zcdp_sc, d_in=1, d_out=(2., 1e-6), T=float)
    
    # construct a zCDP sequential compositor that satisfies (2., 1e-6)-approxDP
    zcdp_compositor = make_zcdp_sc(zcdp_compositor_scale)
    
    # query the root approx-DP compositor queryable to get a child zCDP queryable
    zcdp_sc_qbl = adp_sc_qbl(zcdp_compositor)
    
    rho_1, rho_2 = scale_weights(zcdp_compositor_scale, weights)
    rho_1, rho_2




.. parsed-literal::

    (0.07346057364995517, 0.014692114729991036)



Now that we’ve determined :math:`\rho_1` and :math:`\rho_2`, make a
release:

.. code:: ipython3

    def make_zcdp_sum_query(scale):
        return (
            input_space
            >> dp.t.then_clamp((0, 10))
            >> dp.t.then_sum()
            >> dp.m.then_gaussian(scale)
        )
    
    
    dg_scale = dp.binary_search_param(make_zcdp_sum_query, d_in=1, d_out=rho_1)
    zcdp_sc_qbl(make_zcdp_sum_query(dg_scale))




.. parsed-literal::

    7



At this point, we can either submit a second query to the root approx-DP
compositor queryable (``adp_sc_qbl``), or to the child zCDP compositor
queryable (``zcdp_sc_qbl``).

However, if you submit a query to ``adp_sc_qbl`` first, then to preserve
sequentiality, ``zcdp_sc_qbl`` becomes locked.

.. code:: ipython3

    # convert the pure-DP count measurement to a approx-DP count measurement (where δ=0.)
    adp_count_meas = dp.c.make_pureDP_to_fixed_approxDP(count_meas)
    
    # submit the count measurement to the root approx-DP compositor queryable
    adp_sc_qbl(adp_count_meas)




.. parsed-literal::

    10



We’ve now exhausted the privacy budget of the root approx-DP queryable,
and locked the zCDP queryable, so all queryables will refuse to answer
any more queries.

.. code:: ipython3

    try:
        # submit the count measurement to the child zCDP queryable
        zcdp_sc_qbl(make_zcdp_sum_query(dg_scale))
    except dp.OpenDPException as e:
        print(e)


.. parsed-literal::

    Continued Rust stack trace:
        opendp_core__queryable_query_type
          at /Users/michael/openDP/openDP/rust/src/core/ffi.rs:736:30
        opendp::interactive::Queryable<Q,A>::eval_internal
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:34:15
        opendp::interactive::Queryable<Q,A>::eval_query
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:51:16
        <opendp::interactive::Queryable<Q,A> as opendp::interactive::FromPolyQueryable>::from_poly::{{closure}}
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:239:47
        opendp::interactive::Queryable<Q,A>::eval_query
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:51:16
        opendp::interactive::WrapFn::new_pre_hook::{{closure}}::{{closure}}
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:134:21
        opendp::combinators::composition::sequential::make_sequential_composition::{{closure}}::{{closure}}::{{closure}}
          at /Users/michael/openDP/openDP/rust/src/combinators/composition/sequential/mod.rs:105:29
        opendp::interactive::Queryable<Q,A>::eval_internal
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:34:15
        opendp::interactive::Queryable<Q,A>::eval_query
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:51:16
        opendp::combinators::composition::sequential::make_sequential_composition::{{closure}}::{{closure}}
          at /Users/michael/openDP/openDP/rust/src/combinators/composition/sequential/mod.rs:124:40
      FailedFunction("sequential compositor has received a new query")


.. code:: ipython3

    try:
        # submit the count measurement to the child zCDP queryable
        adp_sc_qbl(adp_count_meas)
    except dp.OpenDPException as e:
        print(e)


.. parsed-literal::

    Continued Rust stack trace:
        opendp_core__queryable_eval
          at /Users/michael/openDP/openDP/rust/src/core/ffi.rs:718:5
        opendp::interactive::Queryable<Q,A>::eval
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:16:15
        opendp::interactive::Queryable<Q,A>::eval_query
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:51:16
        opendp::ffi::any::<impl opendp::core::Measurement<opendp::ffi::any::AnyDomain,opendp::interactive::Queryable<Q,A>,opendp::ffi::any::AnyMetric,opendp::ffi::any::AnyMeasure>>::into_any_Q::{{closure}}::{{closure}}
          at /Users/michael/openDP/openDP/rust/src/ffi/any.rs:239:51
        opendp::interactive::Queryable<Q,A>::eval
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:16:15
        opendp::interactive::Queryable<Q,A>::eval_query
          at /Users/michael/openDP/openDP/rust/src/interactive/mod.rs:51:16
        opendp::combinators::composition::sequential::make_sequential_composition::{{closure}}::{{closure}}
          at /Users/michael/openDP/openDP/rust/src/combinators/composition/sequential/mod.rs:89:37
        opendp::combinators::composition::sequential::make_sequential_composition::{{closure}}::{{closure}}::{{closure}}
          at /Users/michael/openDP/openDP/rust/src/combinators/composition/sequential/mod.rs:90:44
      FailedFunction("out of queries")


In conclusion, OpenDP provides several compositors with different
trade-offs, and interactive compositors (like sequential composition)
provide a protective, differentially private interface for accessing any
dataset stored within the queryable.
