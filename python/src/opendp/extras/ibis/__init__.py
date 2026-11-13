"""Bound Ibis sources for the Polars Context API.

Database contents must satisfy the analysis domain's bounds, nullability and
margins. Schema checks do not certify private values. Trusted collection keeps
deterministic intermediates private and executes only after release.
"""

from dataclasses import dataclass
from typing import Any

__all__ = ["scan_database"]


@dataclass(frozen=True)
class _IbisSource:
    table: Any

    @property
    def schema(self):
        return self.table.schema().to_polars()


def scan_database(connection, table_name: str):
    """Bind a table for ``Context.compositor`` without reading private rows."""
    return _IbisSource(connection.table(table_name))


def _ibistable_domain(analysis_domain):
    from uuid import uuid4
    from opendp._lib import import_optional_dependency
    from opendp._internal import _extrinsic_domain
    from opendp.domains import _lazyframe_from_domain

    Table = import_optional_dependency("ibis").Table
    schema = _lazyframe_from_domain(analysis_domain).collect_schema()

    def member(table):
        if not isinstance(table, Table):
            return False
        backends, unbound = table._find_backends()
        return (
            len(backends) == 1
            and not unbound
            and list(table.schema().to_polars().items()) == list(schema.items())
        )

    # Extrinsic domains compare identifiers, not their descriptors.
    return _extrinsic_domain(
        f"IbisTable({uuid4()})", member, descriptor=analysis_domain
    )


def _adapt_ibis_measurement(native, source_domain):
    """Adapt a calibrated Polars measurement to a bound Ibis input."""
    from opendp._data import onceframe_with_collector
    from opendp._internal import _make_measurement
    from opendp.domains import _extrinsic_domain_descriptor, _lazyframe_from_domain

    if _extrinsic_domain_descriptor(source_domain) != native.input_domain:
        raise ValueError("The analysis domain must match the Ibis source domain.")
    symbolic = _lazyframe_from_domain(native.input_domain)

    def release(table):
        if not source_domain.member(table):
            raise ValueError("Ibis relation schema does not match the analysis domain.")
        private = native(symbolic)
        return onceframe_with_collector(lambda _: _collect(private, table))

    return _make_measurement(
        source_domain,
        native.input_metric,
        native.output_measure,
        release,
        native.map,
        TO="AnyQueryable",
    )


def _collect(native, table):
    from pathlib import Path
    from tempfile import TemporaryDirectory
    import pyarrow as pa  # type: ignore[import-untyped]
    from polars_to_ibis import convert_polars_to_ibis
    from opendp._data import onceframe_split

    with TemporaryDirectory(prefix="opendp-ibis-") as work_dir:
        plans = onceframe_split(native.queryable, work_dir)
        backend = table._find_backend()
        for i, plan in enumerate(plans[:-1]):
            expression = convert_polars_to_ibis(plan, table=table)
            with (
                backend.to_pyarrow_batches(expression, limit=None) as batches,
                pa.OSFile(str(Path(work_dir) / f"{i}.arrow"), "wb") as sink,
                pa.ipc.new_file(sink, batches.schema) as writer,
            ):
                for batch in batches:
                    # SQL literal types may be narrower than the Ibis schema.
                    writer.write_batch(batch.cast(batches.schema))
        return plans[-1].collect().sample(fraction=1.0, shuffle=True)
