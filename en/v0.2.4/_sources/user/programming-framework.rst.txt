Programming Framework
=====================

OpenDP is based on a conceptual model that defines the characteristics of privacy-preserving operations and provides a way for components to be assembled into programs with desired behavior. This model, known as the OpenDP Programming Framework, is described in the paper `A Programming Framework for OpenDP <https://projects.iq.harvard.edu/files/opendp/files/opendp_programming_framework_11may2020_1_01.pdf>`_. The framework is designed with a clear and verifiable means of capturing the sensitive aspects of an algorithm, while remaining highly flexible and extensible. OpenDP (the software library) is intended to be a faithful implementation of that approach. Because OpenDP is based on a well-defined model, users can create applications with rigorous privacy properties.

Summary
-------

The OpenDP Programming Framework consists of a set of high-level conceptual elements. We'll cover the highlights here, which should be enough for you to get acquainted with OpenDP programming. If you're interested in more of the details and motivations behind the framework, you're encouraged to read `the paper <https://projects.iq.harvard.edu/files/opendp/files/opendp_programming_framework_11may2020_1_01.pdf>`_.

.. note::

    In this section, we've used lower case when writing the names of OpenDP concepts. Later, when we talk about :doc:`programming elements <core-structures>`, we'll use the capitalized form to refer to the concrete data types that implement these concepts. (The concept names link to their corresponding type descriptions.)

* :ref:`Measurements <measurement>` are randomized mappings from a dataset to an arbitrary output value. They are a controlled means of introducing privacy (e.g. noise) to a computation. An example of a measurement is one which applies Laplace noise to a value.
* :ref:`Transformations <transformation>` are deterministic mappings from a dataset to another dataset. They are used to summarize or transform values in some way. An example of a transformation is one which calculates the mean of a set of values.
* :ref:`Domains <domains>` are sets which identify the possible values that some object can take. They are used to constrain the input or output of measurements and transformations. Examples of domains are the integers between 1 and 10, or vectors of length 5 containing floating point numbers.
* :ref:`Measures <measures>` and :ref:`metrics <metrics>` are things that specify distances between two mathematical objects.

  * Measures characterize the distance between two probability distributions. An example measure is the "max-divergence" of pure differential privacy.
  * Metrics capture the distance between two neighboring datasets. An example metric is "symmetric distance" (counting the number of elements changed).

* :ref:`Privacy relations and stability relations <relations>` are boolean functions which characterize the notion "closeness" of operation inputs and outputs. They are the glue that binds everything together.

  * A privacy relation is a statement about a measurement. It's a boolean function of two values, an input distance (in a specific metric) and an output distance (in a specific measure). A privacy relation lets you make assertions about a measurement when the measurement is evaluated on any pairs of neighboring datasets. If the privacy relation is true, it's guaranteed that any pair of measurement inputs within the input distance will always produce a pair of transformation outputs within the output distance.
  * A stability relation is a statement about a transformation. It's also a boolean function of two values, an input distance (in a specific metric) and an output distance (in a specific metric, possibly different from the input metric). A stability relation lets you make assertions about the behavior of a transformation when that transformation is evaluated on any pairs of neighboring datasets. If the stability relation is true, it is a guarantee that any pair of transformation inputs within the input distance will always produce transformation outputs within the output distance.

  Relations capture the notion of closeness in a very general way, allowing the extension of OpenDP to different definitions of privacy.

As you can see, these elements are interdependent and support each other. The interaction of these elements is what gives the OpenDP Programming Framework its flexibility and expressiveness.

Key Points
----------

You don't need to know all the details of the Programming Framework to write OpenDP applications, but it helps understand some of the key points:

* OpenDP calculations are built by assembling a measurement from a number of constituent transformations and measurements, typically through chaining or composition.
* Measurements don't have a static privacy loss specified when constructing the measurement. Instead, measurements are typically constructed by specifying the scale of noise, and the loss is bounded by the resulting privacy relation. This requires some extra work compared to specifying the loss directly, but OpenDP provides some utilities to make this easier on the programmer, and the benefit is greatly increased flexibility of the framework as a whole.

Implementation Differences
--------------------------

As a work in progress, it's important to note that OpenDP doesn't yet implement all the details of the Programming Framework.

Interactive Measurements
^^^^^^^^^^^^^^^^^^^^^^^^

An important aspect of the Programming Framework is the flexible way that it models interactive measurements. These are measurements where the operation isn't a static function, but instead captures a series of queries and responses, where the sequence is possibly determined dynamically. This is a very flexible model of computation, and can be used to capture notions such as adaptive composition.

Unfortunately, OpenDP doesn't yet implement interactive measurements, and is limited to plain (non-interactive) measurements. We know this is important functionality, and are in the process of prototyping an implementation, but unfortunately it'll take some time before it's ready for use.

Row Transforms
^^^^^^^^^^^^^^

Row transforms are a way of applying a user-defined function to each of the elements of a dataset. This concept can be used to construct transformations for operations that aren't provided "out of the box" by OpenDP. Unfortunately, supporting row transforms has some privacy limitations around pure functions and also requires some tricky technical work, so these aren't yet implemented in OpenDP.

Applying the Concepts
^^^^^^^^^^^^^^^^^^^^^

This is just a glance at the abstract concepts in the OpenDP Programming Framework. The following sections of this guide describe the actual software components in OpenDP implementing these concepts, and how they can be used in your programs.
