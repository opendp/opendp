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

    For the moment, ``contrib`` must be enabled to use this functionality:

    .. code:: pycon

        >>> dp.enable_features("contrib")
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

    The ``contrib`` feature must be enabled until we are confident in this code.

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

        >>> # TODO: Support signed=True
        >>> # https://github.com/opendp/polars-to-ibis/issues/175
        >>> query = context.query().select(dp.len(signed=False))

        >>> # TODO: Silence warning upstream.
        >>> # https://github.com/opendp/polars-to-ibis/issues/167
        >>> import warnings
        >>> with warnings.catch_warnings():
        ...     warnings.simplefilter("ignore")
        ...     result = run_on_database(query, connection, table_name)

        >>> print("DP result:", result)
        DP result: [...]
        >>> connection.disconnect()

    """
    import opendp.prelude as dp

    from polars_to_ibis import split_polars_on_ffi

    dp.assert_features("contrib")

    query_lf = query.release().lazy()

    ibis_table, param_dicts = split_polars_on_ffi(
        query_lf,
        table_name=table_name,
        backend=connection,
        # In the future, add a parameter to specify the plugin to split on?
    )

    # Decode the parameters as data in Rust, without loading Python objects.
    # Validate every plugin before executing the private aggregate query.
    from json import loads
    from opendp._data import decode_noise_plugin_parameters

    measurements = []
    for param_dict in param_dicts:
        if param_dict["symbol"] != "noise_plugin":
            raise dp.OpenDPException("Only OpenDP noise plugins are supported.")
        kwargs = loads(decode_noise_plugin_parameters(param_dict["kwargs"]))
        support = {
            "Integer": int,
            "Float": float,
        }[kwargs["support"]]
        input_space = (
            dp.atom_domain(T=support, nan=False),
            dp.absolute_distance(T=support),
        )

        make = {
            "Laplace": dp.m.make_laplace,
            "Gaussian": dp.m.make_gaussian,
        }[kwargs["distribution"]]
        measurements.append(make(*input_space, scale=kwargs["scale"]))

    private_result = connection.to_polars(ibis_table).to_dict(as_series=False)
    # For now, assume result dataframe is only a single row,
    # so pull out single values with [0],
    # but I'm not sure that will always be true.
    private_items = [v[0] for v in private_result.values()]

    if len(private_items) != len(measurements):
        raise dp.OpenDPException(
            "Some operations (like dp.mean) are not currently supported "
            "because the plugin-parameters are not 1-1 with the private values."
        )  # pragma: no cover

    return [measurement(item) for item, measurement in zip(private_items, measurements)]
