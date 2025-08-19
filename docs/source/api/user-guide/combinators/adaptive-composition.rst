
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

