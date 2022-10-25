.. _limitations:

Limitations
===========

OpenDP is early in its development history.
We aim to work with the community to make it *the* trustworthy implementation of differential privacy algorithms, but that will take some time.
OpenDP already can be used to build some applications, but please be aware of the following limitations when deploying it.

Privacy Concerns
----------------

As a work in progress, OpenDP has some known privacy concerns.
These vulnerabilities may have an impact on some applications, depending on how they operate.

* **Floating point issues:** Some of the OpenDP privacy mechanisms are based on assuming an idealized model of real-number arithmetic
  (as is common in the differential privacy research literature).
  Their implementations using floating-point numbers have `known issues <https://www.microsoft.com/en-us/research/wp-content/uploads/2012/10/lsbs.pdf>`_,
  where the differential privacy property is not satisfied due to discrepancies between real-number arithmetic and floating-point arithmetic.
  Through the ongoing process of vetting privacy proofs (see below), we plan to clearly distinguish such mechanisms from ones
  whose concrete implementations faithfully satisfy differential privacy (e.g. discrete noise-addition mechanisms).
  See :ref:`floating-point` for additional comments.
* **Side-channel attacks:** OpenDP has not yet been hardened against side-channel attacks.
  These include things like timing attacks, cache effects, etc.
  These may make it possible for an attacker who interacts with the system running OpenDP software to obtain information outside
  the intended interfaces, potentially violating differential privacy.

Please carefully consider the implications of these limitations if you are building a privacy-critical application.

Incomplete Privacy Proofs
-------------------------

An important element of the OpenDP Project is a formal vetting process that all library components must undergo, to verify their privacy characteristics.
This process involves supplying mathematical proofs of the privacy properties of all algorithms and validating that all code faithfully implements
the specified algorithms.

This vetting process is currently underway for the code in the OpenDP Library.
Through the vetting process, we expect to uncover bugs in code and proofs and make corrections to components to ensure they satisfy the specified
privacy-relevant properties.
As components complete the vetting process, they will drop the ``contrib`` feature flag, and will be accessible without explicitly opting into
unvetted components.

Missing Functionality
---------------------

There are some elements of the OpenDP Programming Framework, the conceptual basis for OpenDP, that are not yet implemented.
Foremost among these is the concept of interactive measurements.
(There is an initial prototype, implemented in Rust only, but it's not fully fleshed out).
These elements are on our roadmap, but applications will have to wait to use them.

API Stability
-------------

The APIs of OpenDP are still in flux, and subject to change.
We've iterated a lot on these interfaces, and feel like the general shape is working well, but it's likely that many details will change in backwards-incompatible ways.
This early flexibility is important to evolve the library, and we appreciate your understanding and feedback on improvements.

When we release OpenDP 1.0.0, we'll begin to offer API stability within `major versions <https://semver.org>`_.
Until then, please don't depend on interfaces remaining the same across releases of OpenDP.

Software Quality
----------------

Like any software project, OpenDP likely has bugs. Because it's still an early project, it likely has many bugs!
We strive to make high quality software, but if you encounter issues, `please let us know <https://github.com/opendp/opendp/issues>`_.
