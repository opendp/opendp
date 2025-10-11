.. _constructors:

Constructors
============

In OpenDP, Measurements and Transformations are created by calling constructor functions.
The majority of the library's interface consists of these `make_*` constructors,
like :py:func:`make_clamp <opendp.transformations.make_clamp>` or :py:func:`make_base_laplace <opendp.measurements.make_base_laplace>`.

Because Measurements and Transformations are themselves like functions (they can be invoked on an input and return an output),
you can think of constructors as higher-order functions:
You call them to produce another function that you will then feed data.

Constructors are organized into Transformations, Measurements and Combinators:

.. toctree::

   transformations
   measurements
   combinators

Let's demonstrate with a few examples!
In this example, we use a constructor to build a clamp transformation:

.. testsetup::

    from opendp.mod import enable_features
    enable_features('contrib')

.. doctest::

    >>> from opendp.transformations import make_clamp
    >>> clamper = make_clamp(bounds=(0, 10))
    ...
    >>> # invoke the function with some data
    >>> clamper([-1, 0, 3, 5, 10, 20])
    [0, 0, 3, 5, 10, 10]

The input metric and output metric are implicitly `SymmetricDistance`.
We can use the stability map on this transformation to ask a hypothetical question:
If neigboring datasets differ by as much as `d_in`, then how much can neigboring datasets differ after running this transformation (`d_out`)?

.. doctest::

    >>> # map an input distance to an output distance
    >>> clamper.map(d_in = 1)
    1

We know the clamp transformation is 1-stable, so `d_out = 1 * d_in`. 
Similarly, we can check if the transformation is (`d_in`, `d_out`)-close (that is, `map(d_in) <= d_out` ) by checking the stability relation:

.. doctest::

    >>> # check if clamper is (d_in, d_out)-close
    >>> clamper.check(d_in = 1, d_out = 1)
    True

We can use the `>>` shorthand to chain multiple transformations
(internally uses the `make_chain_tt <chaining>`_ combinator).

.. doctest::

    >>> from opendp.transformations import make_cast_default
    ...
    >>> # build another transformation that casts, and fills nulls with a default value (0)
    >>> caster = make_cast_default(TIA=str, TOA=int)
    ...
    >>> # construct a new transformation such that preprocessor(x) = clamper(caster(x))
    >>> preprocessor = caster >> clamper
    ...
    >>> # invoke the chained transformation
    >>> preprocessor(["1", "2", "3", "20", "a"])
    [1, 2, 3, 10, 0]
    >>> # since both are 1-stable transformations, then the map remains trivial
    >>> preprocessor.map(d_in=2)
    2


In this simplified example with the :py:func:`opendp.measurements.make_base_discrete_laplace` constructor, we assume the data was properly preprocessed and aggregated such that the sensitivity (by absolute distance) is at most 1.


.. doctest::

    >>> from opendp.measurements import make_base_discrete_laplace
    ...
    >>> # call the constructor to produce the measurement `base_dl`
    >>> base_dl = make_base_discrete_laplace(scale=1.0)
    ...
    >>> # investigate the privacy relation
    >>> absolute_distance = 1
    >>> base_dl.map(d_in=absolute_distance) # returns epsilon
    1.0
    >>> # feed some data/invoke the measurement as a function
    >>> aggregated = 5
    >>> release = base_dl(aggregated)

As you can see, constructor functions are the gateway to building differentially private analyses in OpenDP.
The next sections are a tour of the available constructor functions.
