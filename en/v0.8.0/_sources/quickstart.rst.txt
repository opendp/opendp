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

    >>> import opendp.prelude as dp
    >>> base_laplace = dp.space_of(float) >> dp.m.then_base_laplace(scale=1.)
    >>> base_laplace(23.4) # doctest: +SKIP
    22.74877695423367

This code snip uses a number of OpenDP concepts:

* Defining your metric space up-front with ``space_of``.
* Chaining operators together with ``>>``.
* Constructing a ``Measurement`` on your metric space with ``m.then_base_laplace``.
* Invoking the ``base_laplace`` measurement on a value to get a DP release.

If you would like to skip directly to a more complete example, see :ref:`putting-together`.

Otherwise, continue on to the User Guide, where these concepts are explained in detail.
