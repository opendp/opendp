.. _limitations:

Limitations
===========

Please be aware of the following limitations when using OpenDP.

Privacy Concerns
----------------

The DP literature sometimes does not consider the limitations of physical computers.
Two particular areas to be aware of are **idealized numerics** and **side-channel attacks**.

* **Idealized numerics:** Some OpenDP Transformations and Measurements assume an idealized model of real-number arithmetic
  (as is common in the differential privacy research literature).
  Implementations using finite data types like floating-point numbers have `known issues <https://salil.seas.harvard.edu/publications/widespread-underestimation-sensitivity-differentially-private-libraries-and-how>`_,
  where differential privacy is not satisfied due to discrepancies between real-number arithmetic and finite arithmetic.
  To use these less secure APIs, the ``idealized-numerics`` flag must be enabled.
  Through the ongoing process of vetting privacy proofs (see below), we clearly distinguish such mechanisms from ones
  whose concrete implementations faithfully satisfy differential privacy.
* **Side-channel attacks:** OpenDP has not been hardened against side-channel attacks.
  These include timing attacks, cache effects, etc.
  These may make it possible for an attacker who interacts with the system running OpenDP software to obtain information outside
  the intended interfaces, potentially violating differential privacy.

Please carefully consider the implications of these limitations if you are building a privacy-critical application.

Incomplete Privacy Proofs
-------------------------

An important element of the OpenDP Project is a formal vetting process that all library components must undergo, to verify their privacy characteristics.
This process involves supplying mathematical proofs of the privacy properties of all algorithms and validating that all code faithfully implements
the specified algorithms.

Through the vetting process, we expect to uncover bugs in code and proofs and make corrections to components to ensure they satisfy the specified
privacy-relevant properties.
As components complete the vetting process, they will no longer require the ``contrib`` flag.

API Stability
-------------

OpenDP follows `semantic versioning <https://semver.org>`_,
and until we release version 1.0.0, OpenDP APIs are subject to change.
Release notes will include migration instructions, when we do make backwards-incompatible changes.
As the API continues to evolve, we appreciate your feedback about what does and doesn't work.

Software Quality
----------------

Like any software project, OpenDP has bugs. If you encounter problems, `please let us know <https://github.com/opendp/opendp/issues>`_, and we will respond quickly.
