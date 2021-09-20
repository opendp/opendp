.. _opendp-commons:

OpenDP Commons
==============

The OpenDP Commons is a community-driven layer of OpenDP based on a common differential privacy library.
It consists of tools and packages for building end-to-end differentially private systems.
The governance for this layer facilitates contributions and vetting by the community,
as well as reviews, guidance, and guarantees for using the library and tools.

Please :ref:`contact us <contact>` if you are looking into building tools with OpenDP.

The diagram below illustrates how the OpenDP library is the foundation of the OpenDP Commons and how various tools are built on top.

|opendp-cake|

.. |opendp-cake| image:: /_static/images/opendp-cake.svg
   :class: img-responsive


We've listed projects in the OpenDP commons below.

OpenDP Library
--------------

The OpenDP library is at the core of the OpenDP project, implementing the framework described in the paper "`A Programming Framework for OpenDP`_".

.. _A Programming Framework for OpenDP: https://projects.iq.harvard.edu/files/opendp/files/opendp_programming_framework_11may2020_1_01.pdf

It is written in Rust and has bindings for Python.

The OpenDP library is currently under development and the source code can be found at https://github.com/opendp/opendp

DP Creator
----------

DP Creator is a web-based application to budget workloads of statistical queries for public release.

Integration with Dataverse repositories will allow researchers with knowledge of their datasets to calculate DP statistics without requiring expert knowledge in programming or differential privacy.

DP Creator is currently under development and the source code can be found at https://github.com/opendp/dpcreator

Related Projects
================


`Google Differential Privacy`_ consists of libraries to generate ε- and (ε, δ)-differentially private statistics over datasets.

.. _Google Differential Privacy: https://github.com/google/differential-privacy


For additional related projects see "Available Tools for Differentially Private Analysis" in `Appendix C`_ of the `Handbook on Using Administrative Data for Research and Evidence-based Policy`_.

.. _Appendix C: https://admindatahandbook.mit.edu/book/v1.0/diffpriv.html#diffpriv-appendixc
.. _Handbook on Using Administrative Data for Research and Evidence-based Policy: https://admindatahandbook.mit.edu


Past Projects
=============

Many of the people involved with OpenDP are also- or have been- involved with the `Harvard Privacy Tools Project <https://privacytools.seas.harvard.edu/>`_,
and have experience building earlier iterations of differential privacy libraries.

* `psilence (R package) and PSI (Private Sharing Interface) <https://github.com/privacytoolsproject/PSI-Library>`_
* `SmartNoise-Core <https://github.com/opendp/smartnoise-core>`_

We don't recommend using these libraries for new projects,
but we have gained much in the process of building them.
