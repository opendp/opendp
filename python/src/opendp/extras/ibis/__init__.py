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

    import polars_to_ibis

    query_lf = query.release().lazy()

    # The two-stage planner is optional until its API is available in a release.
    # Older translator versions retain the existing scalar execution path.
    split_plan = getattr(polars_to_ibis, "split_polars_on_ffi_plan", None)
    if split_plan is not None:
        plan = split_plan(query_lf, table_name=table_name, backend=connection)
        return _run_ffi_split_plan(plan, connection)

    ibis_table, param_dicts = polars_to_ibis.split_polars_on_ffi(
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


def _run_ffi_split_plan(plan, connection):
    """Noise each typed SQL aggregate before evaluating public postprocessing."""
    from json import loads

    import polars as pl  # type: ignore[import-not-found]
    import opendp.prelude as dp
    from opendp._data import decode_noise_plugin_parameters

    carriers = {
        pl.Int32: "i32",
        pl.Int64: "i64",
        pl.UInt32: "u32",
        pl.UInt64: "u64",
        pl.Float64: "f64",
    }
    names = [plugin.name for plugin in plan.plugins]
    if not names or len(set(names)) != len(names):
        raise dp.OpenDPException("Expected distinct scalar noise plugin columns.")

    measurements = []
    for plugin in plan.plugins:
        if plugin.parameters["symbol"] != "noise_plugin":
            raise dp.OpenDPException("Only OpenDP noise plugins are supported.")
        if plugin.input_dtype not in carriers:
            raise dp.OpenDPException("Unsupported native noise input type.")
        kwargs = loads(decode_noise_plugin_parameters(plugin.parameters["kwargs"]))
        expected_support = "Integer" if plugin.input_dtype.is_integer() else "Float"
        if kwargs["support"] != expected_support:
            raise dp.OpenDPException("Noise support does not match its input type.")
        carrier = carriers[plugin.input_dtype]
        input_space = (
            dp.atom_domain(T=carrier, nan=False),
            dp.absolute_distance(T=carrier),
        )
        make = {
            "Laplace": dp.m.make_laplace,
            "Gaussian": dp.m.make_gaussian,
        }[kwargs["distribution"]]
        measurements.append(make(*input_space, scale=kwargs["scale"]))

    private_result = connection.to_polars(plan.ibis_table)
    if private_result.height != 1 or set(private_result.columns) != set(names):
        raise dp.OpenDPException(
            "Private SQL values are not 1-1 with the scalar noise parameters."
        )

    def typed_scalar(value, plugin, domain):
        # SQL result types do not identify the native carrier: e.g. a backend's
        # COUNT may be Int64 while the original Polars expression is UInt32.
        if plugin.input_dtype.is_integer():
            valid_type = type(value) is int
        else:
            valid_type = type(value) in (int, float)
        if not valid_type:
            raise dp.OpenDPException("Invalid scalar value for a native noise type.")
        try:
            value = pl.Series([value], dtype=plugin.input_dtype, strict=True)[0]
            if not domain.member(value):
                raise ValueError("Value outside the noise domain")
        except (
            ValueError,
            TypeError,
            OverflowError,
            dp.OpenDPException,
            pl.exceptions.PolarsError,
        ):
            raise dp.OpenDPException(
                "Scalar value is outside its native noise type."
            ) from None
        return value

    # Validate all private scalar carriers before evaluating any mechanism.
    private_values = [
        typed_scalar(private_result[plugin.name][0], plugin, measurement.input_domain)
        for plugin, measurement in zip(plan.plugins, measurements)
    ]
    noisy_columns = [
        pl.Series(
            plugin.name,
            [typed_scalar(measurement(value), plugin, measurement.input_domain)],
            dtype=plugin.input_dtype,
        )
        for plugin, measurement, value in zip(
            plan.plugins, measurements, private_values
        )
    ]
    result = plan.postprocess(pl.DataFrame(noisy_columns))
    if result.height != 1:
        raise dp.OpenDPException("Expected one postprocessed result row.")
    return list(result.row(0))
