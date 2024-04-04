:orphan:
:nosearch:

TODO: This is the documentation we want, when polars is available.

Quickstart
==========

Once you've installed OpenDP, you can write your first program.
In this example we'll make a private release from a `very` small dataset:

.. code:: python

    # TODO
    # >>> import opendp.prelude as dp
    # >>> import polars as pl
    # >>> pf = dp.PrivateFrame(pl.DataFrame(
    # ...     "age": [20, 40, 60]
    # ... }))
    # >>> pf.select(
    # ...     pl.col("age").private_mean()
    # ... ) # doctest: +ELLIPSIS
    # shape: (1, 1)
    # ┌──────┐
    # │ age  │
    # │ ---  │
    # │ i64  │
    # ╞══════╡
    # │ ...  │
    # └──────┘

You should get a number that is near, but probably not equal to, 40,
and if you re-evaluate you will probably get a different number.

Let's look at each line:

OpenDP's TODO:py:class:`PrivateFrame` wraps a
`DataFrame <https://pola-rs.github.io/polars/py-polars/html/reference/dataframe/index.html>`_ from Polars.
You can use any Polars method to initialize the DataFrame:
For this example the data is inline, but it could also be read from disk or from a URL.

The ``PrivateFrame`` offers a ``select`` analogous to the DataFrame ``select``.
The difference is that the expression provided must end with a private release method.
While Polars supports ``mean``, attempting the same thing in a ``PrivateFrame`` will cause an error:

.. code:: python

    # TODO
    # >>> pf.select(
    # ...     pl.col("age").mean()
    # ... )
    # Traceback:
    # ...
    # Last method in chain must be a private_* method

The ``PrivateFrame`` protects you from accidentally releasing data which is not differentially private.

Typically, you won't provide your data inline, and you'll want to know more than the mean.
Our next example looks at what else is possible.