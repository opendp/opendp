
Hello, OpenDP!
==============

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
