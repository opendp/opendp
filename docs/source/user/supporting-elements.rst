Supporting Elements
===================

This document builds on the :ref:`core-structures` documentation to give a closer look at the constituent pieces of Measurements and Transformations.

Domains
-------
A domain describes the set of all possible values that data may take on.
Two domains are bundled in each Transformation or Measurement to describe the input and output domains of the function.
These domains serve two purposes:

#. The relation depends on the input and output domain in its proof to restrict the set of neighboring datasets or distributions.
   An example is the relation for `make_sized_bounded_sum`,
   which makes use of a `SizedDomain` domain descriptor to more tightly bound the sensitivity.
#. Combinators also use domains to ensure the output is well-defined.
   For instance, chainer constructors check that interior domains are equivalent,
   to guarantee that the output of the interior function is a valid input to the exterior function.

Some common domains are:

* AllDomain<T>, the set of all non-null values in the type T
* BoundedDomain<T>, the set of all non-null values in the type T, bounded between some L and U.
* VectorDomain<D>, the set of all vectors, where each element of the vector is a member of domain D.
* SizedDomain<D>, to restrict the size of members of some domain D.

In many cases, you provide some qualities about the underlying domain and the rest is automatically chosen by the constructor.

Lets look at the Transformation returned from `make_bounded_sum(bounds=(0, 1))`.
The input domain has type `VectorDomain<BoundedDomain<i32>>`,
read as "the set of all vectors of 32-bit signed integers bounded between 0 and 1."
The bounds argument to the constructor provides L and U, and since TIA (atomic input type) is not passed,
TIA is inferred from the type of the public bounds.
The output domain is simply `AllDomain<i32>`, or "the set of all 32-bit signed integers."


Metrics and Measures
--------------------
A metric is a function that computes the distance between two elements of a set.
In OpenDP, a concrete example is the type `AbsoluteDistance<u8>`,
read as "the absolute distance metric where distances are expressed in unsigned 8-bit integers."
Another example is `SymmetricDistance`, or "the symmetric distance metric",
which is used to count the number of additions or removals to convert one dataset into another dataset.

Some common metrics are:

* SymmetricDistance, computed as the number of additions or removals to convert one dataset into another dataset
* SubstituteDistance, computed as the number of substitutions to convert one dataset into another dataset
* AbsoluteDistance<T>, computed as the absolute difference between any two numeric scalars of type T
* L1Distance<T>, computed as the L1 distance between any two numeric vectors of type T
* L2Distance<T>, computed as the L2 distance between any two numeric vectors of type T

A measure is a function for computing a number from a subset of a set.
In OpenDP, a concrete example is the type `MaxDivergence<f64>`,
read as "the max divergence metric where numbers are expressed in terms of 64-bit floats."
The max divergence metric has distances that correspond exactly to epsilon in the pure definition of differential privacy.

Some common measures are:

* MaxDivergence<T>, which corresponds to pure-DP, where epsilon has type T
* SmoothedMaxDivergence<T>, which corresponds to approximate-DP, where epsilon and delta have type T

Consider that it is never necessary to actually evaluate these functions!
The sole purpose of metrics and measures is to provide the units for distances.

Relations
---------
We assert the privacy properties of a Transformation or Measurement's function via a relation.
The relation accepts an input distance `d_in` and an output distance `d_out`,
where `d_in` and `d_out` are distances in terms of the input and output metric or measure.

Let `d_X` be the input metric, `d_Y` be the output metric or measure,
and `f` be the function in the Transformation or Measurement.

The relation fails if there exists two members of the input domain, x and x', that satisfy two conditions:

#. d_X(x, x') <= d_in. The inputs are d_in-close.
#. d_Y(f(x), f(x')) > d_out. But the outputs aren't d_out-close!

In plaintext:
There should be no perturbation to the input (by at most distance `d_in`) that can significantly influence the output (greater than distance `d_out`).

The typical perturbation is the adjustment made to a dataset by adding or removing a user.


Distances
---------

It is critical that you supply the correct `d_in` for the relation,
whereas you can use utilities in the library to search for the tightest `d_out`.

If you are starting with a dataset the input metric of your measurement will likely be SymmetricDistance.
In this case, `d_in` is the maximum number of records that an individual may contribute.

For example, consider a dataset of student exam scores where we want to compute the DP-mean.
If we know the class has three exams, then we know each student may contribute at most three records, so `d_in` should be 3.

We can use this information in a measurement that casts, imputes, clamps, resizes, averages, and then adds noise.
We can use the relation on this chained measurement to conduct a binary search to find `d_out`, where `d_in` = 3.

It is also possible to use `binary_search_chain` to find a noise scale parameter that satisfies a given budget.


Examples
--------

The following example checks the stability relation on a clamp transformation.

.. code-block:: python

    from opendp.trans import make_clamp
    clamp = make_clamp(bounds=(1, 10))

    # The maximum number of records that any one individual may influence in your dataset
    in_symmetric_distance = 3
    # clamp is a 1-stable transformation, so this should pass for any symmetric_distance >= 3
    assert clamp.check(d_in=in_symmetric_distance, d_out=4)


This more complete example realizes the exam scores example and demonstrates usage of the `binary_search_chain` function.

.. code-block:: python

    from opendp.trans import *
    from opendp.meas import *
    from opendp.mod import binary_search_chain, enable_features

    # floating-point numbers are not differentially private! Here be dragons.
    enable_features("floating-point")

    num_tests = 3
    num_students = 50
    size = num_students * num_tests
    bounds = (0., 100.)  # range of valid exam scores
    epsilon = 1. # target budget

    # create most of the chain once
    aggregator = (
        make_clamp(bounds) >>
        make_bounded_resize(size, bounds, 0.) >>
        make_sized_bounded_mean(size, bounds)
    )
    meas = binary_search_chain(
        lambda s: aggregator >> make_base_laplace(s),
        d_in=num_tests, d_out=epsilon)

    assert meas.check(num_tests, epsilon)
