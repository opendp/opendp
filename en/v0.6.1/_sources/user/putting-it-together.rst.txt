
.. _putting-together:

Putting It Together
===================

Let's say we want to compute the DP mean of a csv dataset of student exam scores,
using a privacy budget of 1 epsilon.
We also want an accuracy estimate with 95% confidence.

Based on public knowledge that the class only has three exams,
we know that each student may contribute at most three records,
so our symmetric distance ``d_in`` is 3.

Referencing the :ref:`transformation-constructors` section,
we'll need to write a :ref:`transformation <transformation>` that computes a mean on a csv.
Our transformation will
:func:`parse a csv <opendp.transformations.make_split_dataframe>`,
:func:`select a column <opendp.transformations.make_select_column>`,
:func:`cast <opendp.transformations.make_cast>`,
:func:`impute <opendp.transformations.make_impute_constant>`,
:func:`clamp <opendp.transformations.make_clamp>`,
:func:`resize <opendp.transformations.make_bounded_resize>` and then aggregate with the
:func:`mean <opendp.transformations.make_sized_bounded_mean>`.

.. doctest::

    >>> from opendp.transformations import *
    >>> from opendp.mod import enable_features
    >>> enable_features('contrib') # we are using un-vetted constructors
    ...
    >>> num_tests = 3  # d_in=symmetric distance; we are told this is public knowledge
    >>> budget = 1. # d_out=epsilon
    ...
    >>> num_students = 50  # we are assuming this is public knowledge
    >>> size = num_students * num_tests  # 150 exams
    >>> bounds = (0., 100.)  # range of valid exam scores- clearly public knowledge
    >>> constant = 70. # impute nullity with a guess
    ...
    >>> transformation = (
    ...     make_split_dataframe(',', col_names=['Student', 'Score']) >>
    ...     make_select_column(key='Score', TOA=str) >>
    ...     make_cast(TIA=str, TOA=float) >>
    ...     make_impute_constant(constant=constant) >>
    ...     make_clamp(bounds) >>
    ...     make_bounded_resize(size, bounds, constant=constant) >>
    ...     make_sized_bounded_mean(size, bounds)
    ... )


.. note::

    For brevity, we made the assumption that the number of students in the class is also public knowledge,
    which allowed us to infer dataset size.
    If your dataset size is not public knowledge, you could either:

    * release a DP count first (:func:`count <opendp.transformations.make_count>` >> :func:`base_discrete_laplace <opendp.measurements.make_base_discrete_laplace>`), and then supply that count to resize
    * release a DP count and DP sum separately, and then postprocess

The next step is to make this computation differentially private.

Referencing the :ref:`measurement-constructors` section,
we'll need to choose a :ref:`measurement <measurement>` that can be chained with our transformation.
The :func:`base_laplace <opendp.measurements.make_base_laplace>` measurement qualifies.

Referencing the :ref:`parameter-search` section, :func:`binary_search_param <opendp.mod.binary_search_param>`
will help us find a noise scale parameter that satisfies our given budget.

.. doctest::

    >>> from opendp.measurements import make_base_laplace
    >>> from opendp.mod import enable_features, binary_search_param
    ...
    >>> # Please make yourself aware of the dangers of floating point numbers
    >>> enable_features("floating-point")
    ...
    >>> # Find the smallest noise scale for which the relation still passes
    >>> # If we didn't need a handle on scale (for accuracy later),
    >>> #     we could just use binary_search_chain and inline the lambda
    >>> make_chain = lambda s: transformation >> make_base_laplace(s)
    >>> scale = binary_search_param(make_chain, d_in=num_tests, d_out=budget) # -> 1.33
    >>> measurement = make_chain(scale)
    ...
    >>> # We already know the privacy relation will pass, but this is how you check it
    >>> assert measurement.check(num_tests, budget)
    ...
    >>> # How did we get an entire class full of Salils!? ...and 2 must have gone surfing instead
    >>> mock_sensitive_dataset = "\n".join(["Salil,95"] * 148)
    ...
    >>> # Spend 1 epsilon creating our DP estimate on the private data
    >>> release = measurement(mock_sensitive_dataset) # -> 95.8


We also wanted an accuracy estimate.
Referencing the :ref:`determining-accuracy` section, :func:`laplacian_scale_to_accuracy <opendp.accuracy.laplacian_scale_to_accuracy>`
can be used to convert the earlier discovered noise scale parameter into an accuracy estimate.

.. doctest::

    >>> # We also wanted an accuracy estimate...
    >>> from opendp.accuracy import laplacian_scale_to_accuracy
    >>> alpha = .05
    >>> accuracy = laplacian_scale_to_accuracy(scale, alpha)
    >>> (f"When the laplace scale is {scale}, "
    ...  f"the DP estimate differs from the true value by no more than {accuracy} "
    ...  f"at a statistical significance level alpha of {alpha}, "
    ...  f"or with (1 - {alpha})100% = {(1 - alpha) * 100}% confidence.")
    'When the laplace scale is 2.0000000000003357, the DP estimate differs from the true value by no more than 5.991464547108987 at a statistical significance level alpha of 0.05, or with (1 - 0.05)100% = 95.0% confidence.'

Please be aware that the preprocessing (impute, clamp, resize) can introduce bias that the accuracy estimate cannot account for.
In this example, since the sensitive dataset is short two exams,
the release is slightly biased toward the imputation constant ``70.0``.
