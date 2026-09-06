
Privacy Filters
---------------

You can convert any odometer into a measurement by setting an upper bound on the privacy loss.
The following example converts the fully adaptive composition odometer into a privacy filter
that rejects any query that would cause the privacy loss to exceed 2.0:

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

Privacy filters are measurements, meaning that they can be passed into :func:`~opendp.combinators.make_composition`, 
adaptive composition queryables, or into other combinators.
However, they have the added benefit of not needing to specify privacy-loss parameters ahead-of-time.
When the privacy filter (``meas_fully_adaptive_comp``) is invoked, 
it still returns an *odometer queryable*, but this time the queryable will limit the overall privacy loss.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            >>> qbl_fully_adaptive_comp = meas_fully_adaptive_comp(
            ...     int_dataset
            ... )

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

        .. code:: pycon

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

        .. code:: pycon

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
