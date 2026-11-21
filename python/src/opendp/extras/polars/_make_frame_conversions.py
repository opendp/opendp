"""Python-backed stable conversions between eager and lazy Polars frames."""

from opendp._internal import _make_transformation
from opendp.domains import _frame_domain_with_series
from opendp.mod import DataFrameDomain, LazyFrameDomain


_FRAME_METRICS = {
    "SymmetricDistance",
    "ChangeOneDistance",
    "SymmetricIdDistance",
    "ChangeOneIdDistance",
    "FrameDistance<SymmetricDistance>",
    "FrameDistance<SymmetricIdDistance>",
}


def _convert(input_domain, input_metric, input_type, output_carrier, function):
    if not isinstance(input_domain, input_type):
        raise TypeError(f"Expected {input_type.__name__}")
    if str(input_metric.type) not in _FRAME_METRICS:
        raise ValueError(f"Unsupported Polars frame metric: {input_metric}")

    output_domain = _frame_domain_with_series(
        input_domain,
        [input_domain.get_series_domain(name) for name in input_domain.columns],
        output_carrier,
    )
    return _make_transformation(
        input_domain, input_metric, output_domain, input_metric, function, lambda d: d
    )


def make_lazy(input_domain, input_metric):
    """Convert an eager Polars DataFrame to a lazy frame, preserving its domain."""
    return _convert(
        input_domain, input_metric, DataFrameDomain, "LazyFrame", lambda f: f.lazy()
    )


def make_collect(input_domain, input_metric):
    """Materialize a stable Polars LazyFrame to a DataFrame without releasing it."""
    return _convert(
        input_domain, input_metric, LazyFrameDomain, "DataFrame", lambda f: f.collect()
    )
