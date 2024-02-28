Getting Started
===============

.. contents:: |toctitle|
    :local:

Installation
------------

The first step in using OpenDP is to install the OpenDP library.

Installing OpenDP from PyPI
^^^^^^^^^^^^^^^^^^^^^^^^^^^

A package for OpenDP is available from `PyPI <https://pypi.org/project/opendp/>`_. You can install it using ``pip`` or other package management tool:

.. code-block:: bash

    % pip install opendp

This will make the OpenDP modules available to your local environment.

Hello, OpenDP!
--------------

Once you've installed OpenDP, you can write your first program. In the example below, we'll construct an identity ``Transformation``, then invoke it on a string.

.. doctest::

    >>> from opendp.trans import make_identity
    >>> from opendp.typing import SubstituteDistance

    >>> identity = make_identity(M=SubstituteDistance, TA=str)
    >>> identity("Hello, world!")
    'Hello, world!'
