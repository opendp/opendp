Contributing Algorithms
=======================

Special care should be taken when contributing algorithms to the OpenDP library. Please follow the steps below.

.. contents:: |toctitle|
	:local:

Write a Proof
-------------

Proofs should be written in a markup language such as LaTeX that can be converted into a PDF and perhaps other formats.

Implement the Algorithm
-----------------------

You can refer to the existing Laplace algorithm as an example:

- rust/opendp/src/meas/laplace.rs
- rust/opendp-ffi/src/meas/laplace.rs

Make a Pull Request
-------------------

Follow :doc:`pull-requests` but also include your proof.
