.. _examples:

More Examples
=============

Most of the examples that follow use the Framework API in Python.
Although it is more verbose than the Context API,
it has the advantage of being very similar to the Rust and R APIs,
and Rust and R users should be able to follow the Python examples.
We are planning to translate the Python examples to Rust and R.

Cookbook examples
-----------------

These introductory notebooks provide some solutions for real-world problems with OpenDP:

.. toctree::
  :titlesonly:

  pums-data-analysis
  unknown-dataset-size
  histograms

Theory examples
---------------

The :doc:`../../theory/index` section also provides several notebooks.
These are focussed on explaining the mathematics behind differential privacy.


API Examples
------------

Examples in the API documentation are focussed on explaining individual functions.
Some examples include:

* DP-sum in :ref:`chaining` (user guide)
* count in :class:`opendp.mod.Transformation` (API docs)
* DP-mean in :class:`opendp.mod.Measurement` (API docs)


Rust Examples
-------------

See the `OpenDP Rust documentation <https://docs.rs/opendp/>`_ for examples on how to use the Rust API.

The Rust API is nearly identical to the Python Framework API,
except that explicit type arguments in the Python API are generics.


Applications built with OpenDP
------------------------------

* `SmartNoise SQL <https://github.com/opendp/smartnoise-sdk/tree/main/sql>`_ enables differentially private SQL queries.
* `SmartNoise Synthesizers <https://github.com/opendp/smartnoise-sdk/tree/main/synth>`_ help you create differentially private synthetic data.
* `DPCreator <https://github.com/opendp/dpcreator>`_ is a web app which guides you through the process of making a differentially private release.

Please :ref:`let us know <contact>` if you have an application to add to this list.
