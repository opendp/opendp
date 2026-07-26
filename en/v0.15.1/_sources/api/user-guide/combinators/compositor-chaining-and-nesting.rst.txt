Compositor Chaining and Nesting
===============================

Chaining
--------

Since non-adaptive compositors, adaptive compositors, and privacy filters are just "plain-old-measurements," 
they also support chaining.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> meas_adaptive_comp = dp.c.make_adaptive_composition(
            ...     input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            ...     input_metric=dp.symmetric_distance(),
            ...     output_measure=dp.max_divergence(),
            ...     d_in=1,
            ...     d_mids=[2.0, 1.0],
            ... )

            >>> str_space = (
            ...     dp.vector_domain(dp.atom_domain(T=str)),
            ...     dp.symmetric_distance(),
            ... )
            >>> meas_adaptive_comp_str = (
            ...     str_space
            ...     >> dp.t.then_cast_default(int)
            ...     >> meas_adaptive_comp
            ... )

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
            >>> qbl_adaptive_comp_str = meas_adaptive_comp_str(
            ...     ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"]
            ... )
            >>> qbl_adaptive_comp_str(meas_sum), qbl_adaptive_comp_str(
            ...     meas_count
            ... )
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

        .. code:: pycon

            >>> max_contributions = 1
            >>> sum_trans = (
            ...     input_space
            ...     >> dp.t.then_clamp((0, 10))
            ...     >> dp.t.then_sum()
            ... )
            >>> meas_adaptive_comp = (
            ...     sum_trans
            ...     >> dp.c.then_adaptive_composition(
            ...         output_measure=dp.max_divergence(),
            ...         d_in=sum_trans.map(max_contributions),
            ...         d_mids=[2.0, 1.0],
            ...     )
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

        .. code:: pycon

            >>> meas_adaptive_comp = dp.c.make_adaptive_composition(
            ...     input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            ...     input_metric=dp.symmetric_distance(),
            ...     output_measure=dp.approximate(dp.max_divergence()),
            ...     d_in=1,
            ...     d_mids=[(2.0, 1e-6), (1.0, 0.0)],
            ... )
            >>> int_dataset = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            >>> qbl_adaptive_comp = meas_adaptive_comp(int_dataset)


The first query to the approximate-DP adaptive compositor must be an
approximate-DP measurement that satisfies $(2, 10^{-6})$-DP.
We will now use the library to find a set of :math:`\rho` parameters
that will satisfy this level of privacy, under a given set of weights.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # find ρ_1, ρ_2 such that ρ_1 + ρ_2 = ρ <= (2, 1e-6),
            >>> #    and ρ_1 is 5 times larger than ρ_2
            >>> weights = [5.0, 1.0]


            >>> def scale_weights(scale, weights):
            ...     return [scale * w for w in weights]
            ...

            >>> def make_zcdp_adaptive_composition(scale):
            ...     return dp.c.make_fix_delta(
            ...         dp.c.make_zCDP_to_approxDP(
            ...             dp.c.make_adaptive_composition(
            ...                 input_domain=dp.vector_domain(
            ...                     dp.atom_domain(T=int)
            ...                 ),
            ...                 input_metric=dp.symmetric_distance(),
            ...                 output_measure=dp.zero_concentrated_divergence(),
            ...                 d_in=1,
            ...                 d_mids=scale_weights(scale, weights),
            ...             )
            ...         ),
            ...         delta=1e-6,
            ...     )
            ...

            >>> # find a scale parameter for the d_mids that makes the overall compositor satisfy (2., 1e-6)-approxDP
            >>> zcdp_compositor_scale = dp.binary_search_param(
            ...     make_zcdp_adaptive_composition,
            ...     d_in=1,
            ...     d_out=(2.0, 1e-6),
            ...     T=float,
            ... )

            >>> # construct a zCDP adaptive compositor that satisfies (2., 1e-6)-approxDP
            >>> meas_adaptive_comp_zCDP = make_zcdp_adaptive_composition(
            ...     zcdp_compositor_scale
            ... )

            >>> # query the root approx-DP compositor queryable to get a child zCDP queryable
            >>> qbl_adaptive_comp_zCDP = qbl_adaptive_comp(
            ...     meas_adaptive_comp_zCDP
            ... )

            >>> rho_1, rho_2 = scale_weights(zcdp_compositor_scale, weights)
            >>> rho_1, rho_2
            (0.0734..., 0.0146...)

Now that we’ve determined :math:`\rho_1` and :math:`\rho_2`, make a
release:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> def make_sum_zCDP(scale):
            ...     return (
            ...         input_space
            ...         >> dp.t.then_clamp((0, 10))
            ...         >> dp.t.then_sum()
            ...         >> dp.m.then_gaussian(scale)
            ...     )
            ...


            >>> dg_scale = dp.binary_search_param(
            ...     make_sum_zCDP, d_in=1, d_out=rho_1
            ... )
            >>> print(
            ...     "zcdp sum:",
            ...     qbl_adaptive_comp_zCDP(make_sum_zCDP(dg_scale)),
            ... )
            zcdp sum: ...

At this point, we can submit queries to both the root approx-DP
compositor queryable (``qbl_adaptive_comp``) and the child zCDP compositor
queryable (``qbl_adaptive_comp_zCDP``).

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # convert the pure-DP count measurement to a approx-DP count measurement (where δ=0.)
            >>> meas_count_approxDP = dp.c.make_approximate(meas_count)

            >>> # submit the count measurement to the root approx-DP compositor queryable
            >>> print(
            ...     "approxDP count:",
            ...     qbl_adaptive_comp(meas_count_approxDP),
            ... )
            approxDP count: ...

We’ve now exhausted the privacy budget of the root approx-DP queryable,
but we can still query the child zCDP queryable.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> def make_count_zCDP(scale):
            ...     return (
            ...         input_space
            ...         >> dp.t.then_count()
            ...         >> dp.m.then_gaussian(scale)
            ...     )
            ...
            >>> dg_scale = dp.binary_search_param(
            ...     make_count_zCDP, d_in=1, d_out=rho_2
            ... )
            >>> print(
            ...     "zcdp count:",
            ...     qbl_adaptive_comp_zCDP(make_count_zCDP(dg_scale)),
            ... )
            zcdp count: ...

Now the privacy budget of both queryables have been exhausted:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

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
