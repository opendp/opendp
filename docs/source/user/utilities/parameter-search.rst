
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

