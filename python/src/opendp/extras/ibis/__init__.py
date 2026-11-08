"""
This module requires extra installs: ``pip install 'opendp[ibis]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.ibis``.

.. caution::

    This is an early release, and is intended primarily to solicit feedback.
    It supports only a limited set of polars expressions,
    and in the future the API may change.

    For the moment, ``contrib`` and ``honest-but-curious`` must be enabled:

    .. code:: pycon

        >>> dp.enable_features("contrib", "honest-but-curious")
"""

__all__ = ["run_on_database", "get_connection", "scan_database"]


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
    from polars_to_ibis import scan_database  # type: ignore[import-untyped]

    # Avoid a direct polars_to_ibis call in user code.
    return scan_database(connection, table_name)


def run_on_database(query, connection, table_name: str):
    """
    Translates the provided query to SQL,
    targets the given database connection and table,
    runs the query on the database,
    and returns the result.

    Requires ``contrib`` and ``honest-but-curious``. The connection and executor
    must be trusted with private data. The query is consumed even if execution
    fails; intermediate IPC files are protected and removed before returning.

    :param query: A Polars query be translated to SQL.
    :param table_name: The name of the database table.
    :param connection: A DuckDB or SQLite connection with no active transaction.
    :returns: A Polars DataFrame containing the complete randomized result.

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

        >>> # TODO: Support signed=True
        >>> # https://github.com/opendp/polars-to-ibis/issues/175
        >>> query = context.query().select(dp.len(signed=False))

        >>> # TODO: Silence warning upstream.
        >>> # https://github.com/opendp/polars-to-ibis/issues/167
        >>> import warnings
        >>> with warnings.catch_warnings():
        ...     warnings.simplefilter("ignore")
        ...     result = run_on_database(query, connection, table_name)

        >>> print("DP result:", list(result.row(0)))
        DP result: [...]
        >>> connection.disconnect()

    """
    import opendp.prelude as dp

    dp.assert_features("contrib", "honest-but-curious")
    return _run_on_database(query.release(), connection, table_name)


def _run_on_database(onceframe, connection, table_name: str):
    from tempfile import TemporaryDirectory

    from opendp._data import onceframe_split

    if connection.name not in {"duckdb", "sqlite"}:
        raise NotImplementedError(
            "Database execution currently requires a DuckDB or SQLite snapshot."
        )

    with TemporaryDirectory(prefix="opendp-ibis-") as work_dir:
        sinks, continuation = onceframe_split(onceframe.queryable, work_dir)
        connection.raw_sql("BEGIN TRANSACTION")
        try:
            _materialize_sinks(sinks, connection, table_name)
            return continuation.collect().sample(fraction=1.0, shuffle=True)
        finally:
            connection.raw_sql("ROLLBACK")


def _materialize_sinks(sinks, connection, table_name: str):
    import io
    import json
    import warnings

    import polars as pl
    from polars_to_ibis import convert_polars_to_ibis

    with warnings.catch_warnings():
        warnings.simplefilter("ignore", UserWarning)
        inputs = json.loads(sinks.serialize(format="json"))["SinkMultiple"]["inputs"]

    for node in inputs:
        sink = node["Sink"]
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)
            head = pl.LazyFrame.deserialize(
                io.StringIO(json.dumps(sink["input"])), format="json"
            )
        if "DataFrameScan" in sink["input"]:
            result = head.collect()
        else:
            query = convert_polars_to_ibis(
                head, table_name=table_name, backend=connection
            )
            result = connection.to_polars(query, limit=None)
        result.write_ipc(sink["payload"]["File"]["target"]["Local"])
