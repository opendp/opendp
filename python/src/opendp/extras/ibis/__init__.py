"""
This experimental module requires ``pip install 'opendp[ibis]'`` and an Ibis backend.
Its members are also available from ``opendp.prelude`` as ``dp.ibis``.

Ibis sources use the existing Polars Context API and privacy analysis. Only
collecting a released OnceFrame executes private database queries. The database
and trusted collector must preserve the analysis assumptions and keep all
deterministic intermediates private.
"""

from dataclasses import dataclass
from typing import Any

from opendp.extras.polars import OnceFrame

__all__ = ["scan_database", "make_private_ibis_lazyframe"]


@dataclass(frozen=True)
class _IbisSource:
    table: Any

    @property
    def schema(self):
        # Derive symbolic types from the very relation retained by the compositor.
        return self.table.schema().to_polars()


def scan_database(connection, table_name: str):
    """Bind a database table for use as ``data`` in :py:meth:`Context.compositor`.

    Only public schema metadata is inspected, not private rows. The returned
    source keeps its bound Ibis relation and symbolic Polars schema together.
    """
    return _IbisSource(connection.table(table_name))


def _ibis_domain(analysis_domain):
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


def make_private_ibis_lazyframe(
    input_domain,
    input_metric,
    output_measure,
    lazyframe,
    global_scale=None,
    threshold=None,
    *,
    source_domain=None,
):
    """Analyze a symbolic Polars query for a bound Ibis table.

    ``input_domain`` is the Polars analysis domain, not the physical input
    carrier. The returned measurement accepts a bound Ibis table through an
    extrinsic domain and returns a Python OnceFrame. It reuses the ordinary
    Polars measurement's exact metric, privacy map and calibrated mechanisms.

    The database contents must satisfy every descriptor of ``input_domain``,
    including bounds, nullability and margins. Only schema compatibility is
    checked without reading private rows; these assertions are a trusted input
    obligation. Constructing or checking this measurement executes no SQL.
    ``source_domain`` lets the Context reuse its exact extrinsic input domain.
    """
    from opendp._internal import _make_measurement
    from opendp.domains import _extrinsic_domain_descriptor, _lazyframe_from_domain
    from opendp.measurements import make_private_lazyframe

    native = make_private_lazyframe(
        input_domain,
        input_metric,
        output_measure,
        lazyframe,
        global_scale=global_scale,
        threshold=threshold,
    )
    symbolic = _lazyframe_from_domain(input_domain)
    domain = source_domain if source_domain is not None else _ibis_domain(input_domain)
    if _extrinsic_domain_descriptor(domain) != input_domain:
        raise ValueError("The analysis domain must match the Ibis source domain.")

    def release(table):
        if not domain.member(table):
            raise ValueError("Ibis relation schema does not match the analysis domain.")
        return _IbisOnceFrame(native(symbolic), table)

    measurement = _make_measurement(
        domain,
        native.input_metric,
        native.output_measure,
        release,
        native.map,
        TO="ExtrinsicObject",
    )
    # Summaries describe the calibrated native query, without executing its source.
    measurement._polars_measurement = native
    return measurement


class _IbisOnceFrame(OnceFrame):
    def __init__(self, native, table):
        from threading import Lock

        lock = Lock()
        consumed = False

        def collect_once():
            nonlocal consumed
            with lock:
                if consumed:
                    raise RuntimeError("OnceFrame is exhausted.")
                consumed = True
            return _collect(native, table)

        # A shared closure also preserves consumption across shallow copies.
        self._collect_once = collect_once

    def collect(self):
        """Consume this release before any translation or database execution."""
        return self._collect_once()

    def lazy(self):
        """Ibis releases cannot expose a reusable private compute plan."""
        raise NotImplementedError("Ibis-backed OnceFrame.lazy() is unsupported.")


def _collect(native, table):
    from tempfile import TemporaryDirectory
    from opendp._data import onceframe_split

    with TemporaryDirectory(prefix="opendp-ibis-") as work_dir:
        plans = onceframe_split(native.queryable, work_dir)
        try:
            return _execute_plans(plans, table, work_dir)
        except Exception:
            error = RuntimeError(
                "Database execution failed; the OnceFrame is exhausted."
            )
        # Discard private errors and execution frames, not just their displayed text.
        raise error from None


def _execute_plans(plans, table, work_dir):
    from pathlib import Path
    import pyarrow as pa  # type: ignore[import-untyped]
    from polars_to_ibis import convert_polars_to_ibis

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
