Installation
============

This section will walk you through the process of getting started with OpenDP. By the end, you should have a working OpenDP installation, and you'll be ready to explore OpenDP programming.

OpenDP is built with a layered architecture. The core consists of a native library, implemented in Rust. On top of this core are bindings for using OpenDP from other languages (currently only Python, but we hope to add bindings for more languages in the future).

In order to use OpenDP in your applications, the first step is installing the library so that it's accessible to your local environment. Depending on how you intend to use OpenDP, these steps may vary.

Platforms
^^^^^^^^^

OpenDP is built for the following platforms:

* Python 3.8-3.11
* Linux, x86 64-bit, versions compatible with `manylinux <https://github.com/pypa/manylinux>`_ (others may work)
* macOS, x86 64-bit, version 10.13 or later
* Windows, x86 64-bit, version 7 or later

Other platforms may work, but will require `building from source <#building-opendp-from-source>`_.

Installing OpenDP for Python from PyPI
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

OpenDP is distributed as a `Python package on PyPI <https://pypi.org/project/opendp/>`_. You can install it using ``pip`` or another package management tool:

.. prompt:: bash

    pip install opendp

.. note::

    The OpenDP Python package contains binaries for all supported platforms, so you don't need to worry about specifying a specific platform.

At this point, you should be good to go! You can confirm your installation in Python by importing the top-level module ``opendp``:

.. doctest::

    >>> import opendp

Installing OpenDP for Rust from crates.io
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

OpenDP is also available as a `Rust crate <https://crates.io/crates/opendp>`_.
This is not as common, as most people will use OpenDP through one of the other language bindings.
But if you need to use the Rust interface of OpenDP directly, you can just reference the ``opendp`` crate in your ``Cargo.toml`` file:

.. code-block:: toml

    [dependencies]
    opendp = { version = "0.1.0", features = ["contrib"] }

.. note::

    The actual version may differ depending on the `releases available <https://github.com/opendp/opendp/releases>`_.

In the above snip, opting into the "contrib" feature includes code that has not yet completed the vetting process.
Continuous noise samplers require explicit opt-in and are behind the "floating-point" feature.

With that configured, the Rust dependency system will automatically download the crate as needed, and you can just ``use`` the ``opendp`` module:

.. code-block:: rust

    use opendp::core::*;
    // OpenDP code goes here!

Building OpenDP from Source
^^^^^^^^^^^^^^^^^^^^^^^^^^^

Under special circumstances, you may want to install OpenDP directly from the source files.
This is only required if you want to build OpenDP from scratch, 
or if you're interested in :doc:`writing Rust code for OpenDP <../contributor/index>`.

There is a thorough guide to building from source in the :doc:`Development Environment <../contributor/development-environment>` documentation.

What's Next?
------------

The next section of this guide will walk you through the conceptual underpinning of OpenDP, known as the :doc:`OpenDP Programming Framework <programming-framework/index>`.

If you're eager to just jump in to programming, then :doc:`get started with the OpenDP library <getting-started>`.

For those who prefer to study reference material, you can consult the :doc:`API Docs <../api/index>`.
