Application Structure
=====================

.. _parameter-search:

Parameter Search
----------------

There are many parameters in a typical DP measurement:

* ``d_in`` input distance (oftentimes how many records differ when you perturb one individual)
* ``d_out`` output distance (oftentimes the privacy budget)
* noise scale and any other parameters passed to the constructors

To evaluate a relation, you must fix all of these parameters.
The relation simply returns a boolean indicating if it passed.
As alluded to in the :ref:`relations` section,
if the relation passes for a given ``d_out``, it will also pass for any value greater than ``d_out``.
This behavior makes it possible to solve for any one parameter using a binary search
because the relation itself acts as your predicate function.
This is extremely powerful!

* | If you have a bound on ``d_in`` and a noise scale, you can solve for the tightest budget ``d_out`` that is still differentially private.
  | This is useful when you want to find the smallest budget that will satisfy a target accuracy.
* | If you have a bound on ``d_in`` and a budget ``d_out``, you can solve for the smallest noise scale that is still differentially private.
  | This is useful when you want to determine how accurate you can make a query with a given budget.
* | If you have a noise scale and a budget ``d_out``, you can solve for the smallest bound on ``d_in`` that is still differentially private.
  | This is useful when you want to determine an upper bound on how many records can be collected from an individual before needing to truncate.
* | If you have ``d_in``, ``d_out``, and noise scale derived from a target accuracy, you can solve for the smallest dataset size ``n`` that is still differentially private.
  | This is useful when you want to determine the necessary sample size when collecting data.
* | If you have ``d_in``, ``d_out``, and noise scale derived from a target accuracy, you can solve for the greatest clipping range that is still differentially private
  | This is useful when you want to minimize the likelihood of introducing bias.

OpenDP comes with some utility functions to make these binary searches easier to conduct:

* :func:`opendp.mod.binary_search_chain`: Pass it a function that makes a chain from one numeric argument, as well as ``d_in`` and ``d_out``. Returns the tightest chain.
* :func:`opendp.mod.binary_search_param`: Same as binary_search_chain, but returns the discovered parameter.
* :func:`opendp.mod.binary_search`: Pass a predicate function and bounds. Returns the discovered parameter. Useful when you just want to solve for ``d_in`` or ``d_out``.


.. _determining-accuracy:

Determining Accuracy
--------------------

The library contains utilities to estimate accuracy at a given noise scale and statistical significance level
or derive the necessary noise scale to meet a given target accuracy and statistical significance level.

The noise scale may be either laplace or gaussian.

:laplacian: | Applies to any L1 noise addition mechanism.
  | :func:`make_base_laplace() <opendp.meas.make_base_laplace>`
  | :func:`make_base_geometric() <opendp.meas.make_base_geometric>`
  | :func:`make_base_stability(MI=L1Distance[T]) <opendp.meas.make_base_stability>`
:gaussian: | Applies to any L2 noise addition mechanism.
  | :func:`make_base_gaussian() <opendp.meas.make_base_gaussian>`
  | :func:`make_base_stability(MI=L2Distance[T]) <opendp.meas.make_base_stability>`

The library provides the following functions for converting to and from noise scales:

* :func:`opendp.accuracy.laplacian_scale_to_accuracy`
* :func:`opendp.accuracy.accuracy_to_laplacian_scale`
* :func:`opendp.accuracy.gaussian_scale_to_accuracy`
* :func:`opendp.accuracy.accuracy_to_gaussian_scale`

These functions take either scale or accuracy, and alpha, a statistical significance parameter.

You can generally plug the distribution, scale, accuracy and alpha
into the following statement to interpret these functions:

.. code-block:: python

    f"When the {distribution} scale is {scale}, "
    f"the DP estimate differs from the true value by no more than {accuracy} "
    f"at a statistical significance level alpha of {alpha}, "
    f"or with (1 - {alpha})100% = {(1 - alpha) * 100}% confidence."


.. _putting-together:

Putting It Together
-------------------

Let's say we want to compute the DP mean of a csv dataset of student exam scores,
using a privacy budget of 1 epsilon.
We also want an accuracy estimate with 95% confidence.

Based on public knowledge that the class only has three exams,
we know that each student may contribute at most three records,
so our symmetric distance ``d_in`` is 3.

Referencing the :ref:`transformation-constructors` section,
we'll need to write a :ref:`transformation <transformation>` that computes a mean on a csv.
Our transformation will
:func:`parse a csv <opendp.trans.make_split_dataframe>`,
:func:`select a column <opendp.trans.make_select_column>`,
:func:`cast <opendp.trans.make_cast>`,
:func:`impute <opendp.trans.make_impute_constant>`,
:func:`clamp <opendp.trans.make_clamp>`,
:func:`resize <opendp.trans.make_bounded_resize>` and then aggregate with the
:func:`mean <opendp.trans.make_sized_bounded_mean>`.

.. doctest::

    >>> from opendp.trans import *
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

    * release a DP count first (:func:`count <opendp.trans.make_count>` >> :func:`base_geometric <opendp.meas.make_base_geometric>`), and then supply that count to resize
    * release a DP count and DP sum separately, and then postprocess

The next step is to make this computation differentially private.

Referencing the :ref:`measurement-constructors` section,
we'll need to choose a :ref:`measurement <measurement>` that can be chained with our transformation.
The :func:`base_laplace <opendp.meas.make_base_laplace>` measurement qualifies (barring :ref:`floating-point issues <floating-point>`).

Referencing the :ref:`parameter-search` section, :func:`binary_search_param <opendp.mod.binary_search_param>`
will help us find a noise scale parameter that satisfies our given budget.

.. doctest::

    >>> from opendp.meas import make_base_laplace
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
    'When the laplace scale is 1.33333333581686, the DP estimate differs from the true value by no more than 3.9943097055119687 at a statistical significance level alpha of 0.05, or with (1 - 0.05)100% = 95.0% confidence.'

Please be aware that the preprocessing (impute, clamp, resize) can introduce bias that the accuracy estimate cannot account for.
In this example, since the sensitive dataset is short two exams,
the release is slightly biased toward the imputation constant ``70.0``.

There are more examples in the next section!
