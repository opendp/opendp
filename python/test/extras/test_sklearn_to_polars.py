"""Stable sklearn-to-Polars query handoff and privacy accounting."""

import numpy as np
import pytest
import opendp.prelude as dp

from opendp._internal import _make_transformation

pl = pytest.importorskip("polars")
dp.enable_features("contrib")


def _space(n=3):
    x_domain = dp.numpy.array2_domain(
        T=dp.f32, num_columns=2, nan=False,
    )
    y_domain = dp.vector_domain(dp.atom_domain(T=dp.i32, bounds=(0, 1)))
    domain = dp.sklearn.sklearn_fit_domain(x_domain, y_domain)
    data = dp.sklearn.SklearnFit(
        np.array([[0.2, 1.0], [0.2, 1.0], [0.8, 2.0]][:n], dtype=np.float32),
        np.array([0, 1, 0][:n], dtype=np.int32),
    )
    return domain, data


def _context(domain, data, *, d_in=1):
    return dp.Context.compositor(
        data,
        dp.unit_of(contributions=d_in),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )


def test_conversion_preserves_rows_types_and_targets():
    domain, data = _space()
    transform = dp.sklearn.make_sklearn_fit_to_lazyframe(
        domain, dp.symmetric_distance(), target_name="y",
    )
    assert transform.output_domain.columns == ["x0", "x1", "y"]
    assert transform.map(2) == 2

    result = transform(data)
    assert isinstance(result, pl.LazyFrame)
    assert result.collect_schema() == {
        "x0": pl.Float32, "x1": pl.Float32, "y": pl.Int32,
    }
    expected = result.collect()
    assert expected.get_column("y").to_list() == [0, 1, 0]
    assert expected.get_column("x0").to_list()[0] == expected.get_column("x0").to_list()[1]
    assert transform.output_domain.member(result)


def test_custom_names_and_empty_input():
    domain, _ = _space()
    transform = dp.sklearn.make_sklearn_fit_to_lazyframe(
        domain, dp.symmetric_distance(),
        feature_names=("first", "second"), target_name="label",
    )
    data = dp.sklearn.SklearnFit(
        np.empty((0, 2), dtype=np.float32),
        np.empty((0,), dtype=np.int32),
    )
    out = transform(data).collect()
    assert out.shape == (0, 3)
    assert out.columns == ["first", "second", "label"]
    assert out.schema == {
        "first": pl.Float32, "second": pl.Float32, "label": pl.Int32,
    }


def test_no_target_and_missing_schema_are_explicit():
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2),
    )
    conversion = dp.sklearn.make_sklearn_fit_to_lazyframe(
        domain, dp.symmetric_distance()
    )
    assert conversion.output_domain.columns == ["x0", "x1"]
    result = conversion(
        dp.sklearn.SklearnFit(np.ones((2, 2), dtype=np.float64))
    ).collect()
    assert result.shape == (2, 2)
    assert result.dtypes == [pl.Float64, pl.Float64]

    unknown = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64)
    )
    with pytest.raises(ValueError, match="public width"):
        dp.sklearn.make_sklearn_fit_to_lazyframe(
            unknown, dp.symmetric_distance()
        )


@pytest.mark.parametrize("names, target", [
    (["a"], "y"),
    (["a", "a"], "y"),
    (["a", "y"], "y"),
])
def test_reject_bad_column_names(names, target):
    domain, _ = _space()
    with pytest.raises(ValueError):
        dp.sklearn.make_sklearn_fit_to_lazyframe(
            domain, dp.symmetric_distance(),
            feature_names=names, target_name=target,
        )


def test_symbolic_handoff_only_consumes_privacy_on_aggregation():
    domain, data = _space()
    ctx = _context(domain, data)
    query = ctx.query().lazy(target_name="y")

    assert ctx.current_privacy_loss() == []
    assert query.collect_schema() == {
        "x0": pl.Float32, "x1": pl.Float32, "y": pl.Int32,
    }

    collected = query.collect()
    assert isinstance(collected, dp.Query)
    assert isinstance(collected._chain.output_domain, dp.DataFrameDomain)
    assert ctx.current_privacy_loss() == []

    # The existing Polars DP API handles the resulting stable LazyFrame.
    result = query.select(dp.len(signed=True)).release().collect()
    assert result.height == 1 and "len" in result.columns
    assert ctx.current_privacy_loss() == [1.0]


def test_previous_stability_map_is_used_for_calibration():
    domain, data = _space()
    ctx = _context(domain, data)
    stable = _make_transformation(
        domain, dp.symmetric_distance(), domain, dp.symmetric_distance(),
        lambda d: d, lambda d: d * 2,
    )
    query = ctx.query().new_with(chain=stable).lazy()
    assert query._query._chain.map(1) == 2
    measure = query.select(dp.len(signed=True)).resolve()
    assert measure.input_domain == domain
    assert measure.input_metric == dp.symmetric_distance()
    assert measure.check(1, 1.0)
    assert ctx.current_privacy_loss() == []
    answer = query.select(dp.len(signed=True)).release().collect()
    assert answer.shape == (1, 1)
    assert ctx.current_privacy_loss() == [1.0]


def test_typed_lazyframe_prefix_passes_through_without_reconversion():
    domain, data = _space()
    ctx = _context(domain, data)
    output_domain = dp.lazyframe_domain([
        dp.series_domain("cluster__prediction", dp.atom_domain(T=dp.i32)),
        dp.series_domain("distances__distance", dp.atom_domain(T=dp.f64)),
        dp.series_domain("y", dp.atom_domain(T=dp.i32, bounds=(0, 1))),
    ])
    executed = []

    def stable_inference(value):
        executed.append(True)
        return pl.DataFrame({
            "cluster__prediction": pl.Series(
                [int(v >= 0.5) for v in value.X[:, 0]], dtype=pl.Int32
            ),
            "distances__distance": pl.Series(
                np.abs(value.X[:, 0].astype(np.float64) - 0.5),
                dtype=pl.Float64,
            ),
            "y": pl.Series(value.y, dtype=pl.Int32),
        }).lazy()

    transform = _make_transformation(
        domain, dp.symmetric_distance(),
        output_domain, dp.symmetric_distance(),
        stable_inference, lambda d: d,
    )
    query = ctx.query().new_with(chain=transform).lazy()
    assert executed == []
    assert query.collect_schema() == {
        "cluster__prediction": pl.Int32,
        "distances__distance": pl.Float64,
        "y": pl.Int32,
    }
    with pytest.raises(ValueError, match="already determines column names"):
        ctx.query().new_with(chain=transform).lazy(feature_names=["anything"])

    dp_query = (
        query
        .filter(pl.col("cluster__prediction") == 1)
        .select(
            dp.len(signed=True),
            pl.col("distances__distance").dp.sum((0.0, 1.0)),
        )
    )
    assert executed == []
    answer = dp_query.release().collect()
    assert executed == [True]
    assert answer.shape == (1, 2)


def test_sklearn_groupby_with_public_keys():
    domain, data = _space()
    ctx = _context(domain, data)
    keys = pl.LazyFrame([pl.Series("y", [0, 1], dtype=pl.Int32)])
    grouped = (
        ctx.query()
        .lazy(target_name="y")
        .group_by("y")
        .agg(dp.len(signed=True))
        .with_keys(keys)
    )
    released = grouped.release().collect().sort("y")
    assert released.get_column("y").to_list() == [0, 1]
    assert released.get_column("len").dtype in (pl.Int32, pl.Int64)
    assert ctx.current_privacy_loss() == [1.0]


def test_measured_query_cannot_be_made_lazy():
    domain, data = _space()
    ctx = _context(domain, data)
    query = ctx.query().lazy().select(dp.len(signed=True))
    with pytest.raises((ValueError, AttributeError)):
        query._query.new_with(chain=query.resolve()).lazy()





def test_default_lazy_hides_target_unless_explicitly_included():
    domain, data = _space()
    ctx = _context(domain, data)
    plain = ctx.query().lazy()
    assert plain.collect_schema() == {
        "x0": pl.Float32, "x1": pl.Float32,
    }
    explicit = ctx.query().lazy(target_name="outcome")
    assert explicit.collect_schema() == {
        "x0": pl.Float32, "x1": pl.Float32, "outcome": pl.Int32,
    }
    assert ctx.current_privacy_loss() == []


def test_dataframe_features_and_series_target_can_enter_polars_lazily():
    domain = dp.sklearn.sklearn_fit_domain(
        dp.dataframe_domain([
            dp.series_domain("feature", dp.atom_domain(T=dp.f64, nan=False)),
            dp.series_domain("text", dp.atom_domain(T=str)),
        ]),
        dp.series_domain("label", dp.atom_domain(T=dp.i32)),
    )
    y = pl.Series("label", [0, 1], dtype=pl.Int32)
    data = dp.sklearn.SklearnFit(
        pl.DataFrame({
            "feature": pl.Series([0.25, 0.75], dtype=pl.Float64),
            "text": pl.Series(["a", "b"], dtype=pl.String),
        }), y,
    )
    ctx = _context(domain, data)
    plain = ctx.query().lazy()
    assert plain.collect_schema() == {"feature": pl.Float64, "text": pl.String}
    with pytest.raises(ValueError, match="already determined"):
        ctx.query().lazy(feature_names=("other", "text"))
    included = ctx.query().lazy(target_name="outcome")
    assert included.collect_schema() == {
        "feature": pl.Float64, "text": pl.String, "outcome": pl.Int32,
    }
    assert included._query._chain.output_domain.member(included._query._chain(data))
    assert ctx.current_privacy_loss() == []
