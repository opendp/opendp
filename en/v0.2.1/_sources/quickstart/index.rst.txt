Quickstart
==========

.. contents:: |toctitle|
    :local:

Installation
------------

The easiest way to get started with OpenDP is from Python. Use ``pip`` to install the `opendp <https://pypi.org/project/opendp/>`_ package from PyPI.

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
