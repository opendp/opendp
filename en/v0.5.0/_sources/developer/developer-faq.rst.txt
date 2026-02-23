.. _developer-faq:

Developer Frequently Asked Questions
====================================

This is a list of common issues developers may encounter.
The general :ref:`faq` will be more relevant if you are not a developer.

Compilation Error: Windows GMP
------------------------------
If you are on a Windows platform and get the error:

.. code-block:: bash

    thread 'main' panicked at 'Program failed with code 2: "make" "-j" "12" "check"'

Refer to `the windows build instructions <https://github.com/opendp/opendp/tree/main/rust/windows>`_.


Compilation Error: "error[E0658]: const generics are unstable"
--------------------------------------------------------------
You have an out-of-date Rust compiler.

.. prompt:: bash

    rustup update


Runtime Error: "No match for concrete type. You've got a debug binary!"
-----------------------------------------------------------------------
If the built FFI library doesn't support a data type, you get the error:

.. code-block:: text

    No match for concrete type {} ({:?}). You've got a debug binary!

The FFI layer creates many copies of generic functions with specific concrete data types (monomorphization).
When you call the FFI layer with a concrete type argument,
the FFI layer invokes the copy of the generic function that was monomorphized with that type.

In order to speed the development process,
FFI library debug builds monomorphize a smaller set of types than a release build.

#. The easy solution is to build the library with the additional ``--release`` flag.
   If you are using python, you will also need to set ``OPENDP_TEST_RELEASE=1`` before importing opendp.

#. If you are doing significant debugging on an algorithm, you could instead choose one of the below to speed things up:

    #. Don't build the FFI crate, only build the OpenDP crate.
       Do your debugging via tests or make your own tiny crate that depends on OpenDP.

    #. If you need FFI to debug, consider changing your data type to one of the debug data types in `dispatch.rs <https://github.com/opendp/opendp/blob/main/rust/opendp-ffi/src/dispatch.rs>`_,
       or temporarily adjust the FFI dispatch for your constructor.


Runtime Error: "AttributeError: module 'enum' has no attribute 'IntFlag'"
-------------------------------------------------------------------------
`This solution <https://stackoverflow.com/a/45716067>`_ resolves the root issue.

.. prompt:: bash

    pip3 uninstall -y enum34
