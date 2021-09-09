.. _core-structures:

Core Structures
===============

OpenDP is focused on creating computations with specific privacy characteristics.
These computations are modeled with two core structures in OpenDP: Transformations and Measurements.
These structures are in all OpenDP programs, regardless of the underlying algorithm or definition of privacy.
By modeling computations in this abstract way, we're able to combine them in flexible arrangements and reason about the resulting programs.

Measurement
-----------

A Measurement is a randomized mapping from datasets to outputs of an arbitrary type.
Each Measurement consists of a privacy relation and a function.

The privacy relation is used to check if the function is `d_out`-DP on `d_in`-close inputs.
The privacy relation is qualified by an input metric and an output measure, which give units to the distances `d_in` and `d_out`.

The function is used at most once to create a differentially private release.
The function is qualified by an input domain and an output domain, which define the set of valid data inputs and the set of valid outputs.

Transformation
--------------

A Transformation is a (deterministic) mapping from datasets to datasets.
Each Transformation consists of a stability relation and a function.

Similarly to measurements, the stability relation is used to check if the function output is `d_out`-close on `d_in`-close inputs.
The stability relation is qualified by an input metric and an output metric, which give units to `d_in` and `d_out`.

The function transforms the data, but the output is not differentially private.
Just like in measurements, the function is qualified by an input domain and output domain, which restrict the set of valid data inputs and outputs.

Transformations are used to preprocess and aggregate the data before chaining with a measurement.

Constructors and Functions
--------------------------

In OpenDP, Measurements and Transformations are created by calling constructor functions.
The majority of the library's interface consists of these constructors.

Because Measurements and Transformations are themselves like functions (they can be invoked on an input and return an output), you can think of constructors as higher-order functions:
You call them to produce another function, that you will then investigate the privacy or stability relation of and feed data.

In this simplified example with `make_base_geometric` we assume the data was properly preprocessed and aggregated such that the sensitivity by absolute distance is at most 1.

.. code-block:: python

    from opendp.meas import make_base_geometric

    # call the constructor to produce a measurement
    base_geometric = make_base_geometric(scale=1.0)

    # investigate the privacy relation
    absolute_distance = 1.0
    epsilon = 1.0
    assert base_geometric.check(d_in=absolute_distance, d_out=epsilon)

    # feed some data
    aggregated = 5
    release = base_geometric(data=aggregated)

Chaining
--------

Two of the most essential constructors are the combinators that chain transformations with transformations, or transformations with measurements.
The usage of these constructors is so common that a special syntax (`>>`) is provided.

We continue the prior example by chaining `base_geometric` with a `bounded_sum` using this right-shift operator shorthand.

.. code-block:: python

    from opendp.trans import make_bounded_sum

    # call the constructor to produce a transformation
    bounded_sum = make_bounded_sum(bounds=(0, 1))
    # CHAINING: opendp.meas.make_mt_chain
    chained_measurement = bounded_sum >> base_geometric

    # investigate the privacy relation
    symmetric_distance = 1
    epsilon = 1.0
    assert chained_measurement.check(d_in=symmetric_distance, d_out=epsilon)

    # feed some data
    dataset = [0, 0, 1, 1, 0, 1, 1, 1]
    release = chained_measurement(data=dataset)

In this example the chaining was successful because:

* bounded_sum's output domain is equivalent to base_geometric's input domain
* bounded_sum's output metric is equivalent to base_geometric's input metric

In addition, the resulting Measurement's input domain and input metric come from bounded_sum's input domain and input metric.
This is intended to enable further chaining with preprocessors like `make_cast`, `make_impute`, `make_clamp` and `make_resize`.

This chaining design gives the core data structures the capability to represent flexible computations.
