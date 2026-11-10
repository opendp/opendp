"""Stable conversion of aligned sklearn inputs to a typed Polars LazyFrame."""

from __future__ import annotations

from opendp._internal import _make_transformation
from opendp.mod import AtomDomain, DataFrameDomain, OptionDomain, SeriesDomain, VectorDomain
from opendp.extras.numpy._make_numpy_to_dataframe import make_numpy_to_dataframe
from opendp.extras.numpy._make_dataframe_to_numpy import _polars_dtype
from opendp.extras.sklearn._domains import SklearnFitDescriptor
from opendp.metrics import (
    change_one_distance,
    hamming_distance,
    insert_delete_distance,
    symmetric_distance,
)


def _series_dtype(element_domain):
    domain = element_domain
    if isinstance(domain, OptionDomain):
        domain = domain.element_domain
    if not isinstance(domain, AtomDomain):
        raise TypeError("Polars target must have an atomic element domain")
    return _polars_dtype(domain.carrier_type)


def make_sklearn_fit_to_lazyframe(
    input_domain, input_metric, *, feature_names=None, target_name=None,
):
    """Expose sklearn features to a typed Polars lazy query.

    X can be a NumPy matrix or an eager DataFrame. Its public dtype and
    schema are preserved. By default y remains separate and is not included.
    Supplying target_name explicitly includes y as a named column.
    This does not release private rows or spend privacy budget.
    """
    if input_metric not in (
        symmetric_distance(), change_one_distance(),
        insert_delete_distance(), hamming_distance(),
    ):
        raise ValueError("Conversion to Polars requires a row-adjacency metric")

    desc = input_domain.cast(SklearnFitDescriptor)
    from opendp.domains import lazyframe_domain, series_domain, option_domain

    if isinstance(desc.X, DataFrameDomain):
        if feature_names is not None:
            raise ValueError("DataFrame feature names are already determined by its domain")
        feature_frame = None
        feature_domain = desc.X
    else:
        # Reuse the NumPy-to-DataFrame bridge for dtype, names, and schema.
        feature_frame = make_numpy_to_dataframe(
            desc.X, input_metric, names=feature_names,
        )
        feature_domain = feature_frame.output_domain
    names = tuple(feature_domain.columns)
    series = [feature_domain.get_series_domain(name) for name in names]
    if target_name is not None and (
        not isinstance(target_name, str) or not target_name
    ):
        raise ValueError("target_name must be a nonempty string or None")
    if target_name is not None and desc.y is None:
        raise ValueError("No target is available to include")
    if target_name is not None and target_name in names:
        raise ValueError("target_name conflicts with a feature name")

    y_dtype = None
    if target_name is not None:
        if isinstance(desc.y, SeriesDomain):
            target_domain = (
                option_domain(desc.y.element_domain)
                if desc.y.nullable else desc.y.element_domain
            )
            y_dtype = _series_dtype(target_domain)
        elif isinstance(desc.y, VectorDomain):
            target_domain = desc.y.element_domain
            y_dtype = _series_dtype(target_domain)
        else:
            raise TypeError("Target must be a SeriesDomain or VectorDomain")
        series.append(series_domain(target_name, target_domain))

    output_domain = lazyframe_domain(series)

    def convert(data):
        import polars as pl

        frame = data.X.clone() if feature_frame is None else feature_frame(data.X)
        if target_name is not None:
            target = (
                data.y.clone().rename(target_name)
                if isinstance(desc.y, SeriesDomain)
                else pl.Series(target_name, data.y, dtype=y_dtype)
            )
            frame = frame.with_columns(target)
        return frame.lazy()

    return _make_transformation(
        input_domain, input_metric, output_domain, input_metric,
        convert, lambda d: d,
    )
