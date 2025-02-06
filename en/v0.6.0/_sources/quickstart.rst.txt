Quickstart
==========

The easiest way to get started with OpenDP is from Python.
Use ``pip`` to install the `opendp <https://pypi.org/project/opendp/>`_ package from PyPI.

.. prompt:: bash

    pip install opendp

This will make the OpenDP modules available to your local environment.

The vetting process is currently underway for the code in the OpenDP Library.
Any code that has not completed the vetting process is marked as "contrib" and will not run unless you opt-in.
Enable ``contrib`` globally with the following snippet:

.. doctest::

    >>> from opendp.mod import enable_features
    >>> enable_features('contrib')


Hello, OpenDP!
--------------

Once you've installed OpenDP, you can write your first program.
In the example below, we'll construct an identity :class:`opendp.mod.Transformation`, then invoke it on a dataset of strings.

.. doctest::

    >>> from opendp.transformations import make_identity
    >>> from opendp.typing import SymmetricDistance, VectorDomain, AllDomain
    ...
    >>> identity = make_identity(D=VectorDomain[AllDomain[str]], M=SymmetricDistance)
    >>> identity(["Hello, world!"])
    ['Hello, world!']

There's a more thorough explanation in the :ref:`Getting Started <hello-opendp>` section.

If you would like to skip directly to a more complete example, see :ref:`putting-together`.

Otherwise, continue on to the User Guide.
