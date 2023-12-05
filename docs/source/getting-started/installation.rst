Installation
============

The easiest way to get started with OpenDP is from Python.
Use ``pip`` to install the `opendp <https://pypi.org/project/opendp/>`_ package from PyPI.

.. prompt:: bash

    pip install opendp

This will make the OpenDP modules available to your local environment.

Almost all of our examples begin with these lines:

.. doctest::

    >>> import opendp.prelude as dp
    >>> dp.enable_features('contrib')

The first line imports :py:mod:`opendp.prelude` under the conventional name `dp`.

The second line enables features which have not yet been vetted.
Any code that has not completed the vetting process is marked as "contrib" and will not run unless you opt-in.
Our goal is to have all the core methods formally vetted,
but until that is complete we need to explicitly opt-in.

Hello, OpenDP!
--------------

Once you've installed OpenDP, you can write your first program.
In the example below, we'll construct a Laplace mechanism of type :class:`opendp.mod.Measurement`, 
then invoke it on a scalar aggregate.

.. doctest::

    >>> import opendp.prelude as dp
    >>> base_laplace = dp.space_of(float) >> dp.m.then_base_laplace(scale=1.)
    >>> dp_agg = base_laplace(23.4)

This code snip uses a number of OpenDP concepts:

* Defining your metric space up-front with ``space_of``.
* Chaining operators together with ``>>``.
* Constructing a ``Measurement`` on your metric space with ``m.then_base_laplace``.
* Invoking the ``base_laplace`` measurement on a value to get a DP release.

If you would like to skip directly to a more complete example, see :ref:`putting-together`.

Otherwise, continue on to the User Guide, where these concepts are explained in detail.
