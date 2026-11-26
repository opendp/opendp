"""Stable, schema-preserving NumPy / eager Polars DataFrame bridges."""

import numpy as np
import pytest

import opendp.prelude as dp
from opendp.mod import OpenDPException

pl = pytest.importorskip("polars")
dp.enable_features("contrib")


@pytest.mark.parametrize(
    "T,dtype",
    [
        (dp.f32, np.float32),
        (dp.f64, np.float64),
        (dp.i32, np.int32),
        (dp.i64, np.int64),
        (bool, np.bool_),
    ],
)
def test_round_trip(T, dtype):
    from opendp.extras.numpy._make_numpy_to_dataframe import (
        make_numpy_to_dataframe,
    )
    from opendp.extras.numpy._make_dataframe_to_numpy import (
        make_dataframe_to_numpy,
    )

    data = np.array([[1, 0], [2, 1]], dtype=dtype)
    domain = (
        dp.numpy.array2_domain(T=T, num_columns=2, size=2, nan=False)
        if T in (dp.f32, dp.f64)
        else dp.numpy.array2_domain(T=T, num_columns=2, size=2)
    )
    to_frame = make_numpy_to_dataframe(
        domain,
        dp.symmetric_distance(),
        names=("left", "right"),
    )
    assert to_frame.map(2) == 2
    frame = to_frame(data)
    assert frame.columns == ["left", "right"]
    assert isinstance(to_frame.output_domain, dp.DataFrameDomain)
    assert to_frame.output_domain.columns == ["left", "right"]
    try:
        to_array = make_dataframe_to_numpy(
            to_frame.output_domain,
            dp.symmetric_distance(),
        )
    except OpenDPException as exc:
        if T == dp.f32 and "No match for concrete type f32" in str(exc):
            pytest.skip("Debug builds omit f32 domain introspection")
        raise
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
    frame = to_frame(np.array([[np.nan]]))
    assert np.isnan(frame["x0"][0])


def test_names_are_public_and_validated():
    from opendp.extras.numpy._make_numpy_to_dataframe import make_numpy_to_dataframe

    domain = dp.numpy.array2_domain(T=dp.f32, num_columns=2)
    with pytest.raises(ValueError, match="width"):
        make_numpy_to_dataframe(
            dp.numpy.array2_domain(T=dp.f32),
            dp.symmetric_distance(),
        )
    for names in (("a",), ("a", "a"), ("a", ""), (1, 2)):
        with pytest.raises(ValueError, match="names"):
            make_numpy_to_dataframe(domain, dp.symmetric_distance(), names=names)


def test_registry_routes_conversions_without_special_query_methods():
    data = np.array([[1.0, 2.0], [3.0, 4.0]], dtype=np.float64)
    ctx = dp.Context.compositor(
        data,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
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
        domain,
        dp.symmetric_distance(),
        names=("a", "b", "c"),
    )
    assert isinstance(transform.output_domain, dp.DataFrameDomain)
    assert transform.output_domain.columns == ["a", "b", "c"]
    for array in (
        np.array([[1, 2, 3], [4, 5, 6]], dtype=np.int32),
        np.empty((0, 3), dtype=np.int32),
    ):
        frame = transform(array)
        assert frame.columns == ["a", "b", "c"]
        assert frame.dtypes == [pl.Int32] * 3
        np.testing.assert_array_equal(frame.to_numpy(), array)


def test_fill_null_before_numeric_conversion():
    from opendp.extras.polars import make_dataframe_fill_null

    frame = pl.DataFrame(
        {
            "a": pl.Series([None, 1.0], dtype=pl.Float64),
            "b": pl.Series([2.0, None], dtype=pl.Float64),
        }
    )
    domain = dp.domain_of(frame, infer=True)
    fill = make_dataframe_fill_null(domain, dp.symmetric_distance(), value=0)
    to_numpy = dp.numpy.make_dataframe_to_numpy(
        *fill.output_space,
    )
    transform = fill >> to_numpy
    np.testing.assert_array_equal(
        transform(frame),
        np.array([[0.0, 2.0], [1.0, 0.0]]),
    )
    assert transform.check(1, 1)
    assert to_numpy.output_domain.descriptor.num_columns == 2


def test_frame_to_row_distance_uses_public_global_bound():
    from opendp.extras.numpy._row_distance import (
        _frame_distance_to_symmetric_distance,
    )

    metric, stability = _frame_distance_to_symmetric_distance(
        dp.frame_distance(dp.symmetric_distance())
    )
    assert metric == dp.symmetric_distance()
    assert stability([dp.polars.Bound(by=[], per_group=3, num_groups=1)]) == 3
    with pytest.raises(ValueError, match="global"):
        stability([dp.polars.Bound(by=["group"], per_group=3)])


def test_dataframe_to_numpy_rejects_invalid_public_schemas():
    from opendp.extras.numpy._make_dataframe_to_numpy import (
        _numpy_dtype,
        make_dataframe_to_numpy,
    )

    metric = dp.symmetric_distance()
    with pytest.raises(TypeError, match="DataFrameDomain"):
        make_dataframe_to_numpy(dp.atom_domain(T=dp.i32), metric)
    with pytest.raises(ValueError, match="At least one feature"):
        make_dataframe_to_numpy(dp.dataframe_domain([]), metric)

    nullable = dp.dataframe_domain(
        [dp.series_domain("x", dp.option_domain(dp.atom_domain(T=dp.f64)))]
    )
    with pytest.raises(ValueError, match="non-nullable"):
        make_dataframe_to_numpy(nullable, metric)

    mixed = dp.dataframe_domain(
        [
            dp.series_domain("x", dp.atom_domain(T=dp.i32)),
            dp.series_domain("y", dp.atom_domain(T=dp.f64)),
        ]
    )
    with pytest.raises(TypeError, match="uniform numeric"):
        make_dataframe_to_numpy(mixed, metric)

    strings = dp.dataframe_domain([dp.series_domain("name", dp.atom_domain(T=str))])
    with pytest.raises(TypeError, match="uniform numeric"):
        make_dataframe_to_numpy(strings, metric)
    with pytest.raises(TypeError, match="Unsupported NumPy dtype"):
        _numpy_dtype("String")


def test_numpy_bridge_rejects_invalid_public_names_and_row_metrics():
    from opendp.extras.numpy._make_numpy_to_dataframe import make_numpy_to_dataframe
    from opendp.extras.numpy._row_distance import _row_distance

    domain = dp.numpy.array2_domain(T=dp.i32, num_columns=2)
    with pytest.raises(TypeError, match="sequence"):
        make_numpy_to_dataframe(domain, dp.symmetric_distance(), names="x")
    for metric in (
        dp.change_one_distance(),
        dp.insert_delete_distance(),
        dp.hamming_distance(),
        dp.discrete_distance(),
        dp.frame_distance(dp.symmetric_distance()),
    ):
        with pytest.raises(ValueError, match="SymmetricDistance"):
            _row_distance(metric)
        with pytest.raises(ValueError, match="SymmetricDistance"):
            make_numpy_to_dataframe(domain, metric)

    from opendp.extras.numpy._row_distance import (
        _frame_distance_to_symmetric_distance,
    )

    _, stability = _frame_distance_to_symmetric_distance(
        dp.frame_distance(dp.symmetric_distance())
    )
    with pytest.raises(ValueError, match="global row-contribution bound"):
        stability([dp.polars.Bound(by=[], per_group=None, num_groups=1)])


def test_bridge_callbacks_reject_mismatched_public_dtypes():
    from opendp.extras.numpy._make_dataframe_to_numpy import make_dataframe_to_numpy
    from opendp.extras.numpy._make_numpy_to_dataframe import make_numpy_to_dataframe

    frame_domain = dp.dataframe_domain(
        [dp.series_domain("x", dp.atom_domain(T=dp.f64, nan=False))]
    )
    to_numpy = make_dataframe_to_numpy(frame_domain, dp.symmetric_distance())
    with pytest.raises(Exception, match="Frame dtype differs from its public domain"):
        to_numpy(pl.DataFrame({"x": pl.Series([1], dtype=pl.Int32)}))

    array_domain = dp.numpy.array2_domain(T=dp.i32, num_columns=1)
    to_frame = make_numpy_to_dataframe(array_domain, dp.symmetric_distance())
    with pytest.raises(Exception, match="does not match the public domain"):
        to_frame(np.array([[1.0]], dtype=np.float64))
    with pytest.raises(Exception, match="does not match the public domain"):
        to_frame(np.array([[1, 2]], dtype=np.int32))


def test_numpy_string_conversion_preserves_public_schema():
    domain = dp.numpy.array2_domain(T=str, num_columns=2)
    transform = dp.numpy.make_numpy_to_dataframe(
        domain, dp.symmetric_distance(), names=("label", "group")
    )
    for array in (
        np.array([["negative", "α"], ["positive", ""]]),
        np.empty((0, 2), dtype=str),
    ):
        result = transform(array)
        assert result.schema == {"label": pl.String, "group": pl.String}
        assert result.rows() == [tuple(row) for row in array.tolist()]
        assert transform.output_domain == dp.dataframe_domain(
            [
                dp.series_domain(name, dp.atom_domain(T=str))
                for name in ("label", "group")
            ]
        )
    assert transform.map(1) == 1
    with pytest.raises(Exception, match="does not match the public domain"):
        transform(np.array([["negative", "α"]], dtype=object))


def test_frame_distance_rejects_non_symmetric_inner_metric():
    from opendp.extras.numpy._row_distance import (
        _frame_distance_to_symmetric_distance,
    )

    metric = dp.frame_distance(dp.symmetric_id_distance(pl.col("owner")))
    with pytest.raises(ValueError, match="Only FrameDistance"):
        _frame_distance_to_symmetric_distance(metric)
    with pytest.raises(ValueError, match="Only FrameDistance"):
        dp.numpy.make_dataframe_to_numpy(
            dp.dataframe_domain([dp.series_domain("x", dp.atom_domain(T=dp.i32))]),
            metric,
        )


@pytest.mark.parametrize("T", [dp.u8, dp.u16])
def test_bridge_rejects_trimmed_unsigned_dtypes(T):
    from opendp.extras.numpy import (
        make_dataframe_to_numpy,
        make_numpy_to_dataframe,
    )

    array_domain = dp.numpy.array2_domain(T=T, num_columns=1)
    with pytest.raises(TypeError, match="Unsupported NumPy dtype"):
        make_numpy_to_dataframe(array_domain, dp.symmetric_distance())

    frame_domain = dp.dataframe_domain([dp.series_domain("x", dp.atom_domain(T=T))])
    with pytest.raises(TypeError, match="uniform numeric dtype"):
        make_dataframe_to_numpy(frame_domain, dp.symmetric_distance())


def test_query_collect_to_numpy_lowers_frame_distance():
    frame = pl.DataFrame({"x": pl.Series([1, 2, 3], dtype=pl.Int32)})
    domain = dp.lazyframe_domain([dp.series_domain("x", dp.atom_domain(T=dp.i32))])
    context = dp.Context.compositor(
        frame.lazy(),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )
    converted = context.query().collect().to_numpy()
    transformation = converted.resolve(allow_transformations=True)

    assert transformation.output_metric == dp.symmetric_distance()
    assert transformation.map(1) == 1
    np.testing.assert_array_equal(transformation(frame.lazy()), frame.to_numpy())
    assert context.current_privacy_loss() == []


def test_public_numpy_bridges_without_hbc():
    array = np.array([[1, 2], [3, 4]], dtype=np.int32)
    domain = dp.numpy.array2_domain(T=dp.i32, num_columns=2)
    dp.disable_features("honest-but-curious")
    try:
        to_frame = dp.numpy.make_numpy_to_dataframe(domain, dp.symmetric_distance())
        to_numpy = dp.numpy.make_dataframe_to_numpy(*to_frame.output_space)
        transform = to_frame >> to_numpy
        np.testing.assert_array_equal(transform(array), array)
        assert transform.map(1) == 1
    finally:
        dp.enable_features("honest-but-curious")
