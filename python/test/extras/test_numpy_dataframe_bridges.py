"""Stable, schema-preserving NumPy / eager Polars DataFrame bridges."""

import numpy as np
import pytest

import opendp.prelude as dp

pl = pytest.importorskip("polars")
dp.enable_features("contrib")


@pytest.mark.parametrize("T,dtype", [
    (dp.f32, np.float32), (dp.f64, np.float64),
    (dp.i32, np.int32), (dp.i64, np.int64), (bool, np.bool_),
])
def test_round_trip(T, dtype):
    from opendp.extras.numpy._make_numpy_to_dataframe import (
        make_numpy_to_dataframe,
    )
    from opendp.extras.numpy._make_dataframe_to_numpy import (
        make_dataframe_to_numpy,
    )

    data = np.array([[1, 0], [2, 1]], dtype=dtype)
    domain = dp.numpy.array2_domain(
        T=T, num_columns=2, size=2,
        **({"nan": False} if T in (dp.f32, dp.f64) else {}),
    )
    to_frame = make_numpy_to_dataframe(
        domain, dp.symmetric_distance(), names=("left", "right"),
    )
    assert to_frame.map(2) == 2
    frame = to_frame(data)
    assert frame.columns == ["left", "right"]
    assert to_frame.output_domain.member(frame)
    to_array = make_dataframe_to_numpy(
        to_frame.output_domain, dp.symmetric_distance(),
    )
    result = to_array(frame)
    assert result.dtype == dtype
    np.testing.assert_array_equal(result, data)
    assert to_array.map(2) == 2
    assert to_array.output_domain.descriptor.nan is False


def test_nan_policy_is_preserved():
    from opendp.extras.numpy._make_numpy_to_dataframe import make_numpy_to_dataframe

    domain = dp.numpy.array2_domain(T=dp.f64, num_columns=1, nan=True)
    to_frame = make_numpy_to_dataframe(domain, dp.symmetric_distance())
    assert to_frame.output_domain.get_series_domain("x0").element_domain.nan is True
    assert to_frame.output_domain.member(to_frame(np.array([[np.nan]])))


def test_names_are_public_and_validated():
    from opendp.extras.numpy._make_numpy_to_dataframe import make_numpy_to_dataframe

    domain = dp.numpy.array2_domain(T=dp.f32, num_columns=2)
    with pytest.raises(ValueError, match="width"):
        make_numpy_to_dataframe(
            dp.numpy.array2_domain(T=dp.f32), dp.symmetric_distance(),
        )
    for names in (("a",), ("a", "a"), ("a", ""), (1, 2)):
        with pytest.raises(ValueError, match="names"):
            make_numpy_to_dataframe(domain, dp.symmetric_distance(), names=names)


def test_registry_routes_conversions_without_special_query_methods():
    data = np.array([[1., 2.], [3., 4.]], dtype=np.float64)
    ctx = dp.Context.compositor(
        data, dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.), split_evenly_over=1,
    )
    converted = ctx.query().to_dataframe(names=("first", "second")).to_numpy()
    trans = converted.resolve(allow_transformations=True)
    np.testing.assert_array_equal(trans(data), data)
    assert trans.map(1) == 1
    assert ctx.current_privacy_loss() == []


def test_rectangular_and_empty_arrays_keep_schema_and_orientation():
    from opendp.extras.numpy._make_numpy_to_dataframe import make_numpy_to_dataframe

    domain = dp.numpy.array2_domain(T=dp.i32, num_columns=3)
    transform = make_numpy_to_dataframe(
        domain, dp.symmetric_distance(), names=("a", "b", "c"),
    )
    for array in (
        np.array([[1, 2, 3], [4, 5, 6]], dtype=np.int32),
        np.empty((0, 3), dtype=np.int32),
    ):
        frame = transform(array)
        assert frame.columns == ["a", "b", "c"]
        assert frame.dtypes == [pl.Int32] * 3
        np.testing.assert_array_equal(frame.to_numpy(), array)
        assert transform.output_domain.member(frame)


def test_fill_null_before_numeric_conversion():
    from opendp.extras.polars import make_dataframe_fill_null

    frame = pl.DataFrame({
        "a": pl.Series([None, 1.0], dtype=pl.Float64),
        "b": pl.Series([2.0, None], dtype=pl.Float64),
    })
    domain = dp.domain_of(frame, infer=True)
    fill = make_dataframe_fill_null(domain, dp.symmetric_distance(), value=0)
    to_numpy = dp.numpy.make_dataframe_to_numpy(
        *fill.output_space,
    )
    transform = fill >> to_numpy
    np.testing.assert_array_equal(
        transform(frame), np.array([[0.0, 2.0], [1.0, 0.0]]),
    )
    assert transform.check(1, 1)
    assert to_numpy.output_domain.descriptor.num_columns == 2
