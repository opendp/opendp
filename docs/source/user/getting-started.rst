Getting Started
===============

.. contents:: |toctitle|
    :local:

Installation
------------

The first step in using OpenDP is to install the OpenDP library.

Installing OpenDP from PiPy
^^^^^^^^^^^^^^^^^^^^^^^^^^^

A package for OpenDP is available from `PyPI <https://pypi.org/project/opendp/>`_. You can install it using ``pip`` or other package management tool:

.. code-block:: bash

    % pip install opendp

This will make the OpenDP modules available to your local environment.

Hello, OpenDP!
--------------

Once you've installed OpenDP, you can write your first program. In the example below, we'll construct an identity ``Transformation``, then invoke it on a string.

::

    import opendp.trans as trans

    identity = make_identity(M="SymmetricDistance", T=str)
    print(identity("Hello, world!")
