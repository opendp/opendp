Core Structures
===============

.. contents:: |toctitle|
    :local:

Overview
--------

OpenDP is focused on creating computations with specific privacy characteristics. These computations are modeled with two core structures in OpenDP: Transformations and Measurements. These structures are in all OpenDP programs, regardless of the underlying algorithm or definition of privacy. By modeling computations in this abstract way, we're able to combine them in flexible arrangements and reason about the resulting programs.

Measurement
-----------

A Measurement is a randomized mapping from datasets to outputs of an arbitrary type.

Transformation
--------------

A Transformation is a (deterministic) mapping from datasets to datasets.

Constructors and Functions
--------------------------

In OpenDP, Measurements and Transformations are created by calling constructor functions. Because Measurements and Transformations are themselves like functions (they can be invoked on an input and return an output), you can think of constructors as higher-order functions: You call them to produce another function, that you will then feed some data.

Examples
--------
