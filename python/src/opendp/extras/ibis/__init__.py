"""
This experimental module requires ``pip install 'opendp[ibis]'`` and an Ibis backend.
Its members are also available from ``opendp.prelude`` as ``dp.ibis``.

Requires the ``contrib`` and ``honest-but-curious`` features. The executor and
connection are trusted with private data; only the final randomized result may
be exposed to the analyst.
"""

__all__ = ["make_executor", "scan_database"]


def scan_database(connection, table_name: str):
    """Construct an empty Polars LazyFrame with the database table's schema."""
    from polars_to_ibis import scan_database

    return scan_database(connection, table_name)


def make_executor(connection, table_name: str, *, snapshot=None):
    """Make an interactive postprocessor from a OnceFrame to a Polars DataFrame.

    Use ``make_executor(connection, table_name)(query.release())``, or chain the
    returned OpenDP Function onto a measurement. This realizes the existing
    release without spending additional privacy budget. A OnceFrame is consumed
    before splitting, including on failure; execution must not be retried.

    All precomputations run through Ibis over the designated database table.
    Additional in-memory table sources are unsupported. Intermediate IPC files
    are private to the executor and removed on success or failure. Polars runs
    only the randomized continuation, whose complete output is shuffled.
    Execution errors are redacted because they may contain private intermediates.

    :param connection: An Ibis backend supporting ``to_pyarrow_batches``.
    :param table_name: The database table represented by the OnceFrame's source.
    :param snapshot: Optional context-manager factory called with ``connection``.
        It must yield an Ibis backend bound to one consistent, repeatable source
        snapshot, exclusively used throughout the context, and clean up on exit.
        Required when a release has multiple precomputations: Ibis has no common
        snapshot-isolation capability API, and an ordinary transaction may not
        provide repeatable reads. The caller must supply the backend-specific
        guarantee, not merely issue BEGIN. A single precomputation instead relies
        on the backend's single-query consistency guarantee.
    :returns: An OpenDP Function consuming a OnceFrame and returning a DataFrame.
    """
    from contextlib import nullcontext
    from pathlib import Path
    from tempfile import TemporaryDirectory

    import pyarrow as pa  # type: ignore[import-untyped]
    from polars_to_ibis import convert_polars_to_ibis

    from opendp._data import onceframe_split
    from opendp._internal import _new_pure_function
    from opendp.mod import assert_features

    assert_features("contrib", "honest-but-curious")

    def execute(onceframe):
        with TemporaryDirectory(prefix="opendp-ibis-") as work_dir:
            plans = onceframe_split(onceframe.queryable, work_dir)
            if len(plans) > 2 and snapshot is None:
                raise ValueError(
                    "Multiple precomputations require a repeatable snapshot context."
                )
            try:
                scope = (
                    snapshot(connection)
                    if snapshot is not None
                    else nullcontext(connection)
                )
                with scope as backend:
                    for i, plan in enumerate(plans[:-1]):
                        expression = convert_polars_to_ibis(
                            plan, table_name=table_name, backend=backend
                        )
                        with (
                            backend.to_pyarrow_batches(
                                expression, limit=None
                            ) as batches,
                            pa.OSFile(str(Path(work_dir) / f"{i}.arrow"), "wb") as sink,
                            pa.ipc.new_file(sink, batches.schema) as writer,
                        ):
                            for batch in batches:
                                # SQL literal types may be narrower than the Ibis schema.
                                writer.write_batch(batch.cast(batches.schema))
                return plans[-1].collect().sample(fraction=1.0, shuffle=True)
            except Exception:
                raise RuntimeError(
                    "Database execution failed; the OnceFrame is exhausted."
                ) from None

    return _new_pure_function(execute, TO="DataFrame")
