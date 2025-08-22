Aggregation: Mean
=================

Any functions that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")


Known Dataset Size
------------------

The much easier case to consider is when the dataset size is known:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> input_space = (
            ...     dp.vector_domain(
            ...         dp.atom_domain(bounds=(0.0, 10.0)), size=10
            ...     ),
            ...     dp.symmetric_distance(),
            ... )
            >>> sb_mean_trans = dp.t.make_mean(*input_space)
            >>> sb_mean_trans([5.0] * 10)
            5.0

The sensitivity of this transformation is the same as in ``make_sum``
(when dataset size is known), but is divided by ``size``.

That is, :math:`map(d_{in}) = (d_{in} // 2) \cdot max(|L|, U) / size`,
where :math:`//` denotes integer division with truncation.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # since we are in the bounded-DP model, d_in should be a multiple of 2,
            >>> # because it takes one removal and one addition to change one record
            >>> sb_mean_trans.map(2)
            1.0000000000000169

Note that this operation does not divide by the length of the input
data, it divides by the size parameter passed to the constructor. As in
any other context, it is expected that the data passed into the function
is a member of the input domain, so no promises of privacy or
correctness are guaranteed when the data is not in the input domain. In
particular, the function may give a result with no error message.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> sb_mean_trans = dp.t.make_mean(*input_space)
            >>> sb_mean_trans([5.0])
            0.5

You can check that a dataset is a member of a domain by calling
``.member``:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> sb_mean_trans.input_domain.member([5.0])
            False

In this case, ``[5.]`` is not a member because the input domain consists
of vectors of length ten.

Unknown Dataset Size
--------------------

There are several approaches for releasing the mean when the dataset
size is unknown.

The first approach is to use the resize transformation. You can
separately release an estimate for the dataset size, and then preprocess
the dataset with a resize transformation.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> data = [5.0] * 10
            >>> bounds = (0.0, 10.0)

            >>> input_space = (
            ...     dp.vector_domain(dp.atom_domain(T=float)),
            ...     dp.symmetric_distance(),
            ... )

            >>> # (where TIA stands for Atomic Input Type)
            >>> count_meas = (
            ...     input_space
            ...     >> dp.t.then_count()
            ...     >> dp.m.then_laplace(1.0)
            ... )

            >>> dp_count = count_meas(data)

            >>> mean_meas = (
            ...     input_space
            ...     >> dp.t.then_impute_constant(0.0)
            ...     >> dp.t.then_clamp(bounds)
            ...     >> dp.t.then_resize(dp_count, constant=5.0)
            ...     >> dp.t.then_mean()
            ...     >> dp.m.then_laplace(1.0)
            ... )

            >>> print("dp mean:", mean_meas(data))
            dp mean: ...

The total privacy expenditure is the composition of the ``count_meas``
and ``mean_meas`` releases.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> print(
            ...     "map 1:",
            ...     dp.c.make_composition([count_meas, mean_meas]).map(1),
            ... )
            map 1: ...

Another approach is to compute the DP sum and DP count, and then
postprocess the output.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> dp_sum = (
            ...     input_space
            ...     >> dp.t.then_impute_constant(0.0)
            ...     >> dp.t.then_clamp(bounds)
            ...     >> dp.t.then_sum()
            ...     >> dp.m.then_laplace(10.0)
            ... )
            >>> dp_count = (
            ...     input_space
            ...     >> dp.t.then_count()
            ...     >> dp.m.then_laplace(1.0)
            ... )

            >>> dp_fraction_meas = dp.c.make_composition([dp_sum, dp_count])

            >>> dp_sum, dp_count = dp_fraction_meas(data)
            >>> print("dp mean:", dp_sum / dp_count)
            dp mean: ...
            >>> print("epsilon:", dp_fraction_meas.map(1))
            epsilon: ...

The same approaches are valid for the variance estimator. The `Resize preprocessing documentation <preprocess-resize.ipynb>`__
goes into greater detail on the tradeoffs of these approaches.
