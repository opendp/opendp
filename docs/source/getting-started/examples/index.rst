.. _examples:

More Examples
=============

The Polars API we've used up to this point is only available for Python.

The examples through the rest of the documentation use a lower-level API
which is closer to the underlying Rust implementation.
Although it is more verbose, it has the advantage of being available for Python, Rust, and R.
We are updating our examples to cover all three languages,
but even where we haven't yet completed the translation,
Rust and R users should be able to follow the Python examples.

.. _notebooks:

Python Notebooks
----------------

These introductory notebooks can be run locally, on Binder, or Colab:

.. toctree::
  :titlesonly:

  deeper-look
  pums-data-analysis
  unknown-dataset-size
  histograms

The :doc:`../../theory/index` section also provides several notebooks.


API Examples
------------

There are many useful examples in API docs:

* DP-sum in :ref:`chaining` (user guide)
* count in :class:`opendp.mod.Transformation` (API docs)
* DP-mean in :class:`opendp.mod.Measurement` (API docs)


Rust Examples
-------------

See the `OpenDP Rust documentation <https://docs.rs/opendp/>`_ for examples on how to use the Rust API.

The Rust API is nearly identical to the Python API,
except that explicit type arguments in the Python API are generics.

There is also a `tiny demo crate <https://github.com/opendp/opendp/commit/8561d7e57e960eb72fffa9f24e2dbe54bb6084bb>`_ for wiring up your own FFI language bindings.


Applications built with OpenDP
------------------------------

* `SmartNoise SQL <https://github.com/opendp/smartnoise-sdk/tree/main/sql>`_ enables differentially private SQL queries.
* `SmartNoise Synthesizers <https://github.com/opendp/smartnoise-sdk/tree/main/synth>`_ help you create differentially private synthetic data.
* `DPCreator <https://github.com/opendp/dpcreator>`_ is a web app which guides you through the process of making a differentially private release.

Please :ref:`let us know <contact>` if you have an application to add to this list.
