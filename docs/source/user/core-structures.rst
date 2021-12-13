.. _core-structures:

Core Structures
===============

OpenDP is focused on creating computations with specific privacy characteristics.
These computations are modeled with two core structures in OpenDP:
:py:class:`opendp.mod.Transformation` and :py:class:`opendp.mod.Measurement`.
These structures are in all OpenDP programs, regardless of the underlying algorithm or definition of privacy.
By modeling computations in this abstract way, we're able to combine them in flexible arrangements and reason about the resulting programs.

A unifying perspective towards OpenDP is that OpenDP is a system for `relating`:

#. an upper bound on distance between neighboring function inputs (to)
#. an upper bound on distance between respective function outputs (or distributions)

OpenDP naturally captures the definition of privacy via :ref:`relations <relations>`,
because the definition of privacy is an upper bound on distance between probability distributions.

Each measurement or transformation is a self-contained structure with a relation, function, and supporting proof.


.. _measurement:

Measurement
-----------

A :py:class:`Measurement <opendp.mod.Measurement>` is a randomized mapping from datasets to outputs of an arbitrary type.
Lets say we have an arbitrary instance of a Measurement, called ``meas``, and a code snippet ``meas.check(d_in, d_out)``.
If the code snippet evaluates to True, then ``meas`` is ``d_out``-DP on ``d_in``-close inputs,
or equivalently "(``d_in``, ``d_out``)-close".
The code snippet simply checks the privacy relation that comes bundled inside ``meas``.

The distances ``d_in`` and ``d_out`` are expressed in the units of the input metric and output measure.
Depending on the context, ``d_in`` could be a distance bound to neighboring datasets or a global sensitivity,
and ``d_out`` may be ``epsilon``, ``(epsilon, delta)``, or some other measure of privacy.
More information on distances is available :ref:`here <distances>`.

Each invocation of the measurement's function (via ``meas.invoke(data)`` or ``meas(data)``) is a differentially private release.
The privacy expenditure of each release is any ``d_out`` that makes the relation pass.

A measurement structure contains the following internal fields:

:input_domain: A :ref:`domain <domains>` that describes the set of all possible input values for the function.
:output_domain: A :ref:`domain <domains>` that describes the set of all possible output values of the function.
:function: A :ref:`function <functions>` that computes a differentially-private release on private data.
:input_metric: A :ref:`metric <metrics>` used to compute distance between two members of the input domain.
:output_measure: A :ref:`measure <measures>` used to measure distance between two distributions in the output domain.
:privacy_relation: A :ref:`relation <relations>` that encapsulates the privacy characteristics of the function.

The framework quantifies output distance bounds on measurements with a measure, instead of a metric,
because measurements emit samples from a probability distribution,
and measures can be used to quantify differences between probability distributions.
This is the primary differentiating factor between measurements and transformations.

.. _transformation:

Transformation
--------------

A :py:class:`Transformation <opendp.mod.Transformation>` is a (deterministic) mapping from datasets to datasets.
Transformations are used to preprocess and aggregate data before chaining with a measurement.

Similarly to ``meas`` above, let's say we have an arbitrary instance of a Transformation, called ``trans``,
and a code snippet ``trans.check(d_in, d_out)``.
If the code snippet evaluates to True, then ``trans`` is ``d_out``-close on ``d_in``-close inputs,
or equivalently "(``d_in``, ``d_out``)-close".
The code snippet simply checks the stability relation that comes bundled inside ``trans``.
In this context, the relation captures the stability of a transformation.

The distances ``d_in`` and ``d_out`` are expressed in the units of the input metric and output metric.
Depending on the context, ``d_in`` and ``d_out`` could be a distance bound to neighboring datasets or a global sensitivity.
More information on distances is available :ref:`here <distances>`.

Invoking the function (via ``trans.invoke(data)`` or ``trans(data)``) transforms the data, but the output is not differentially private.
Transformations need to be :ref:`chained <chaining>` with a measurement before they can be used to create a differentially-private release.

A transformation structure contains the following internal fields:

:input_domain: A :ref:`domain <domains>` that describes the set of all possible input values for the function.
:output_domain: A :ref:`domain <domains>` that describes the set of all possible output values of the function.
:function: A :ref:`function <functions>` that transforms data.
:input_metric: A :ref:`metric <metrics>` used to compute distance between two members of the input domain.
:output_metric: A :ref:`metric <metrics>` used to measure distance between two members of the output domain.
:stability_relation: A :ref:`relation <relations>` that encapsulates the stability characteristics of the function.

.. _constructors:

Constructors and Functions
--------------------------

In OpenDP, Measurements and Transformations are created by calling constructor functions.
The majority of the library's interface consists of these constructors.

Because Measurements and Transformations are themselves like functions (they can be invoked on an input and return an output),
you can think of constructors as higher-order functions:
You call them to produce another function that you will then feed data.

There's a few top-level constructor listings:

* :ref:`combinator-constructors`
* :ref:`measurement-constructors`
* :ref:`transformation-constructors`

In this simplified example with the :py:func:`opendp.meas.make_base_discrete_laplace` constructor, we assume the data was properly preprocessed and aggregated such that the sensitivity (by absolute distance) is at most 1.

.. testsetup::

    from opendp.mod import enable_features
    enable_features('contrib')

.. doctest::

    >>> from opendp.meas import make_base_discrete_laplace
    ...
    >>> # call the constructor to produce a measurement
    >>> base_discrete_laplace = make_base_discrete_laplace(scale=1.0)
    ...
    >>> # investigate the privacy relation
    >>> absolute_distance = 1
    >>> epsilon = 1.0
    >>> assert base_discrete_laplace.check(d_in=absolute_distance, d_out=epsilon)
    ...
    >>> # feed some data/invoke the measurement as a function
    >>> aggregated = 5
    >>> release = base_discrete_laplace(aggregated)

