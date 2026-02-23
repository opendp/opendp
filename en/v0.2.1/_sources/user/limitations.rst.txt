Limitations
===========

.. contents:: |toctitle|
   :local:

Overview
--------

OpenDP is early in its development history. We aim to build *the* trusted implementation of differential privacy algorithms, but we're not there yet. We feel that OpenDP already can be used to build some useful applications, but please be aware of the following limitations.

Privacy Concerns
----------------

As a work in progress, OpenDP has some known privacy concerns. These vulnerabilities can have a significant impact on some applications, so you should carefully consider them when using OpenDP.

* Floating point issues: There are situations where OpenDP generates noise using native machine representations. This has the known issue of porous floating point output, resulting in `attacks <https://www.microsoft.com/en-us/research/wp-content/uploads/2012/10/lsbs.pdf>`_ where privacy can be violated.

* Side channel attacks: OpenDP is not robust to some side channel attacks. These include things like timing attacks, cache effects, etc. These may make it possible for an attacker to obtain information outside the intended interfaces of OpenDP functions, violating privacy.

We know that these issues are critical for a privacy library, and are formulating plans to address them. Until then, *we strongly recommend that you not yet use OpenDP for any privacy-critical applications*.

Incomplete Privacy Proofs
-------------------------

An important element of the OpenDP project is a formal vetting process that all library components must undergo, to verify their privacy characteristics. This process involves supplying mathematical proofs of the privacy properties of all algorithms and validating that all code faithfully implements the specified algorithms.

For the most part, the code in OpenDP hasn't yet undergone that vetting. This process is underway, but will take time. Therefore, we can't yet properly vouch for the privacy claims of the implemented algorithms. If mathematical verifiability is important to you, then you shouldn't yet rely on the code in OpenDP.

Missing Functionality
---------------------

There are some elements of the OpenDP Programming Framework, the conceptual basis for OpenDP, that are not yet implemented. Foremost among these is the concept of interactive measurements. (There is an initial prototype, implemented in Rust only, but it's not fully fleshed out). These elements are on our roadmap, but applications will have to wait to use them.

API Stability
-------------

The APIs of OpenDP are still in flux, and subject to change. We've iterated a lot on these interfaces, and feel like the general shape is working well, but it's likely that many details will change in backwards-incompatible ways. This
early flexibility is important to evolve the library, and we appreciate your understanding and feedback on improvements.

When we release OpenDP 1.0.0, we'll begin to offer API stability within `major versions <https://semver.org>`_. Until then, please don't depend on interfaces remaining the same across releases of OpenDP.

Software Quality
----------------

Like any software project, OpenDP likely has bugs. Because it's still an early project, it likely has many bugs! We strive to make high quality software, but if you encounter issues, `please let us know <https://github.com/opendp/opendp/issues>`_.
