"""Real OpenDP tests for private and stable sklearn Pipeline composition."""

import numpy as np
import pytest

pytest.importorskip("sklearn")
import opendp.prelude as dp

from opendp._internal import _make_measurement, _make_transformation
from opendp.mod import OpenDPException
from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from sklearn.preprocessing import StandardScaler
from sklearn.base import BaseEstimator, ClusterMixin
from sklearn.frozen import FrozenEstimator

dp.enable_features("contrib")


class _Shift(MeasurementMixin, TransformationMixin, BaseEstimator):
    def __init__(self, shift=2, fragile=False):
        self.shift = shift
        self.fragile = fragile

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        shift = int(self.shift)
        return _make_measurement(
            input_domain,
            input_metric,
            output_measure,
            lambda _data: shift,
            lambda d: d_out * d / d_in,
        )

    def _ingest_release(self, release):
        self.shift_ = int(release)

    def rowwise_output_domain(self, input_domain, *, method):
        if method != "transform":
            raise ValueError(f"Unsupported method {method!r}")
        if self.fragile and self.shift_ > 0:
            raise ValueError("Cannot construct a transformation from this DP release")
        desc = input_domain.cast(dp.numpy.NPArray2Domain)
        return dp.numpy.array2_domain(
            T=desc.T, size=desc.size, num_columns=desc.num_columns, nan=desc.nan
        )

    def transform(self, X):
        return X + self.shift_


class _Predictor(MeasurementMixin, TransformationMixin, BaseEstimator):
    def __init__(self, label=17):
        self.label = label

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        numeric = dp.sklearn.make_sklearn_fit_numpy(input_domain, input_metric)
        label = int(self.label)
        measurement = _make_measurement(
            numeric.output_domain,
            numeric.output_metric,
            output_measure,
            lambda _data: label,
            lambda d: d_out * d / numeric.map(d_in),
        )
        return numeric >> measurement

    def _ingest_release(self, release):
        self.label_ = int(release)

    def predict(self, X):
        return np.full(len(X), self.label_)

    def rowwise_output_domain(self, input_domain, *, method):
        if method != "predict":
            raise ValueError(f"Unsupported method {method!r}")
        self.label_
        return dp.numpy.array2_domain(
            T=dp.i64,
            size=input_domain.cast(dp.numpy.NPArray2Domain).size,
            num_columns=1,
        )


def _domain(y=False):
    X = dp.numpy.array2_domain(T=float, num_columns=2, nan=False)
    if not y:
        return dp.sklearn.sklearn_fit_domain(X)
    target = dp.vector_domain(dp.atom_domain(T=int))
    return dp.sklearn.sklearn_fit_domain(X, target)


def _data(y=False):
    X = np.ones((2, 2))
    return dp.sklearn.SklearnFit(X, [0, 1] if y else None)


def _certify(model, domain, method="transform"):
    from opendp.extras.sklearn._rowwise import _make_rowwise_transform

    return _make_rowwise_transform(
        domain, dp.symmetric_distance(), model, method=method
    )


def _context(budget=4.0, y=False, n=1):
    return dp.Context.compositor(
        _data(y),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=budget),
        domain=_domain(y),
        split_evenly_over=n,
    )


def test_pipeline_children_need_trusted_inference():
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    model = LogisticRegression(classes=(0, 1))
    model._ingest_release(np.array([2.0, -1.0, 0.0]))
    supported = dp.sklearn.Pipeline([("predictor", FrozenEstimator(model))])
    unchecked = dp.sklearn.Pipeline([("shift", _Shift())])
    unchecked.steps[0][1].shift_ = 2

    class UnreviewedFrozen(FrozenEstimator):
        pass

    unsafe_wrapper = dp.sklearn.Pipeline([("predictor", UnreviewedFrozen(model))])

    assert _certify(supported, _domain(), method="predict").map(1) == 1
    assert _certify(unchecked, _domain()).map(1) == 1

    dp.disable_features("honest-but-curious")
    try:
        assert _certify(supported, _domain(), method="predict").map(1) == 1
        with pytest.raises(OpenDPException, match="honest-but-curious"):
            _certify(unchecked, _domain())
        with pytest.raises(OpenDPException, match="honest-but-curious"):
            _certify(unsafe_wrapper, _domain(), method="predict")
    finally:
        dp.enable_features("honest-but-curious")


def test_frozen_classifier_is_the_final_pipeline_step():
    trained = _context().query().sklearn(_Predictor(label=9)).release()
    frozen = FrozenEstimator(trained)
    X = np.ones((2, 2))
    pipeline = dp.sklearn.Pipeline([("classifier", frozen)])

    assert not pipeline._has_private_fit()
    assert pipeline.fit(X) is pipeline
    assert pipeline.named_steps["classifier"] is frozen
    assert not hasattr(pipeline, "transform")
    np.testing.assert_array_equal(pipeline.predict(X), [9, 9])

    invalid = dp.sklearn.Pipeline([("classifier", frozen), ("later", _Predictor())])
    with pytest.raises(TypeError, match="cannot transform records"):
        _context().query().sklearn(invalid)


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
    source = dp.sklearn.Pipeline(
        [
            ("preprocess", inner),
            ("predictor", _Predictor(5)),
        ],
        privacy_weights={"preprocess": 3, "predictor": 1},
    )
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
        fitted_inner.transform(np.ones((2, 2))),
        np.full((2, 2), 4.0),
    )
    np.testing.assert_array_equal(
        fitted.predict(np.ones((2, 2))),
        [5, 5],
    )
    assert ctx.current_privacy_loss() == [4.0]


def test_nested_frozen_pipeline_needs_no_fitting():
    trained = _context().query().sklearn(_Shift(shift=2)).release()
    frozen = FrozenEstimator(trained)
    stable = dp.sklearn.Pipeline(
        [
            ("inner", dp.sklearn.Pipeline([("frozen", frozen)])),
        ]
    )
    transform = _certify(stable, _domain())
    assert transform.map(2) == 2
    out = transform(_data())
    np.testing.assert_array_equal(out.X, np.full((2, 2), 3.0))
    assert transform.output_domain.member(out)

    combined = dp.sklearn.Pipeline(
        [
            ("stable", stable),
            ("predictor", _Predictor()),
        ]
    )
    fitted = _context().query().sklearn(combined).release()
    reused = fitted.named_steps["stable"].named_steps["inner"].named_steps["frozen"]
    assert reused is not frozen
    assert reused.estimator is not trained
    assert reused.estimator.shift_ == 2
    np.testing.assert_array_equal(fitted.predict(np.ones((2, 2))), [17, 17])


def test_prior_stability_map_is_used_before_fitting():
    ctx = _context()
    transform = _make_transformation(
        _domain(),
        dp.symmetric_distance(),
        _domain(),
        dp.symmetric_distance(),
        lambda value: value,
        lambda d: 2 * d,
    )
    query = (
        ctx.query()
        .new_with(chain=transform)
        .sklearn(dp.sklearn.Pipeline([("predict", _Predictor())]))
    )
    assert query.resolve().check(1, 4.0)
    fitted = query.release()
    assert fitted.named_steps["predict"].label_ == 17
    assert ctx.current_privacy_loss() == [4.0]


def test_failing_after_dp_release_is_accounted():
    source = dp.sklearn.Pipeline(
        [
            ("fragile", _Shift(shift=1, fragile=True)),
            ("predict", _Predictor()),
        ]
    )
    ctx = _context()
    with pytest.raises(OpenDPException, match="Cannot construct a transformation"):
        ctx.query().sklearn(source).release()
    assert not hasattr(source.named_steps["fragile"], "shift_")
    # The failed outer invocation has still used its privacy allowance.
    with pytest.raises(Exception):
        ctx.query().sklearn(
            dp.sklearn.Pipeline(
                [
                    ("predict", _Predictor()),
                ]
            )
        ).release()


def test_no_unprotected_fitting_shortcuts():
    from opendp.extras.sklearn._estimator import MeasurementMixin

    model = dp.sklearn.Pipeline([("predict", _Predictor())])
    # Pipeline.fit supports stable-only stages, but never fits private stages without a budget.
    assert not hasattr(model, "fit_transform")
    assert not hasattr(model, "fit_predict")
    with pytest.raises(TypeError, match="Private fit requires"):
        model.fit(np.ones((2, 2)))

    transformer = dp.sklearn.Pipeline([("shift", _Shift())])
    assert hasattr(transformer, "fit_transform")
    assert not hasattr(transformer, "fit_predict")
    # A nominally stable Pipeline cannot expose private records when a child fits.
    with pytest.raises(NotImplementedError, match="Private fit_transform"):
        transformer.fit_transform(np.ones((2, 2)))
    assert not hasattr(transformer.named_steps["shift"], "shift_")

    class _ClusterPredictor(MeasurementMixin, ClusterMixin, BaseEstimator):
        def make_measurement(self, *args):
            return _Predictor().make_measurement(*args)

        def _ingest_release(self, release):
            self.label_ = int(release)

        def predict(self, X):
            return np.full(len(X), self.label_)

    cluster = dp.sklearn.Pipeline([("cluster", _ClusterPredictor())])
    assert hasattr(cluster, "fit_predict")
    assert not hasattr(cluster, "fit_transform")
    with pytest.raises(NotImplementedError, match="Private fit_predict"):
        cluster.fit_predict(np.ones((2, 2)))
    assert not hasattr(cluster.named_steps["cluster"], "label_")


def test_reject_uncertified_ordinary_sklearn_steps():
    pipeline = dp.sklearn.Pipeline(
        [
            ("unsafe", StandardScaler()),
            ("predictor", _Predictor()),
        ]
    )
    ctx = _context()
    with pytest.raises(TypeError, match="trusted stable"):
        ctx.query().sklearn(pipeline)
    assert ctx.current_privacy_loss() == []


@pytest.mark.parametrize("later", [_Predictor(), None, "passthrough"])
def test_reject_untransformable_intermediate_before_spending_budget(later):
    model = dp.sklearn.Pipeline([("predictor", _Predictor()), ("later", later)])
    ctx = _context()
    with pytest.raises(TypeError, match="cannot transform records"):
        ctx.query().sklearn(model)
    assert ctx.current_privacy_loss() == []


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

    # Direct measurement construction must also snapshot frozen stages,
    # independently of Context's estimator snapshot.
    shift = _Shift(shift=0, fragile=True)
    shift._ingest_release(0)
    model = dp.sklearn.Pipeline(
        [("shift", FrozenEstimator(shift)), ("predict", _Predictor())]
    )
    measurement = model.make_measurement(
        _domain(), dp.symmetric_distance(), dp.max_divergence(), 1, 4.0
    )
    shift.shift_ = 1  # Without a snapshot the next certification would fail.
    fitted = dict(measurement(_data()))
    assert fitted["shift"].estimator.shift_ == 0
    assert fitted["predict"].label_ == 17
    again = dict(measurement(_data()))
    assert again["shift"] is not fitted["shift"]
    assert again["predict"] is not fitted["predict"]


def test_frozen_transformer_as_final_private_pipeline_stage():
    trained = _context().query().sklearn(_Shift(shift=2)).release()
    final = FrozenEstimator(trained)
    source = dp.sklearn.Pipeline([("private", _Shift(shift=3)), ("final", final)])
    ctx = _context()
    fitted = ctx.query().sklearn(source).release()
    assert fitted.named_steps["final"] is not final
    np.testing.assert_array_equal(
        fitted.transform(np.ones((2, 2))), np.full((2, 2), 6.0)
    )
    assert ctx.current_privacy_loss() == [4.0]


def test_mixed_frozen_and_unfrozen_pipeline():
    trained = _context().query().sklearn(_Shift(shift=2)).release()
    frozen = FrozenEstimator(trained)
    model = dp.sklearn.Pipeline(
        [
            ("reuse", frozen),
            ("new", _Predictor(label=11)),
        ]
    )
    ctx = _context()
    query = ctx.query().sklearn(model)
    trained.shift_ = 99
    fitted = query.release()
    reused = fitted.named_steps["reuse"]
    assert reused is not frozen
    assert reused.estimator is not trained
    assert trained.shift_ == 99
    assert reused.estimator.shift_ == 2
    np.testing.assert_array_equal(
        reused.transform(np.zeros((2, 2))),
        np.full((2, 2), 2.0),
    )
    assert fitted.named_steps["new"].label_ == 11
    assert ctx.current_privacy_loss() == [4.0]

    only_frozen = dp.sklearn.Pipeline([("reuse", frozen)])
    stable_ctx = _context()
    query = stable_ctx.query().sklearn(only_frozen)
    assert query.resolve(allow_transformations=True).map(1) == 1
    assert stable_ctx.current_privacy_loss() == []


def test_polars_column_transformer_retains_target_for_next_stage():
    pl = pytest.importorskip("polars")

    class CheckTarget(_Predictor):
        def make_measurement(self, domain, metric, measure, d_in, d_out):
            descriptor = domain.cast(dp.sklearn.SklearnFitDescriptor)
            assert isinstance(descriptor.y, dp.SeriesDomain)
            assert isinstance(descriptor.X, dp.DataFrameDomain)
            assert descriptor.X.columns == ["keep__feature"]
            return super().make_measurement(domain, metric, measure, d_in, d_out)

    frame = pl.DataFrame(
        {
            "feature": pl.Series([1.0, 2.0], dtype=pl.Float64),
            "target": pl.Series([0, 1], dtype=pl.Int32),
        }
    )
    schema = dp.dataframe_domain(
        [
            dp.series_domain("feature", dp.atom_domain(T=dp.f64, nan=False)),
            dp.series_domain("target", dp.atom_domain(T=dp.i32)),
        ]
    )
    ctx = dp.Context.compositor(
        frame,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=2.0),
        domain=schema,
        split_evenly_over=1,
    )
    model = dp.sklearn.Pipeline(
        [
            (
                "features",
                dp.sklearn.ColumnTransformer(
                    [
                        ("keep", "passthrough", ["feature"]),
                    ]
                ).set_output(transform="polars"),
            ),
            ("predictor", CheckTarget(label=5)),
        ]
    )
    fitted = ctx.query().sklearn(model, y="target").release()
    assert fitted.named_steps["predictor"].label_ == 5
    assert ctx.current_privacy_loss() == [2.0]


def test_final_ingestion_failure_is_after_accounting():
    class BadIngest(_Predictor):
        def _ingest_release(self, release):
            raise ValueError("bad release ingestion")

    ctx = _context()
    with pytest.raises(OpenDPException, match="bad release ingestion"):
        ctx.query().sklearn(dp.sklearn.Pipeline([("predict", BadIngest())])).release()
    # Rust reserved the only allowance before the fitted release was ingested.
    # Python bookkeeping of failed calls is fixed separately.
    with pytest.raises(OpenDPException):
        ctx.query().sklearn(dp.sklearn.Pipeline([("predict", _Predictor())])).release()


def test_pipeline_nested_as_rowwise_column_transformer_branch():

    class RowwiseShift(_Shift):
        def rowwise_output_domain(self, input_domain, *, method):
            if method != "transform":
                raise ValueError(f"Unsupported method {method!r}")
            self.shift_
            x = input_domain.cast(dp.numpy.NPArray2Domain)
            return dp.numpy.array2_domain(T=dp.f64, num_columns=x.num_columns, nan=True)

    inner = dp.sklearn.ColumnTransformer(
        [
            ("first", "passthrough", [0]),
        ]
    )
    pipeline = dp.sklearn.Pipeline(
        [
            ("select", inner),
            ("shift", RowwiseShift(3)),
        ]
    )
    source = dp.sklearn.ColumnTransformer(
        [
            ("nested", pipeline, [0, 1]),
            ("original", "passthrough", [1]),
        ]
    )
    ctx = _context(y=True)
    fitted = ctx.query().sklearn(source).release()
    assert ctx.current_privacy_loss() == [4.0]

    domain = _domain(y=True)
    transform = _certify(fitted, domain)
    result = transform(_data(y=True))
    np.testing.assert_array_equal(result.X, [[4.0, 1.0], [4.0, 1.0]])
    assert result.y == [0, 1]
    assert transform.output_domain.member(result)

    # Fitted pipelines may be reused as trusted frozen rowwise branches.
    frozen = FrozenEstimator(fitted.named_transformers_["nested"])
    reuse = dp.sklearn.ColumnTransformer(
        [
            ("reuse", frozen, [0, 1]),
        ]
    )
    stable = _certify(reuse, domain)
    np.testing.assert_array_equal(
        stable(_data(y=True)).X,
        [[4.0], [4.0]],
    )


def test_predictor_pipeline_branch_rejected_before_accounting():
    pipeline = dp.sklearn.Pipeline(
        [
            ("predictor", _Predictor()),
        ]
    )
    outer = dp.sklearn.ColumnTransformer(
        [
            ("bad", pipeline, [0]),
        ]
    )
    ctx = _context()
    with pytest.raises(TypeError, match="fit predictors separately"):
        ctx.query().sklearn(outer)
    assert ctx.current_privacy_loss() == []


def test_pipeline_rowwise_predict_and_predict_proba():
    class RowwisePredictor(_Predictor):
        def predict_proba(self, X):
            return np.tile([[0.25, 0.75]], (len(X), 1))

        def rowwise_output_domain(self, input_domain, *, method):
            self.label_
            width = {"predict": 1, "predict_proba": 2}.get(method)
            if width is None:
                raise ValueError(f"Unsupported inference method {method!r}")
            return dp.numpy.array2_domain(
                T=dp.i64 if method == "predict" else dp.f64,
                num_columns=width,
            )

    model = dp.sklearn.Pipeline(
        [
            (
                "features",
                dp.sklearn.ColumnTransformer(
                    [
                        ("first", "passthrough", [0]),
                    ]
                ),
            ),
            ("predictor", RowwisePredictor(label=11)),
        ]
    )
    ctx = _context(y=True)
    fitted = ctx.query().sklearn(model).release()
    assert ctx.current_privacy_loss() == [4.0]
    cases: list[tuple[str, list[list[float]]]] = [
        ("predict", [[11], [11]]),
        ("predict_proba", [[0.25, 0.75], [0.25, 0.75]]),
    ]
    for method, expected in cases:
        domain = _domain(y=True)
        trans = _certify(fitted, domain, method=method)
        assert trans.map(1) == 1
        result = trans(_data(y=True))
        np.testing.assert_array_equal(result.X, expected)
        assert result.y == [0, 1]
        assert trans.output_domain.member(result)
        empty = trans(dp.sklearn.SklearnFit(np.empty((0, 2)), []))
        assert empty.X.shape == (0, len(expected[0]))
        assert trans.output_domain.member(empty)


def test_stable_only_pipeline_fits_without_budget():
    from sklearn.utils.validation import check_is_fitted

    X = np.ones((2, 2))
    fitted_shift = _Shift(shift=2)
    fitted_shift._ingest_release(2)
    frozen = FrozenEstimator(fitted_shift)
    source = dp.sklearn.Pipeline([("frozen", frozen)])
    assert source.fit(X) is source
    check_is_fitted(source)
    np.testing.assert_array_equal(source.transform(X), np.full((2, 2), 3.0))
    with pytest.raises(ValueError, match="does not consume"):
        source.fit(
            X,
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1),
        )


def test_stable_child_metadata_is_prepared_before_private_fit():
    class PublicShift(TransformationMixin, BaseEstimator):
        def rowwise_output_domain(self, input_domain, *, method):
            assert method == "transform"
            assert self.n_features_in_ == 2
            return input_domain

        def transform(self, X):
            return X + self.n_features_in_

    model = dp.sklearn.Pipeline([("public", PublicShift()), ("private", _Shift())])
    ctx = _context()
    fitted = ctx.query().sklearn(model).release()
    expected = np.full((2, 2), 5.0)
    np.testing.assert_array_equal(fitted.transform(_data().X), expected)
    trans = _certify(fitted, _domain())
    result = trans(_data())
    np.testing.assert_array_equal(result.X, expected)
    assert trans.output_domain.member(result)
    assert trans.map(1) == 1
    assert ctx.current_privacy_loss() == [4.0]


def test_pipeline_rejects_invalid_public_structure():
    for steps, error in [
        ([], "at least one step"),
        ([(1, None)], "Pipeline names must be strings"),
        ([("same", None), ("same", None)], "not unique"),
        ([("privacy_weights", None)], "conflict"),
    ]:
        with pytest.raises(ValueError, match=error):
            dp.sklearn.Pipeline(steps).fit(np.ones((2, 2)))

    with pytest.raises(TypeError, match="memory"):
        dp.sklearn.Pipeline([("skip", None)], memory="cache")


def test_pipeline_rowwise_and_stable_stage_validation():
    from sklearn.utils.validation import check_is_fitted

    features = _domain().cast(dp.sklearn.SklearnFitDescriptor).X
    passthrough = dp.sklearn.Pipeline([("skip", None)])
    assert passthrough.rowwise_output_domain(features, method="transform") == features
    with pytest.raises(ValueError, match="no method"):
        passthrough.rowwise_output_domain(features, method="predict")
    with pytest.raises(TypeError, match="cannot certify"):
        dp.sklearn.Pipeline([("predictor", _Predictor())]).rowwise_output_domain(
            features, method="transform"
        )

    fitted_shift = _Shift(2)
    fitted_shift._ingest_release(2)
    frozen_shift = FrozenEstimator(fitted_shift)
    stable = dp.sklearn.Pipeline([("skip", "passthrough"), ("shift", frozen_shift)])
    assert stable.fit(np.ones((2, 2))) is stable
    check_is_fitted(stable)

    fitted_predictor = _Predictor()
    fitted_predictor._ingest_release(1)
    frozen_predictor = FrozenEstimator(fitted_predictor)
    invalid = dp.sklearn.Pipeline(
        [("predictor", frozen_predictor), ("shift", frozen_shift)]
    )
    with pytest.raises(TypeError, match="cannot transform records"):
        invalid.fit(np.ones((2, 2)))


def test_pipeline_rejects_uncertified_public_final_stage():
    class NoPrivateFit(MeasurementMixin, BaseEstimator):
        def _has_private_fit(self):
            return False

        def make_measurement(self, *args):
            raise AssertionError("Must not fit")

        def _ingest_release(self, release):
            raise AssertionError("Must not ingest")

    model = dp.sklearn.Pipeline([("shift", _Shift()), ("final", NoPrivateFit())])
    with pytest.raises(TypeError, match="not a certified stable stage"):
        model.make_measurement(
            _domain(), dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
        )


def test_pipeline_private_fit_and_passthrough_ingestion():
    X = np.ones((2, 2))
    model = dp.sklearn.Pipeline(
        [
            ("skip", "passthrough"),
            ("shift", _Shift(2)),
            ("pass", None),
            ("predictor", _Predictor(label=5)),
        ]
    )
    assert (
        model.fit(
            X,
            domain=_domain(),
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=2.0),
        )
        is model
    )
    assert model.named_steps["skip"] is None
    assert model.named_steps["pass"] is None
    assert model.named_steps["shift"].shift_ == 2
    np.testing.assert_array_equal(model.predict(X), [5, 5])


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_public_private_pipeline_fit_and_inference_without_hbc(route):
    from opendp.extras.sklearn.linear_model import LogisticRegression

    X = np.array([[1.0, 0.0], [0.0, 2.0]])
    y = np.array([0, 1], dtype=np.int32)
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=False),
        dp.vector_domain(dp.atom_domain(T=dp.i32)),
    )
    privacy_unit = dp.unit_of(contributions=1)
    privacy_loss = dp.loss_of(rho=1.0)
    dp.disable_features("honest-but-curious")
    try:
        model = dp.sklearn.Pipeline(
            [
                (
                    "features",
                    dp.sklearn.ColumnTransformer([("keep", "passthrough", [0, 1])]),
                ),
                (
                    "classifier",
                    LogisticRegression(classes=(0, 1), max_iter=1, reference_size=2),
                ),
            ]
        )
        if route == "standalone":
            fitted = model.fit(
                X,
                y,
                domain=domain,
                privacy_unit=privacy_unit,
                privacy_loss=privacy_loss,
            )
        else:
            ctx = dp.Context.compositor(
                dp.sklearn.SklearnFit(X, y),
                domain=domain,
                privacy_unit=privacy_unit,
                privacy_loss=privacy_loss,
                split_evenly_over=1,
            )
            fitted = ctx.query().sklearn(model).release()
        expected = fitted.predict(X)
        query = dp.Query((domain, dp.symmetric_distance()), dp.max_divergence())
        trans = query.sklearn(fitted, method="predict").resolve(
            allow_transformations=True
        )
        result = trans(dp.sklearn.SklearnFit(X, y))
        np.testing.assert_array_equal(result.X[:, 0], expected)
        np.testing.assert_array_equal(result.y, y)
        assert trans.output_domain.member(result)
        assert trans.map(1) == 1
    finally:
        dp.enable_features("honest-but-curious")


@pytest.mark.parametrize("private", [False, True])
def test_nested_stable_composites_initialize_without_hbc(private):
    from opendp.extras.sklearn.linear_model import LogisticRegression

    X = np.array([[1.0, 0.0], [0.0, 2.0]])
    y = np.array([0, 1], dtype=np.int32)
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=False),
        dp.vector_domain(dp.atom_domain(T=dp.i32)),
    )
    inner = dp.sklearn.Pipeline(
        [("columns", dp.sklearn.ColumnTransformer([("keep", "passthrough", [0, 1])]))]
    )
    columns = dp.sklearn.ColumnTransformer([("nested", inner, [0, 1])])
    stages = [("features", columns)]
    if private:
        stages.append(
            (
                "classifier",
                LogisticRegression(classes=(0, 1), max_iter=1, reference_size=2),
            )
        )
    source = dp.sklearn.Pipeline(stages)
    dp.disable_features("honest-but-curious")
    try:
        if private:
            ctx = dp.Context.compositor(
                dp.sklearn.SklearnFit(X, y),
                dp.unit_of(contributions=1),
                dp.loss_of(rho=0.25),
                domain=domain,
                split_evenly_over=1,
            )
            query = ctx.query().sklearn(source)
            assert ctx.current_privacy_loss() == []
            fitted = query.release()
            assert ctx.current_privacy_loss() == [0.25]
            assert not hasattr(source.named_steps["features"], "transformers_")
            operation = "predict"
        else:
            fitted = source.fit(X, y, domain=domain)
            operation = "transform"
        assert fitted.named_steps["features"].get_feature_names_out().tolist() == [
            "nested__keep__x0",
            "nested__keep__x1",
        ]
        expected = getattr(fitted, operation)(X)
        query = dp.Query((domain, dp.symmetric_distance()), dp.max_divergence())
        trans = query.sklearn(fitted, method=operation).resolve(
            allow_transformations=True
        )
        # Repeated public and compiled inference must use the initialized snapshots.
        for _ in range(2):
            result = trans(dp.sklearn.SklearnFit(X, y))
            np.testing.assert_array_equal(result.X.reshape(expected.shape), expected)
            np.testing.assert_array_equal(getattr(fitted, operation)(X), expected)
            assert trans.output_domain.member(result)
        if private:
            assert ctx.current_privacy_loss() == [0.25]
        fitted.steps = [("changed", None)]
        np.testing.assert_array_equal(
            trans(dp.sklearn.SklearnFit(X, y)).X.reshape(expected.shape), expected
        )
    finally:
        dp.enable_features("honest-but-curious")


def test_public_dataframe_pipeline_without_hbc():
    pl = pytest.importorskip("polars")
    from opendp.extras.sklearn.linear_model import LogisticRegression

    frame = pl.DataFrame(
        {
            "feature": [1.0, 2.0],
            "target": pl.Series([0, 1], dtype=pl.Int32),
        }
    )
    domain = dp.dataframe_domain(
        [
            dp.series_domain("feature", dp.atom_domain(T=dp.f64, nan=False)),
            dp.series_domain("target", dp.atom_domain(T=dp.i32)),
        ]
    )
    dp.disable_features("honest-but-curious")
    try:
        model = dp.sklearn.Pipeline(
            [
                (
                    "features",
                    dp.sklearn.ColumnTransformer(
                        [("keep", "passthrough", ["feature"])]
                    ).set_output(transform="polars"),
                ),
                (
                    "classifier",
                    LogisticRegression(classes=(0, 1), max_iter=1, reference_size=2),
                ),
            ]
        )
        ctx = dp.Context.compositor(
            frame,
            dp.unit_of(contributions=1),
            dp.loss_of(rho=0.25),
            domain=domain,
            split_evenly_over=1,
        )
        fitted = ctx.query().sklearn(model, y="target").release()
        assert ctx.current_privacy_loss() == [0.25]
        expected = fitted.predict(frame.drop("target"))
        query = dp.Query((domain, dp.symmetric_distance()), dp.max_divergence())
        trans = query.sklearn(fitted, y="target", method="predict").resolve(
            allow_transformations=True
        )
        result = trans(frame)
        assert result.X.columns == ["prediction"]
        np.testing.assert_array_equal(result.X["prediction"].to_numpy(), expected)
        assert result.y.equals(frame["target"])
        assert ctx.current_privacy_loss() == [0.25]
    finally:
        dp.enable_features("honest-but-curious")


def test_stable_composite_refit_replaces_stale_public_metadata():
    pl = pytest.importorskip("polars")
    frame = pl.DataFrame({"named": [1.0, 2.0], "other": [3.0, 4.0]})
    columns = dp.sklearn.ColumnTransformer([("keep", "passthrough", [0])])
    columns.set_output(transform="polars").fit(frame)
    assert columns.feature_names_in_.tolist() == ["named", "other"]
    assert columns.get_feature_names_out().tolist() == ["keep__named"]
    columns.fit(frame.to_numpy())
    assert not hasattr(columns, "feature_names_in_")
    assert columns.get_feature_names_out().tolist() == ["keep__x0"]
    assert columns.transform(frame.to_numpy()).columns == ["keep__x0"]


@pytest.mark.parametrize("route", ["predictor", "first", "intermediate", "private"])
def test_pipeline_fit_preserves_reviewed_frozen_state_without_hbc(route):
    from copy import deepcopy
    from opendp.extras.sklearn.linear_model import LogisticRegression

    X = np.array([[1.0, 0.0], [0.0, 2.0]])
    y = np.array([0, 1], dtype=np.int32)
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=False),
        dp.vector_domain(dp.atom_domain(T=dp.i32)),
    )
    classifier = LogisticRegression(classes=(0, 1))
    classifier._ingest_release(np.array([2.0, -1.0, 0.0]))
    frozen_classifier = FrozenEstimator(classifier)
    columns = dp.sklearn.ColumnTransformer([("keep", "passthrough", [0, 1])])
    columns.fit(X)
    frozen_columns = FrozenEstimator(columns)
    wrappers = [frozen_classifier, frozen_columns]
    states = [deepcopy(vars(wrapper.estimator)) for wrapper in wrappers]

    if route == "predictor":
        steps = [("classifier", frozen_classifier)]
        method = "predict"
    elif route == "first":
        steps = [("columns", frozen_columns), ("skip", None)]
        method = "transform"
    elif route == "intermediate":
        steps = [
            ("first", dp.sklearn.ColumnTransformer([("keep", "passthrough", [0, 1])])),
            ("columns", frozen_columns),
            ("classifier", frozen_classifier),
        ]
        method = "predict"
    else:
        steps = [
            ("columns", frozen_columns),
            (
                "classifier",
                LogisticRegression(classes=(0, 1), max_iter=1, reference_size=2),
            ),
        ]
        method = "predict"

    dp.disable_features("honest-but-curious")
    try:
        pipeline = dp.sklearn.Pipeline(steps)
        if route == "private":
            ctx = dp.Context.compositor(
                dp.sklearn.SklearnFit(X, y),
                dp.unit_of(contributions=1),
                dp.loss_of(rho=0.25),
                domain=domain,
                split_evenly_over=1,
            )
            fitted = ctx.query().sklearn(pipeline).release()
            assert ctx.current_privacy_loss() == [0.25]
        else:
            fitted = pipeline.fit(X, y, domain=domain)
        for name, step in steps:
            if isinstance(step, FrozenEstimator):
                if route == "private":
                    assert fitted.named_steps[name] is not step
                else:
                    assert fitted.named_steps[name] is step

        query = dp.Query((domain, dp.symmetric_distance()), dp.max_divergence())
        trans = query.sklearn(fitted, method=method).resolve(allow_transformations=True)
        expected = getattr(fitted, method)(X)
        for _ in range(2):
            result = trans(dp.sklearn.SklearnFit(X, y))
            np.testing.assert_array_equal(result.X.reshape(expected.shape), expected)
            np.testing.assert_array_equal(result.y, y)
            assert trans.output_domain.member(result)
            assert trans.map(1) == 1

        for wrapper, original, state in zip(wrappers, [classifier, columns], states):
            assert vars(wrapper) == {"estimator": original}
            assert vars(original).keys() == state.keys()
            for key, value in state.items():
                if isinstance(value, np.ndarray):
                    np.testing.assert_array_equal(vars(original)[key], value)
                else:
                    assert vars(original)[key] == value
    finally:
        dp.enable_features("honest-but-curious")
