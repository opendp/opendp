OpenDP Commons
==============

**Contents:**

.. contents:: |toctitle|
	:local:

What is the OpenDP Commons?
---------------------------

The OpenDP Commons is a community-driven layer of OpenDP includes a common differential privacy library and common tools and packages that can be used to build end-to-end differentially private systems. The governance for this layer facilitates contributions and vetting by the community, as well as reviews, guidance, and guarantees for using the library and tools.

The diagram below illustrates how the OpenDP library is the foundation of the OpenDP Commons and how various tools are built on top.

|opendp-cake|

.. |opendp-cake| image:: ../../_static/images/opendp-cake.svg
   :class: img-responsive


List of OpenDP Commons Tools and Packages
-----------------------------------------

OpenDP Library
++++++++++++++

The OpenDP library is at the core of the OpenDP project, implementing the framework described in the paper "`A Programming Framework for OpenDP`_".

.. _A Programming Framework for OpenDP: https://projects.iq.harvard.edu/files/opendp/files/opendp_programming_framework_11may2020_1_01.pdf

It is written in Rust and has bindings for Python.

The OpenDP library is currently under development and the source code can be found at https://github.com/opendp/opendp

DP Creator
++++++++++

DP Creator is a web-based application to budget workloads of statistical queries for public release.

Integration with Dataverse repositories will allow researchers with knowledge of their datasets to calculate DP statistics without requiring expert knowledge in programming or differential privacy.

DP Creator is currently under development and the source code can be found at https://github.com/opendp/dpcreator
