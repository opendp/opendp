"""
This module requires extra installs: ``pip install 'opendp[sql]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sql``.

.. caution::

    This is a very early release, and is intended primarily to solicit feedback.
    It supports only a limited set of polars expressions,
    and in the future the API may change.
"""
__all__ = ['execute_on_database', 'get_connection', 'scan_database']


def get_connection(database_name: str, **kwargs):
    """
    Returns a database connection object.

    .. note:

        An `Ibis backend <https://ibis-project.org/install>`_
        must be installed for the database you will target.
        For sqlite, for example:

        .. code:: shell

            $ pip install 'ibis-framework[sqlite]'

    :param database_name: A supported database type. "sqlite", for example.
    :param kwargs: Connection parameters.
    """
    import ibis  # type: ignore[import-untyped]

    # Avoid a direct ibis call in user code.
    return getattr(ibis, database_name).connect(**kwargs)


def scan_database(connection, table_name: str):
    """
    Given a database table, returns a polars schema.

    :param connection: A connection object
    :param table_name: The name of the database table to scan
    """
    from polars_to_ibis import scan_database  # type: ignore[import-not-found]

    # Avoid a direct polars_to_ibis call in user code.
    return scan_database(connection, table_name)


def execute_on_database(query, connection, table_name: str):
    """
    Translates the provided query to SQL,
    targets the given database connection and table,
    executes the query on the database,
    and returns the result.

    :param query: A Polars query be translated to SQL.
    :param table_name: The name of the database table.
    :param connection: A connection object.

    :example:

    .. code:: pycon

        >>> connection = get_connection('sqlite')
        >>> table_name = 'demo'
        >>> import polars as pl
        >>> df = pl.DataFrame({"ints": [1, 2, 3, 4]})
        >>> connection.create_table(table_name, df, overwrite=True)
        DatabaseTable: demo
          ints int64

        >>> schema_lf = scan_database(connection, table_name)

        >>> context = dp.Context.compositor(
        ...     data=schema_lf,
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ...     split_evenly_over=1,
        ...     margins=[
        ...         dp.polars.Margin(max_length=1_000_000),
        ...     ],
        ... )
        >>> query = context.query().select(dp.len())
        >>> import warnings  # TODO: Silence warning upstream.
        >>> with warnings.catch_warnings():
        ...     warnings.simplefilter("ignore")
        ...     result = execute_on_database(query, connection, table_name)
    """
    import opendp.prelude as dp

    from polars_to_ibis import split_polars_on_ffi

    query_lf = query.release().lazy()

    ibis_table, plugin_parameters = split_polars_on_ffi(
        query_lf,
        table_name=table_name,
        # In the future, add a parameter to specify the plugin to split on?
    )

    # Use ibis_table:

    private_result = connection.to_polars(ibis_table).to_dict(as_series=False)
    private_item = next(iter(private_result.items()))[1][0]

    # Use plugin_parameters:

    # TODO: Probably replace with https://github.com/google/saferpickle
    # ... but that is work that can be done in opendp, after porting.
    import pickle

    kwargs = pickle.loads(bytes(plugin_parameters["kwargs"]))

    match kwargs["support"]:
        case "Integer":
            support = int
        case "Float":  # pragma: no cover
            support = float  # type: ignore[assignment]
        case _:  # pragma: no cover
            raise ValueError(
                f"Expected 'Integer' or 'Float', not {kwargs['support']}"
            )
    input_space = dp.atom_domain(T=support, nan=False), dp.absolute_distance(
        T=support
    )

    match kwargs["distribution"]:
        case "Laplace":
            make = dp.m.make_laplace
        case "Gaussian":  # pragma: no cover
            make = dp.m.make_gaussian
        case _:  # pragma: no cover
            raise ValueError(
                f"Expected 'Laplace' or 'Gaussian', not {kwargs['distribution']}"
            )
    measurement = make(*input_space, scale=kwargs["scale"])
    return measurement(private_item)