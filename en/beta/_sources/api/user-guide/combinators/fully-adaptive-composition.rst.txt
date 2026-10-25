
.. _fully-adaptive-composition:

Fully Adaptive Composition and Odometers
----------------------------------------

Where adaptive composition allows for queries to be chosen adaptively,
*fully* adaptive composition also allows for the *privacy loss* of queries to be chosen adaptively.
The API for fully adaptive composition matches that of adaptive composition,
but drops the ``d_mids`` argument, as these will be chosen as you go.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> odom_fully_adaptive_comp = (
            ...     dp.c.make_fully_adaptive_composition(
            ...         input_domain=dp.vector_domain(
            ...             dp.atom_domain(T=int)
            ...         ),
            ...         input_metric=dp.symmetric_distance(),
            ...         output_measure=dp.max_divergence(),
            ...     )
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

        .. code:: pycon

            >>> int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            >>> qbl_fully_adaptive_comp = odom_fully_adaptive_comp(
            ...     int_dataset
            ... )
    
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

        .. code:: pycon

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

        .. code:: pycon

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

        .. code:: pycon

            >>> qbl_fully_adaptive_comp.privacy_loss(1)
            3.0

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/compositors-framework.R
            :language: r
            :start-after: fully-adaptive-composition-loss2
            :end-before: /fully-adaptive-composition-loss2
