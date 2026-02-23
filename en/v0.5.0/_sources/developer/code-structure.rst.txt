.. _code-structure:

Code Structure
**************
Most library contributions will involve the following components:

.. contents::
    :local:

The requirements to be met for the vetting process are numbered for each component.

Constructor Function
====================

The user guide has a relevant section explaining :ref:`constructors <constructors>`.

#. The constructor is the only mandatory component to implement!
#. If the proof is not written, then your work must be marked by the ``contrib`` flag.
   In Rust, you just need to add an annotation ``#[cfg(feature="contrib")]`` above an element to mark it.
   You can place this flag on your module definition to mark your entire module at once.
   It is possible to submit another pull request at a later time to add a proof that will move your constructor out of ``contrib``.
#. You should place your code inside a folder containing a mod.rs.
   This is to give room to place the proof file adjacent to the implementation.
#. Your source should be accompanied by a test module at the end of the file.
   Test modules are also a great way to play with your constructor before the FFI bindings are available.
#. Format your code nicely (rustfmt), add documentation, and comment meaningfully.

If you are not sure where to start, study some of the other constructors in the library.
If you are still unsure, please ask for help!

Proof
=====
Any constructor that has been merged into the library but does not have a vetted proof is a part of ``contrib``.
Both the Rust library and language bindings have ``contrib`` components disabled by default.
This means the user will have to explicitly opt-in to access your constructor if it is part of ``contrib``.

#. Your proof should show the following:

    #. That the function, when evaluated on any element in the input domain, emits a value in the output domain.
    #. That the map always returns a ``d_out`` for any specified ``d_in`` that is (``d_in``, ``d_out``)-close.
    #. That your choices of metrics/measures are compatible with your domains.

#. Your proof should include pseudocode.
   This pseudocode is necessary for the vetting process:
   We will need to check that the Rust and pseudocode are isomorphic.
#. The proof documents should be located in the same folder as your constructor function.
   At the moment there are many proofs undergoing review `in an adjacent repository <https://github.com/opendp/whitepapers/pulls>`_.
   We will be moving these proof files to be adjacent to the code in the near future,
   but for now they may provide a useful template and definition base for your own proofs.
#. The linked repository contains two definitions files you should reference.
   Going forward we intend to use these shared definitions for all proofs, for consistency.

You may find the more general row transform proof useful.
If your transformation is a row-transform, you can lean on this proof— you
only need to prove that your transformation meets the supporting requirements.

FFI Wrapper
===========
Any constructor that does not have an FFI wrapper will not be available in bindings languages.
At the same time, we acknowledge that writing FFI wrappers can be tricky.
We are working on more automated tooling to generate FFI wrappers, but in the meantime,
a core developer can work with you to write the FFI wrappers, if you write the constructor.

This section only has one requirement:

#. If you have FFI support, then there should also be tests for your constructor in bindings languages.
