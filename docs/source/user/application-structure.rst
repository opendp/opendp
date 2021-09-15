Application Structure
=====================

.. contents:: |toctitle|
    :local:

.. _accuracy:

Accuracy
--------

The library contains utilities to estimate accuracy at a given noise scale and statistical significance level,
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

.. _binary-search:

Binary Search
-------------

There are many parameters in a typical DP measurement:

* ``d_in`` input distance (oftentimes how many records differ when you perturb one individual)
* ``d_out`` output distance (oftentimes the privacy budget)
* noise scale or other hyperparameters passed to the constructors

To evaluate a relation, you must fix all of these parameters.
The relation simply returns a boolean indicating if it passed.

As alluded to in the :ref:`relations` section,
if the relation passes for a given ``d_out``, it will also pass for any value greater than ``d_out``.

This behavior makes it possible to solve for any one parameter using a binary search,
because the relation itself acts as your predicate function.
This is extremely powerful!

* | If you have a bound on ``d_in`` and a noise scale, you can solve for the tightest budget ``d_out`` that is still differentially private.
  | This is useful when you want to find the smallest budget that will satisfy a target accuracy.
* | If you have a bound on ``d_in`` and a budget ``d_out``, you can solve for the smallest noise scale that is still differentially private.
  | This is useful when you want to determine how accurate you can make a query with a given budget.
* | If you have a noise scale and a budget ``d_out``, you can solve for the smallest bound on ``d_in`` that is still differentially private.
  | This is useful when you want to determine an upper bound on how many records can be collected from an individual before needing to truncate.
* | If you have ``d_in``, ``d_out``, and noise scale derived from a target accuracy, and want to find the smallest dataset size ``n`` that is still differentially private.
  | This is useful when you want to determine the necessary sample size when collecting data.
* | If you have ``d_in``, ``d_out``, and noise scale derived from a target accuracy, and want to find the greatest clipping range that is still differentially private
  | This is useful when you want to minimize the likelihood of introducing bias.

OpenDP comes with some utility functions to make these binary searches easier to conduct:

* :func:`opendp.mod.binary_search_chain`: Pass it a function that makes a chain from one numeric argument, as well as ``d_in`` and ``d_out``. Returns the tightest chain.
* :func:`opendp.mod.binary_search_param`: Same as binary_search_chain, but returns the discovered parameter.
* :func:`opendp.mod.binary_search`: Pass a predicate function and bounds. Returns the discovered parameter. Useful when you just want to solve for ``d_in`` or ``d_out``.

It might be helpful to go through an example on how to use this.

.. _putting-together:

Putting It Together
-------------------

Lets say we want to compute the DP mean, along with an 95% confident accuracy estimate, of a dataset of student exam scores.
We have a privacy budget of one-epsilon, ``d_out`` is 1.
With the public knowledge that the class only has three exams,
we know each student may contribute at most three records, so ``d_in`` is 3.

Referencing the :ref:`transformations` section, first we'll need to cast, impute, clamp and resize.

Then we'll aggregate and chain with a :func:`opendp.meas.make_base_laplace` measurement.

Referencing the :ref:`binary-search` section, :func:`opendp.mod.binary_search_param`
will help us find a noise scale parameter that satisfies our given budget.
Referencing the :ref:`accuracy` section, :func:`opendp.accuracy.laplace_scale_to_accuracy`
can be used to convert this discovered noise scale parameter to an accuracy estimate.

.. doctest::

    >>> from opendp.trans import *
    >>> from opendp.meas import *
    >>> from opendp.mod import binary_search_chain, enable_features
    ...
    >>> # floating-point numbers are not differentially private! Here be dragons.
    >>> enable_features("floating-point")
    ...
    >>> num_tests = 3
    >>> num_students = 50
    >>> size = num_students * num_tests
    >>> bounds = (0., 100.)  # range of valid exam scores
    >>> epsilon = 1. # target budget
    ...
    >>> # create most of the chain once
    >>> aggregator = (
    ...     make_clamp(bounds) >>
    ...     make_bounded_resize(size, bounds, constant=0.) >>
    ...     make_sized_bounded_mean(size, bounds)
    ... )
    >>> # find the smallest noise scale for which the relation still passes
    >>> # if we didn't need a handle on scale, we could just use binary_search_chain and inline the lambda
    >>> make_chain = lambda s: aggregator >> make_base_laplace(s)
    >>> scale = binary_search_param(make_chain, d_in=num_tests, d_out=epsilon) # -> 1.33
    >>> meas = make_chain(scale)
    ...
    >>> # We already know the privacy relation will pass, but this is how we check it!
    >>> assert meas.check(num_tests, epsilon)
    ...
    >>> # Spend 1 epsilon creating our DP estimate on the private data
    >>> dummy_private_dataset = [95.] * 150
    >>> release = meas(dummy_private_dataset) # -> 95.8
    ...
    >>> # We also wanted an accuracy estimate...
    >>> from opendp.accuracy import laplacian_scale_to_accuracy
    >>> alpha = .05
    >>> accuracy = laplacian_scale_to_accuracy(scale, alpha)
    >>> (f"When the laplacian scale is {scale}, "
    ...  f"the DP estimate differs from the true value by no more than {accuracy} "
    ...  f"at a statistical significance level alpha of {alpha}, "
    ...  f"or with (1 - {alpha})100% = {(1 - alpha) * 100}% confidence.")
    'When the laplacian scale is 1.33333333581686, the DP estimate differs from the true value by no more than 3.9943097055119687 at a statistical significance level alpha of 0.05, or with (1 - 0.05)100% = 95.0% confidence.'
