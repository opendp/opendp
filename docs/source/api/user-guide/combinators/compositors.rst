.. _compositors:

Compositors
===========

Any functions that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")
    

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: init
            :end-before: /init

            

Define a few queries you might want to run up-front:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # define the dataset space and how distances are measured
            >>> input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
            
            >>> meas_count = input_space >> dp.t.then_count() >> dp.m.then_laplace(scale=1.0)
            >>> meas_sum = (
            ...     input_space
            ...     >> dp.t.then_clamp((0, 10))
            ...     >> dp.t.then_sum()
            ...     >> dp.m.then_laplace(scale=5.0)
            ... )

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: up-front
            :end-before: /up-front
            

Notice that both of these measurements share the same input domain,
input metric, and output measure:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> print("count:", meas_count)
            count: Measurement(
                input_domain   = VectorDomain(AtomDomain(T=i32)),
                input_metric   = SymmetricDistance(),
                output_measure = MaxDivergence)

            >>> print("sum:", meas_sum)
            sum: Measurement(
                input_domain   = VectorDomain(AtomDomain(T=i32)),
                input_metric   = SymmetricDistance(),
                output_measure = MaxDivergence)

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: print-up-front
            :end-before: /print-up-front

This is important, because compositors require these three supporting
elements to match for all queries.

(Non-Adaptive) Composition
--------------------------

The non-adaptive compositor takes a collection of queries to execute on the dataset simultaneously. 
When the data is passed in, all queries are evaluated together, in a single batch.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> meas_mean_fraction = dp.c.make_composition([meas_sum, meas_count])
            
            >>> int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            >>> dp_sum, dp_count = meas_mean_fraction(int_dataset)
            >>> print("dp sum:", dp_sum)
            dp sum: ...
            >>> print("dp count:", dp_count)
            dp count: ...

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: non-adaptive-composition-init
            :end-before: /non-adaptive-composition-init

The privacy map sums the constituent output distances.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> meas_mean_fraction.map(1)
            3.0

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: non-adaptive-composition-map
            :end-before: /non-adaptive-composition-map

.. _adaptive-composition:

Adaptive Composition
--------------------

Adaptive composition allows for queries to be submitted interactively. 
That is, you can make submit a query, view the output, 
and then submit another query that uses the information gained from the prior release. 

The API for adaptive compositors is more verbose than in the
non-adaptive case because you must explicitly pass the input domain,
input metric, and output measure, as well as an upper bound on input
distances (``d_in``), and the privacy consumption allowed for each query
(``d_mids``).

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> meas_adaptive_comp = dp.c.make_adaptive_composition(
            ...     input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            ...     input_metric=dp.symmetric_distance(),
            ...     output_measure=dp.max_divergence(),
            ...     d_in=1,
            ...     d_mids=[2., 1.]
            ... )

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: med-adaptive-composition-init
            :end-before: /med-adaptive-composition-init


Given this information, we know the privacy consumption of the entire
composition:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> meas_adaptive_comp.map(1)
            3.0

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: med-adaptive-composition-map
            :end-before: /med-adaptive-composition-map

When the adaptive composition measurement (``meas_adaptive_comp``) is invoked, it
returns a *queryable*.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            >>> qbl_adaptive_comp = meas_adaptive_comp(int_dataset)

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: med-adaptive-composition-invoke
            :end-before: /med-adaptive-composition-invoke

A queryable is like a state machine: it takes an input query, updates
its internal state, and returns an answer. For adaptive composition,
the input query is a measurement, the internal state is the dataset and
privacy consumption, and the answer is the differentially private
release from the measurement.

Similarly as before, we now interactively submit queries to estimate the
sum and count:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> print("dp sum:", qbl_adaptive_comp(meas_sum))
            dp sum: ...
            >>> print("dp count:", qbl_adaptive_comp(meas_count))
            dp count: ...

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: med-adaptive-composition-query
            :end-before: /med-adaptive-composition-query

.. note::

    The adaptive composition API has another internal distinction 
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


.. _fully-adaptive-composition:

Fully Adaptive Composition
--------------------------

Where adaptive composition allows for queries to be chosen adaptively,
*fully* adaptive composition also allows for the *privacy loss* of queries to be chosen adaptively.
The API for fully adaptive composition matches that of adaptive composition,
but drops the ``d_mids`` argument, as these will be chosen as you go.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> odom_fully_adaptive_comp = dp.c.make_fully_adaptive_composition(
            ...     input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            ...     input_metric=dp.symmetric_distance(),
            ...     output_measure=dp.max_divergence(),
            ... )

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: fully-adaptive-composition
            :end-before: /fully-adaptive-composition
            

When the adaptive composition odometer (``odom_fully_adaptive_comp``) is invoked, 
it returns an *odometer queryable*.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            >>> qbl_fully_adaptive_comp = odom_fully_adaptive_comp(int_dataset)
    
    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: fully-adaptive-composition-invoke
            :end-before: /fully-adaptive-composition-invoke

You can check the privacy loss over all queries submitted to the queryable at any time.
Since no queries have been submitted yet, the privacy loss is 0.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> qbl_fully_adaptive_comp.privacy_loss(1)
            0.0

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: fully-adaptive-composition-loss1
            :end-before: /fully-adaptive-composition-loss1

Similarly as before, we now interactively submit queries to estimate the
sum and count:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> print("dp sum:", qbl_fully_adaptive_comp(meas_sum))
            dp sum: ...
            >>> print("dp count:", qbl_fully_adaptive_comp(meas_count))
            dp count: ...
    
    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: fully-adaptive-composition-eval1
            :end-before: /fully-adaptive-composition-eval1

Now that we have submitted two queries, we can see that the privacy loss has increased commensurately:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> qbl_fully_adaptive_comp.privacy_loss(1)
            3.0

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: fully-adaptive-composition-loss2
            :end-before: /fully-adaptive-composition-loss2

Privacy Filter
--------------
You can convert any odometer into a measurement by setting an upper bound on the privacy loss.
The following example converts the fully adaptive composition odometer into a privacy filter
that rejects any query that would cause the privacy loss to exceed 2.0:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> meas_fully_adaptive_comp = dp.c.make_privacy_filter(
            ...     odom_fully_adaptive_comp,
            ...     d_in=1,
            ...     d_out=2.0,
            ... )

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: privacy-filter
            :end-before: /privacy-filter

Privacy filters are measurements, meaning that they can be passed into :func:`make_composition <opendp.combinators.make_composition>`, 
adaptive composition queryables, or into other combinators.
However, they have the added benefit of not needing to specify privacy-loss parameters ahead-of-time.
When the privacy filter (``meas_fully_adaptive_comp``) is invoked, 
it still returns an *odometer queryable*, but this time the queryable will limit the overall privacy loss.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            >>> qbl_fully_adaptive_comp = meas_fully_adaptive_comp(int_dataset)

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: privacy-filter-invoke
            :end-before: /privacy-filter-invoke

Similarly as before, we now interactively submit queries to estimate the
sum and count:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> print("dp count:", qbl_fully_adaptive_comp(meas_count))
            dp count: ...
            >>> print("dp count:", qbl_fully_adaptive_comp(meas_count))
            dp count: ...

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: privacy-filter-eval1
            :end-before: /privacy-filter-eval1

Now that we have submitted two queries, we can see that the privacy loss has increased commensurately:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> qbl_fully_adaptive_comp.privacy_loss(1)
            2.0

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: privacy-filter-loss1
            :end-before: /privacy-filter-loss1

Since the privacy loss is capped at 2.0, any more queries will be rejected:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> print("dp count:", qbl_fully_adaptive_comp(meas_count))
            Traceback (most recent call last):
            ...
            opendp.mod.OpenDPException: 
              FailedFunction("filter is now exhausted: pending privacy loss (3.0) would exceed privacy budget (2.0)")

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: privacy-filter-eval2
            :end-before: /privacy-filter-eval2

Chaining
--------

Since non-adaptive compositors, adaptive compositors, and privacy filters are just "plain-old-measurements," 
they also support chaining.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> str_space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
            >>> meas_adaptive_comp_str = str_space >> dp.t.then_cast_default(int) >> meas_adaptive_comp
            
            >>> qbl_adaptive_comp_str = meas_adaptive_comp_str(["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"])
            >>> qbl_adaptive_comp_str(meas_sum), qbl_adaptive_comp_str(meas_count)
            (..., ...)

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: measurement-chaining1
            :end-before: /measurement-chaining1

``meas_adaptive_comp_str`` is invoked with a string dataset, but returns a
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
            >>> meas_adaptive_comp = sum_trans >> dp.c.then_adaptive_composition(
            ...     output_measure=dp.max_divergence(),
            ...     d_in=sum_trans.map(max_contributions),
            ...     d_mids=[2., 1.]
            ... )
    
    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: measurement-chaining2
            :end-before: /measurement-chaining2

In this code snip, we used the supporting elements and map from the
transformation to fill in arguments to the adaptive compositor
constructor, and to derive a suitable ``d_in`` for the compositor, based
on a known ``d_in`` for the sum transformation.

Nesting
-------

Just like in chaining, since non-adaptive compositors, adaptive compositors, and privacy filters are
"plain-old-measurements" they can also be used as arguments to
interactive compositors and other combinators. In this example, we nest a zCDP adaptive
compositor inside an approximate-DP adaptive compositor.

We first make the approximate-DP adaptive compositor, accepting two
queries. The first query must be $(2, 10^{-6})$-DP, and the
second (1, 0)-DP.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> meas_adaptive_comp = dp.c.make_adaptive_composition(
            ...     input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            ...     input_metric=dp.symmetric_distance(),
            ...     output_measure=dp.approximate(dp.max_divergence()),
            ...     d_in=1,
            ...     d_mids=[(2., 1e-6), (1., 0.)]
            ... )
            >>> qbl_adaptive_comp = meas_adaptive_comp(int_dataset)


The first query to the approximate-DP adaptive compositor must be an
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
            
            >>> def make_zcdp_adaptive_composition(scale):
            ...     return dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(dp.c.make_adaptive_composition(
            ...         input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            ...         input_metric=dp.symmetric_distance(),
            ...         output_measure=dp.zero_concentrated_divergence(),
            ...         d_in=1,
            ...         d_mids=scale_weights(scale, weights)
            ...     )), delta=1e-6)
            
            >>> # find a scale parameter for the d_mids that makes the overall compositor satisfy (2., 1e-6)-approxDP
            >>> zcdp_compositor_scale = dp.binary_search_param(
            ...     make_zcdp_adaptive_composition, 
            ...     d_in=1, d_out=(2., 1e-6), T=float
            ... )
            
            >>> # construct a zCDP adaptive compositor that satisfies (2., 1e-6)-approxDP
            >>> meas_adaptive_comp_zCDP = make_zcdp_adaptive_composition(zcdp_compositor_scale)
            
            >>> # query the root approx-DP compositor queryable to get a child zCDP queryable
            >>> qbl_adaptive_comp_zCDP = qbl_adaptive_comp(meas_adaptive_comp_zCDP)
            
            >>> rho_1, rho_2 = scale_weights(zcdp_compositor_scale, weights)
            >>> rho_1, rho_2
            (0.0734..., 0.0146...)

Now that we’ve determined :math:`\rho_1` and :math:`\rho_2`, make a
release:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> def make_sum_zCDP(scale):
            ...     return (
            ...         input_space
            ...         >> dp.t.then_clamp((0, 10))
            ...         >> dp.t.then_sum()
            ...         >> dp.m.then_gaussian(scale)
            ...     )
            
            
            >>> dg_scale = dp.binary_search_param(make_sum_zCDP, d_in=1, d_out=rho_1)
            >>> print('zcdp sum:', qbl_adaptive_comp_zCDP(make_sum_zCDP(dg_scale)))
            zcdp sum: ...

At this point, we can submit queries to both the root approx-DP
compositor queryable (``qbl_adaptive_comp``) and the child zCDP compositor
queryable (``qbl_adaptive_comp_zCDP``).

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # convert the pure-DP count measurement to a approx-DP count measurement (where δ=0.)
            >>> meas_count_approxDP = dp.c.make_approximate(meas_count)
            
            >>> # submit the count measurement to the root approx-DP compositor queryable
            >>> print('approxDP count:', qbl_adaptive_comp(meas_count_approxDP))
            approxDP count: ...

We’ve now exhausted the privacy budget of the root approx-DP queryable,
but we can still query the child zCDP queryable.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> def make_count_zCDP(scale):
            ...     return (
            ...         input_space
            ...         >> dp.t.then_count()
            ...         >> dp.m.then_gaussian(scale)
            ...     )
            >>> dg_scale = dp.binary_search_param(make_count_zCDP, d_in=1, d_out=rho_2)
            >>> print('zcdp count:', qbl_adaptive_comp_zCDP(make_count_zCDP(dg_scale)))
            zcdp count: ...

Now the privacy budget of both queryables have been exhausted:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> qbl_adaptive_comp(meas_count_approxDP)
            Traceback (most recent call last):
            ...
            opendp.mod.OpenDPException: 
              FailedFunction("out of queries")

            >>> qbl_adaptive_comp_zCDP(make_sum_zCDP(dg_scale))
            Traceback (most recent call last):
            ...
            opendp.mod.OpenDPException: 
              FailedFunction("out of queries")

In conclusion, OpenDP provides several compositors with different
trade-offs, and interactive compositors (like adaptive composition)
provide a protective, differentially private interface for accessing any
dataset stored within the queryable.
