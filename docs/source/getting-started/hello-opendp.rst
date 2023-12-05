
Hello, OpenDP!
==============

Once you've installed OpenDP, you can write your first program.
In this example we'll make a private release from a `very` small dataset:

.. doctest::

    >>> import opendp.prelude as dp
    >>> import polars as pl
    >>> lf = dp.LazyFrame(pl.DataFrame(
    ...     "age": [20, 40, 60]
    ... }))
    >>> lf.select(
	...     pl.col("age").private_mean()
    ... )
    shape: (1, 1)
    ┌──────┐
    │ age  │
    │ ---  │
    │ i64  │
    ╞══════╡
    │ ...  │
    └──────┘

You should get a number that is near, but probably not equal to, 40,
and if you re-evaluate you will probably get a different number.

Let's look at each line:

OpenDP's :py:class:`LazyFrame` wraps a
`DataFrame <https://pola-rs.github.io/polars/py-polars/html/reference/dataframe/index.html>`_ from Polars.
You can use any Polars method to initialize the DataFrame:
For this example the data is inline, but it could also be read from disk or from a URL.

The LazyFrame offers a ``select`` analogous to the DataFrame ``select``.
The difference is that the expression provided must end with a private release method.
While Polars natively supports ``mean``, attempting the same thing in a LazyFrame will cause an error:

.. doctest::

    >>> lf.select(
	...     pl.col("age").mean()
    ... )
    Traceback:
    ...
    Last method in chain must be a private_* method

The ``LazyFrame`` protects you from accidentally releasing data which is not differentially private.

Typically, you won't provide your data in line, and you'll want to know more than the mean.
Our next example looks at what else is possible.