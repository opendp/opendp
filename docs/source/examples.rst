.. _examples:

Examples
========

Python Docs
-----------

There are a few useful examples embedded in the user guide and API docs:

* Don't miss the :ref:`conclusion of the user-guide <putting-together>`!
* base_discrete_laplace in :ref:`constructors` (user guide)
* DP-sum in :ref:`chaining` (user guide)
* count in :class:`opendp.mod.Transformation` (API docs)
* DP-mean in :class:`opendp.mod.Measurement` (API docs)

.. _notebooks:

Python Notebooks
----------------

Example notebooks can be found in `python/example`_ in the OpenDP library source tree.

.. _python/example: https://github.com/opendp/opendp/tree/main/python/example

You can open and run these notebooks online in Binder_ (navigate to ``python/example``).

.. _Binder: https://mybinder.org/v2/gh/opendp/opendp/HEAD

Rust Examples
-------------

See the `OpenDP rust documentation <https://docs.rs/opendp/>`_ for examples on how to use the Rust API.

The Rust API is nearly identical to the Python API,
except that explicit type arguments in the python API are generics.

There is also a `tiny demo crate <https://github.com/opendp/opendp/commit/8561d7e57e960eb72fffa9f24e2dbe54bb6084bb>`_ for wiring up your own FFI language bindings.


Applications
------------

DPCreator, a part of the :ref:`opendp-commons`, is based on the OpenDP library.

Please :ref:`let us know <contact>` if you are building applications with the library.
