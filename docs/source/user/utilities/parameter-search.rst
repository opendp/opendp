
.. _parameter-search:

Parameter Search
================
The OpenDP library provides two different kinds of search algorithms to aid in finding free parameters.
The primary being the binary search functions, and secondary an exponential search.

Binary Search
-------------
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

OpenDP comes with some utility functions to make these binary searches easier to conduct:

* :func:`opendp.mod.binary_search_chain`: Pass it a function that makes a chain from one numeric argument, as well as ``d_in`` and ``d_out``. Returns the tightest chain.
* :func:`opendp.mod.binary_search_param`: Same as binary_search_chain, but returns the discovered parameter.
* :func:`opendp.mod.binary_search`: Pass a predicate function and bounds. Returns the discovered parameter. Useful when you just want to solve for ``d_in`` or ``d_out``.

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
    ...     lambda c: make_bounded_sum((-c, c)) >> make_base_gaussian(scale=1.), 
    ...     d_in=2, d_out=1.)
    0.3535533897700931

The API documentation on these functions have more specific usage examples.

Exponential Search
------------------

An exponential search starts at an origin location in the search space, and finds the first step where a predicate function changes value.
Generally speaking, each step the algorithm takes is exponentially larger than the previous one.
If bounds are not passed to the binary search algorithm, an exponential search is run to find the bounds for the binary search.
This is generally less likely to overflow than if you were to set large binary search bounds, because the magnitude of exponential bounds queries starts small.

:func:`opendp.mod.exponential_bounds_search` uses a number of heuristics that tend to work well on most problems.
If the heuristics fail you, then pass your own bounds into the binary search utilities.

A more in-depth explanation of this algorithm is hidden below:

.. raw:: html

   <details style="margin:-1em 0 2em 4em">
   <summary><a>Expand Me</a></summary>

If it is unkown whether the algorithm needs integer or float bounds, the algorithm first checks the predicate at a float zero. 
If a type error is thrown, it similarly checks the predicate function at an integer zero.
If the predicate function fails both times, you'll have to pass a type argument ``T`` of either ``float`` or ``int``.
This heuristic can fail if the predicate function is invalid at zero.

The integer bounds search doesn't actually take exponential steps, it checks the predicate function along zero, one, and eight even steps of size 2^16.
On the other hand, since floats are logarithmically distributed, 8 steps are made along 2^(k^2).
This explores a parameter regime that is unlikely to overflow, even when the origin is offset.

If the positive band search fails to find a change in sign, then the same procedure is run in the negative direction.
In the case that no acceptance region crosses the edge of a search band, the algorithm gives up, 
and you'll have to work out a reasonable set of bounds that intersect the acceptance region on your own.
Luckily, most predicate functions are monotonic, so this is unlikely to happen.

If at any time the predicate function throws an exception, then a search is run for the decision boundary of the exception.
We can safely consider the exception region invalid, and attempt to exclude it from the search space.
An example of this is when searching for a suitable size, n, for which the predicate function outright throws an exception if negative due to being malformed.

If this search fails to find an edge to the exception region, we give up, and claim that the predicate function always fails.
Otherwise, we shift the origin of the bounds search to the exception boundary, and try one more directional bounds search away from the exception.

.. raw:: html

   </details>