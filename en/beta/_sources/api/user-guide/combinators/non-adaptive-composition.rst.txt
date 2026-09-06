(Non-Adaptive) Composition
--------------------------

Any functions that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

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

        .. code:: pycon

            >>> # define the dataset space and how distances are measured
            >>> input_space = (
            ...     dp.vector_domain(dp.atom_domain(T=int)),
            ...     dp.symmetric_distance(),
            ... )

            >>> meas_count = (
            ...     input_space
            ...     >> dp.t.then_count()
            ...     >> dp.m.then_laplace(scale=1.0)
            ... )
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

        .. code:: pycon

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

The non-adaptive compositor takes a collection of queries to execute on the dataset simultaneously. 
When the data is passed in, all queries are evaluated together, in a single batch.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> meas_mean_fraction = dp.c.make_composition(
            ...     [meas_sum, meas_count]
            ... )

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

        .. code:: pycon

            >>> meas_mean_fraction.map(1)
            3.0

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: non-adaptive-composition-map
            :end-before: /non-adaptive-composition-map