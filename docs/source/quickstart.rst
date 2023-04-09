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
In the example below, we'll construct a Laplace mechanism of type :class:`opendp.mod.Measurement`, 
then invoke it on a scalar aggregate.

.. doctest::

    >>> from opendp.measurements import make_base_laplace
    ...
    >>> base_laplace = make_base_laplace(scale=1.)
    >>> base_laplace(23.4) # doctest: +SKIP
    22.74877695423367

If you would like to skip directly to a more complete example, see :ref:`putting-together`.

Otherwise, continue on to the User Guide.
