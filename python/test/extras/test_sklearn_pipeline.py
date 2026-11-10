"""Real OpenDP tests for private and stable sklearn Pipeline composition."""

import numpy as np
import pytest

pytest.importorskip("sklearn")
import opendp.prelude as dp

from opendp._internal import _make_transformation
from opendp.extras.sklearn._estimator import DPEstimator, DPTransformer
from sklearn.preprocessing import StandardScaler

dp.enable_features("contrib")


class _Shift(DPTransformer):
    def __init__(self, shift=2, fragile=False):
        self.shift = shift
        self.fragile = fragile

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        shift = int(self.shift)
        return dp.m.make_user_measurement(
            input_domain, input_metric, output_measure,
            lambda _data: shift,
            lambda d: d_out * d / d_in,
            TO=int,
        )

    def _ingest_release(self, release):
        self.shift_ = int(release)

    def make_transform(self, input_domain, input_metric):
        if self.fragile and self.shift_ > 0:
            raise ValueError("Cannot construct a transformation from this DP release")
        shift = int(self.shift_)
        return _make_transformation(
            input_domain, input_metric, input_domain, input_metric,
            lambda data: dp.sklearn.SklearnFit(data.X + shift, data.y),
            lambda d: d,
        )

    def transform(self, X):
        return X + self.shift_


class _Predictor(DPEstimator):
    def __init__(self, label=17):
        self.label = label

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        numeric = dp.sklearn.make_sklearn_fit(
            input_domain, input_metric, as_numpy=True,
        )
        label = int(self.label)
        measurement = dp.m.make_user_measurement(
            *numeric.output_space, output_measure,
            lambda _data: label,
            lambda d: d_out * d / numeric.map(d_in),
            TO=int,
        )
        return numeric >> measurement

    def _ingest_release(self, release):
        self.label_ = int(release)

    def predict(self, X):
        return np.full(len(X), self.label_)


def _domain(y=False):
    X = dp.numpy.array2_domain(T=float, num_columns=2, nan=False)
    if not y:
        return dp.sklearn.sklearn_fit_domain(X)
    target = dp.vector_domain(dp.atom_domain(T=int))
    return dp.sklearn.sklearn_fit_domain(X, target)


def _data(y=False):
    X = np.ones((2, 2))
    return dp.sklearn.SklearnFit(X, [0, 1] if y else None)


def _context(budget=4., y=False, n=1):
    return dp.Context.compositor(
        _data(y),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=budget),
        domain=_domain(y),
        split_evenly_over=n,
    )


def test_single_private_fit_returns_independent_clone():
    predictor = _Predictor()
    source = dp.sklearn.Pipeline([("predictor", predictor)])
    ctx = _context()
    query = ctx.query().sklearn(source)
    assert ctx.current_privacy_loss() == []
    fitted = query.release()
    assert fitted is not source
    assert fitted.named_steps["predictor"] is not predictor
    assert not hasattr(predictor, "label_")
    assert fitted.named_steps["predictor"].label_ == 17
    np.testing.assert_array_equal(fitted.predict(np.ones((2, 2))), [17, 17])
    assert ctx.current_privacy_loss() == [4.0]

def test_nested_private_pipelines():
    one = _Shift(1)
    two = _Shift(2)
    inner = dp.sklearn.Pipeline([("one", one), ("two", two)])
    source = dp.sklearn.Pipeline([
        ("preprocess", inner),
        ("predictor", _Predictor(5)),
    ], privacy_weights={"preprocess": 3, "predictor": 1})
    ctx = _context()
    fitted = ctx.query().sklearn(source).release()
    fitted_inner = fitted.named_steps["preprocess"]
    assert fitted_inner is not inner
    assert fitted_inner.named_steps["one"].shift_ == 1
    assert fitted_inner.named_steps["two"].shift_ == 2
    assert not hasattr(one, "shift_")
    assert not hasattr(two, "shift_")
    assert not hasattr(source.named_steps["predictor"], "label_")
    np.testing.assert_array_equal(
        fitted_inner.transform(np.ones((2, 2))), np.full((2, 2), 4.),
    )
    np.testing.assert_array_equal(
        fitted.predict(np.ones((2, 2))), [5, 5],
    )
    assert ctx.current_privacy_loss() == [4.0]


def test_nested_frozen_pipeline_needs_no_fitting():
    trained = _context().query().sklearn(_Shift(shift=2)).release()
    frozen = trained.as_transformer()
    stable = dp.sklearn.Pipeline([
        ("inner", dp.sklearn.Pipeline([("frozen", frozen)])),
    ])
    transform = stable.make_transform(_domain(), dp.symmetric_distance())
    assert transform.map(2) == 2
    out = transform(_data())
    np.testing.assert_array_equal(out.X, np.full((2, 2), 3.))
    assert transform.output_domain.member(out)

    combined = dp.sklearn.Pipeline([
        ("stable", stable),
        ("predictor", _Predictor()),
    ])
    fitted = _context().query().sklearn(combined).release()
    assert fitted.named_steps["stable"].named_steps["inner"].named_steps[
        "frozen"
    ] is frozen
    np.testing.assert_array_equal(fitted.predict(np.ones((2, 2))), [17, 17])

def test_private_transform_and_target_alignment():
    ctx = _context(y=True)
    source = dp.sklearn.Pipeline([
        ("first", _Shift(1)),
        ("second", _Shift(2)),
    ])
    fitted = ctx.query().sklearn(source).release()
    transform = fitted.make_transform(_domain(y=True), dp.symmetric_distance())
    assert transform.map(1) == 1
    result = transform(_data(y=True))
    np.testing.assert_array_equal(result.X, np.full((2, 2), 4.))
    assert result.y == [0, 1]
    assert transform.output_domain.member(result)
    assert ctx.current_privacy_loss() == [4.0]


def test_prior_stability_map_is_used_before_fitting():
    ctx = _context()
    transform = _make_transformation(
        _domain(), dp.symmetric_distance(),
        _domain(), dp.symmetric_distance(),
        lambda value: value,
        lambda d: 2 * d,
    )
    query = ctx.query().new_with(chain=transform).sklearn(
        dp.sklearn.Pipeline([("predict", _Predictor())])
    )
    assert query.resolve().check(1, 4.0)
    fitted = query.release()
    assert fitted.named_steps["predict"].label_ == 17
    assert ctx.current_privacy_loss() == [4.0]


def test_failing_after_dp_release_is_accounted():
    source = dp.sklearn.Pipeline([
        ("fragile", _Shift(shift=1, fragile=True)),
        ("predict", _Predictor()),
    ])
    ctx = _context()
    with pytest.raises(ValueError, match="Cannot construct a transformation"):
        ctx.query().sklearn(source).release()
    assert not hasattr(source.named_steps["fragile"], "shift_")
    # The failed outer invocation has still used its privacy allowance.
    with pytest.raises(Exception):
        ctx.query().sklearn(dp.sklearn.Pipeline([
            ("predict", _Predictor()),
        ])).release()


def test_no_unprotected_fitting_shortcuts():
    model = dp.sklearn.Pipeline([("predict", _Predictor())])
    with pytest.raises(NotImplementedError):
        model.fit_predict(np.ones((2, 2)))
    with pytest.raises(NotImplementedError):
        model.fit_transform(np.ones((2, 2)))
    with pytest.raises(TypeError):
        model.fit(np.ones((2, 2)))


def test_reject_uncertified_ordinary_sklearn_steps():
    pipeline = dp.sklearn.Pipeline([
        ("unsafe", StandardScaler()), ("predictor", _Predictor()),
    ])
    ctx = _context()
    with pytest.raises(TypeError, match="trusted stable"):
        ctx.query().sklearn(pipeline)
    assert ctx.current_privacy_loss() == []


def test_reject_untransformable_intermediate_before_spending_budget():
    model = dp.sklearn.Pipeline([
        ("predictor", _Predictor()),
        ("later", _Predictor()),
    ])
    ctx = _context()
    with pytest.raises(TypeError, match="cannot transform records"):
        ctx.query().sklearn(model)
    assert ctx.current_privacy_loss() == []


@pytest.mark.parametrize("weights", [
    {"first": 0, "second": 1},
    {"first": 1},
    {"first": float("nan"), "second": 1},
])
def test_bad_privacy_weights(weights):
    model = dp.sklearn.Pipeline([
        ("first", _Shift(1)),
        ("second", _Predictor()),
    ], privacy_weights=weights)
    with pytest.raises(ValueError, match="privacy_weights"):
        _context().query().sklearn(model)


def test_parameters_snapshotted_before_release():
    predictor = _Predictor(23)
    model = dp.sklearn.Pipeline([("predictor", predictor)])
    ctx = _context()
    query = ctx.query().sklearn(model)
    predictor.set_params(label=99)
    model.set_params(privacy_weights=None)
    fitted = query.release()
    assert fitted.named_steps["predictor"].label_ == 23
    assert predictor.label == 99
    assert ctx.current_privacy_loss() == [4.0]


def test_standalone_private_fit():
    model = dp.sklearn.Pipeline([("predictor", _Predictor(3))])
    assert model.fit(
        _data().X,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        domain=_domain(),
    ) is model
    assert model.named_steps["predictor"].label_ == 3
    np.testing.assert_array_equal(model.predict(np.ones((2, 2))), [3, 3])

def test_mixed_frozen_and_unfrozen_pipeline():
    trained = _context().query().sklearn(_Shift(shift=2)).release()
    frozen = trained.as_transformer("transform")
    model = dp.sklearn.Pipeline([
        ("reuse", frozen),
        ("new", _Predictor(label=11)),
    ])
    ctx = _context()
    fitted = ctx.query().sklearn(model).release()
    assert fitted.named_steps["reuse"] is frozen
    assert fitted.named_steps["new"].label_ == 11
    assert ctx.current_privacy_loss() == [4.0]

    only_frozen = dp.sklearn.Pipeline([("reuse", frozen)])
    stable_ctx = _context()
    query = stable_ctx.query().sklearn(only_frozen)
    assert query.resolve(allow_transformations=True).map(1) == 1
    assert stable_ctx.current_privacy_loss() == []


def test_dataframe_column_transformer_then_numpy_predictor():
    pl = pytest.importorskip("polars")

    class _PrivateRow(_Shift):
        def make_rowwise(self, input_domain):
            from opendp.extras.sklearn._frozen import _RowwiseSpec

            features = input_domain.cast(dp.sklearn.SklearnFitDescriptor).X
            numeric = (
                dp.numpy.make_dataframe_to_numpy(features, dp.symmetric_distance())
                if isinstance(features, dp.DataFrameDomain) else None
            )
            shift = float(self.shift_)
            return _RowwiseSpec(
                lambda X: (numeric(X) if numeric is not None else X)[:, :1] + shift,
                dp.numpy.array2_domain(
                    T=dp.f64, num_columns=1, nan=True,
                ),
            )

    frame = pl.DataFrame({
        "age": pl.Series([1., 2.], dtype=pl.Float64),
        "income": pl.Series([3., 4.], dtype=pl.Float64),
        "target": pl.Series([0, 1], dtype=pl.Int32),
    })
    schema = dp.dataframe_domain([
        dp.series_domain("age", dp.atom_domain(T=dp.f64, nan=False)),
        dp.series_domain("income", dp.atom_domain(T=dp.f64, nan=False)),
        dp.series_domain("target", dp.atom_domain(T=dp.i32)),
    ])
    ctx = dp.Context.compositor(
        frame, dp.unit_of(contributions=1),
        dp.loss_of(epsilon=2.), domain=schema, split_evenly_over=1,
    )
    model = dp.sklearn.Pipeline([
        ("features", dp.sklearn.ColumnTransformer([
            ("scaled", _PrivateRow(shift=2), ["age"]),
            ("original", "passthrough", ["income"]),
        ])),
        ("predictor", _Predictor(label=5)),
    ])
    fitted = ctx.query().sklearn(model, y="target").release()
    assert fitted.named_steps["features"]._dp_output == "default"
    np.testing.assert_array_equal(
        fitted.predict(frame.drop("target")), [5, 5],
    )
    assert ctx.current_privacy_loss() == [2.0]



def test_polars_column_transformer_retains_target_for_next_stage():
    pl = pytest.importorskip("polars")

    class CheckTarget(_Predictor):
        def make(self, domain, metric, measure, d_in, d_out):
            descriptor = domain.cast(dp.sklearn.SklearnFitDescriptor)
            assert isinstance(descriptor.y, dp.SeriesDomain)
            assert isinstance(descriptor.X, dp.DataFrameDomain)
            assert descriptor.X.columns == ["keep__feature"]
            return super().make(domain, metric, measure, d_in, d_out)

    frame = pl.DataFrame({
        "feature": pl.Series([1.0, 2.0], dtype=pl.Float64),
        "target": pl.Series([0, 1], dtype=pl.Int32),
    })
    schema = dp.dataframe_domain([
        dp.series_domain("feature", dp.atom_domain(T=dp.f64, nan=False)),
        dp.series_domain("target", dp.atom_domain(T=dp.i32)),
    ])
    ctx = dp.Context.compositor(
        frame, dp.unit_of(contributions=1),
        dp.loss_of(epsilon=2.0), domain=schema, split_evenly_over=1,
    )
    model = dp.sklearn.Pipeline([
        ("features", dp.sklearn.ColumnTransformer([
            ("keep", "passthrough", ["feature"]),
        ]).set_output(transform="polars")),
        ("predictor", CheckTarget(label=5)),
    ])
    fitted = ctx.query().sklearn(model, y="target").release()
    assert fitted.named_steps["predictor"].label_ == 5
    assert ctx.current_privacy_loss() == [2.0]


def test_final_ingestion_failure_is_after_accounting():
    class BadIngest(_Predictor):
        def _ingest_release(self, release):
            raise ValueError("bad release ingestion")

    ctx = _context()
    with pytest.raises(ValueError, match="bad release ingestion"):
        ctx.query().sklearn(dp.sklearn.Pipeline([
            ("predict", BadIngest())
        ])).release()
    assert ctx.current_privacy_loss() == [4.0]


def test_pipeline_nested_as_rowwise_column_transformer_branch():
    from opendp.extras.sklearn._frozen import _RowwiseSpec

    class RowwiseShift(_Shift):
        def make_rowwise(self, input_domain):
            desc = input_domain.cast(dp.sklearn.SklearnFitDescriptor)
            x = desc.X.cast(dp.numpy.NPArray2Domain)
            shift = float(self.shift_)
            return _RowwiseSpec(
                lambda X: X + shift,
                dp.numpy.array2_domain(
                    T=dp.f64, num_columns=x.num_columns, nan=True,
                ),
            )

    inner = dp.sklearn.ColumnTransformer([
        ("first", "passthrough", [0]),
    ])
    pipeline = dp.sklearn.Pipeline([
        ("select", inner),
        ("shift", RowwiseShift(3)),
    ])
    source = dp.sklearn.ColumnTransformer([
        ("nested", pipeline, [0, 1]),
        ("original", "passthrough", [1]),
    ])
    ctx = _context(y=True)
    fitted = ctx.query().sklearn(source).release()
    assert ctx.current_privacy_loss() == [4.0]

    domain = _domain(y=True)
    transform = fitted.make_transform(domain, dp.symmetric_distance())
    result = transform(_data(y=True))
    np.testing.assert_array_equal(result.X, [[4.0, 1.0], [4.0, 1.0]])
    assert result.y == [0, 1]
    assert transform.output_domain.member(result)

    # Fitted pipelines may be reused as trusted frozen rowwise branches.
    frozen = fitted.named_transformers_["nested"].as_transformer()
    reuse = dp.sklearn.ColumnTransformer([
        ("reuse", frozen, [0, 1]),
    ])
    stable = reuse.make_transform(domain, dp.symmetric_distance())
    np.testing.assert_array_equal(
        stable(_data(y=True)).X, [[4.0], [4.0]],
    )


def test_non_rowwise_nested_pipeline_rejected_before_release():
    pipeline = dp.sklearn.Pipeline([
        ("predictor", _Predictor()),
    ])
    outer = dp.sklearn.ColumnTransformer([
        ("bad", pipeline, [0]),
    ])
    ctx = _context()
    with pytest.raises(TypeError, match="not certified rowwise"):
        ctx.query().sklearn(outer)
    assert ctx.current_privacy_loss() == []
