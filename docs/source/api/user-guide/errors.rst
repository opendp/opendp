Errors
======

OpenDP is designed to prevent operations which would violate the guarantees of differential privacy.
This means there are a lot of potential errors, some more clear than others.
This is not a complete list, but it does describe some situations,
and tries to provide more context than a short error message can accomodate.

``'by' kwarg must be a sequence type``
----------------------------------

Confirm that you aren't using a string for the ``by`` kwarg by mistake.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> import polars as pl
        >>> lf = pl.LazyFrame({"a_column": [1, 2, 3, 4]})
        >>> dp.Context.compositor(
        ...     data=lf,
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ...     split_evenly_over=1,
        ...     margins=[
        ...         dp.polars.Margin(by="a_column"),
        ...     ],
        ... )
        Traceback (most recent call last):
        ...
        ValueError: 'by' kwarg must be a sequence type; Did you mean ["a_column"]? https://docs.opendp.org/en/v.../api/user-guide/errors.html#by-kwarg-must-be-a-sequence-type
        
        >>> context = dp.Context.compositor(
        ...     data=lf,
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ...     split_evenly_over=1,
        ...     margins=[
        ...         # "by" should be a list of column names or expressions:    
        ...         dp.polars.Margin(by=["a_column"])
        ...     ],
        ... )
