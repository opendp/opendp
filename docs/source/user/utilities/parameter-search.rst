
.. _parameter-search:

Parameter Search
================

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

.. testsetup::

    from opendp.measurements import *
    from opendp.transformations import *
    from opendp.mod import *
    from opendp.mod import enable_features
    enable_features('contrib', 'floating-point')

* | If you have a bound on ``d_in`` and a budget ``d_out``, you can solve for the smallest noise scale that is still differentially private.
  | This is useful when you want to determine how accurate you can make a query with a given budget.


  .. doctest::

    >>> binary_search_param(lambda s: make_base_gaussian(scale=s), d_in=1., d_out=1.)
    0.7071067811865477
  
* | If you have a bound on ``d_in`` and a noise scale, you can solve for the tightest budget ``d_out`` that is still differentially private.
  | This is useful when you want to find the smallest budget that will satisfy a target accuracy.

  .. doctest::

    >>> # in this case, a search is unnecessary. We can just use the map:
    >>> make_base_gaussian(scale=1.).map(d_in=1.)
    0.5

* | If you have a noise scale and a budget ``d_out``, you can solve for the largest bound on ``d_in`` that is still differentially private.
  | This is useful when you want to determine an upper bound on how many records can be collected from an individual before needing to truncate.

  .. doctest::

    >>> # finds the largest permissible d_in, a sensitivity
    >>> binary_search(lambda d_in: make_base_gaussian(scale=1.).check(d_in=d_in, d_out=1.))
    1.414213562373095


* | If you have ``d_in``, ``d_out``, and noise scale derived from a target accuracy, you can solve for the smallest dataset size ``n`` that is still differentially private.
  | This is useful when you want to determine the necessary sample size when collecting data.

  .. doctest::

    >>> # finds the smallest n
    >>> binary_search_param(
    ...     lambda n: make_sized_bounded_mean(n, (0., 10.)) >> make_base_gaussian(scale=1.), 
    ...     d_in=2, d_out=1.)
    8

* | If you have ``d_in``, ``d_out``, and noise scale derived from a target accuracy, you can solve for the greatest clipping range that is still differentially private
  | This is useful when you want to minimize the likelihood of introducing bias.

  .. doctest::

    >>> # finds the largest clipping bounds
    >>> binary_search_param(
    ...     lambda c: make_bounded_sum((c, c)) >> make_base_gaussian(scale=1.), 
    ...     d_in=2, d_out=1.)
    0.7071067778938249

OpenDP comes with some utility functions to make these binary searches easier to conduct:

* :func:`opendp.mod.binary_search_chain`: Pass it a function that makes a chain from one numeric argument, as well as ``d_in`` and ``d_out``. Returns the tightest chain.
* :func:`opendp.mod.binary_search_param`: Same as binary_search_chain, but returns the discovered parameter.
* :func:`opendp.mod.binary_search`: Pass a predicate function and bounds. Returns the discovered parameter. Useful when you just want to solve for ``d_in`` or ``d_out``.

The API documentation on these functions have more specific usage examples.
