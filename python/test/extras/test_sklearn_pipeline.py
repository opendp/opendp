"""Real OpenDP tests for private and stable sklearn Pipeline composition."""

import numpy as np
import pytest

pytest.importorskip("sklearn")
import opendp.prelude as dp

from opendp._internal import _make_transformation
from opendp.mod import OpenDPException
from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from sklearn.preprocessing import StandardScaler
from sklearn.base import BaseEstimator, ClusterMixin
from sklearn.frozen import FrozenEstimator

dp.enable_features("contrib")


@pytest.fixture(autouse=True)
def allow_test_rowwise_callbacks():
    from opendp.mod import GLOBAL_FEATURES

    enabled = "honest-but-curious" in GLOBAL_FEATURES
    dp.enable_features("honest-but-curious")
    yield
    if not enabled:
        dp.disable_features("honest-but-curious")


class _Shift(MeasurementMixin, TransformationMixin, BaseEstimator):
    def __init__(self, shift=2, fragile=False):
        self.shift = shift
        self.fragile = fragile

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        shift = int(self.shift)
        return dp.m.make_user_measurement(
            input_domain,
            input_metric,
            output_measure,
            lambda _data: shift,
            lambda d: d_out * d / d_in,
            TO=int,
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
        measurement = dp.m.make_user_measurement(
            numeric.output_domain,
            numeric.output_metric,
            output_measure,
            lambda _data: label,
            lambda d: d_out * d / numeric.map(d_in),
            TO=int,
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

    model = LogisticRegression()
    model._ingest_release(np.array([2.0, -1.0, 0.0]))
    supported = dp.sklearn.Pipeline([("predictor", FrozenEstimator(model))])
    unchecked = dp.sklearn.Pipeline([("shift", _Shift())])
    unchecked.steps[0][1].shift_ = 2
    overridden = FrozenEstimator(model)
    overridden.predict = lambda X: np.repeat(X[:1, 0], len(X))
    unsafe_wrapper = dp.sklearn.Pipeline([("predictor", overridden)])

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
    assert reused is frozen
    assert reused.estimator is trained
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


def test_reject_untransformable_intermediate_before_spending_budget():
    model = dp.sklearn.Pipeline(
        [
            ("predictor", _Predictor()),
            ("later", _Predictor()),
        ]
    )
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


def test_frozen_transformer_as_final_private_pipeline_stage():
    trained = _context().query().sklearn(_Shift(shift=2)).release()
    final = FrozenEstimator(trained)
    source = dp.sklearn.Pipeline([("private", _Shift(shift=3)), ("final", final)])
    ctx = _context()
    fitted = ctx.query().sklearn(source).release()
    assert fitted.named_steps["final"] is final
    np.testing.assert_array_equal(
        fitted.transform(np.ones((2, 2))), np.full((2, 2), 6.0)
    )
    assert ctx.current_privacy_loss() == [4.0]


def test_stable_only_measurement_cannot_be_inferred():
    class PublicOnly(MeasurementMixin, BaseEstimator):
        def _has_private_fit(self):
            return False

        def make_measurement(self, *args):
            raise AssertionError("Public-only stages must not fit")

        def _ingest_release(self, release):
            raise AssertionError("Public-only stages must not ingest")

    with pytest.raises(
        TypeError, match="Stable inference requires TransformationMixin"
    ):
        _context().query().sklearn(PublicOnly())


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
    fitted = query.release()
    reused = fitted.named_steps["reuse"]
    assert reused is frozen
    assert reused.estimator is trained
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
    for method, expected in [
        ("predict", [[11], [11]]),
        ("predict_proba", [[0.25, 0.75], [0.25, 0.75]]),
    ]:
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


def test_fit_metadata_does_not_recompile_private_pipeline():
    transform_calls = []

    class CountedShift(_Shift):
        def rowwise_output_domain(self, domain, *, method):
            transform_calls.append(True)
            return super().rowwise_output_domain(domain, method=method)

    model = dp.sklearn.Pipeline(
        [
            ("shift", CountedShift()),
            ("predict", _Predictor()),
        ]
    )
    fitted = _context().query().sklearn(model).release()
    assert fitted.n_features_in_ == 2
    assert transform_calls == [True]


def test_pipeline_rejects_invalid_public_structure_and_cache():
    from opendp.extras.sklearn.pipeline import _snapshot

    for steps, error in [
        ([], "at least one step"),
        ([("bad__name", None)], "unique strings"),
        ([("same", None), ("same", None)], "unique strings"),
    ]:
        with pytest.raises(ValueError, match=error):
            _snapshot(steps)

    cached = dp.sklearn.Pipeline([("skip", None)], memory="cache")
    with pytest.raises(NotImplementedError, match="caching"):
        cached.fit(np.ones((2, 2)))
    private = dp.sklearn.Pipeline([("shift", _Shift())], memory="cache")
    with pytest.raises(NotImplementedError, match="caching"):
        private.make_measurement(
            _domain(), dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
        )
    with pytest.raises(ValueError, match="fitting releases"):
        dp.sklearn.Pipeline([("shift", _Shift())])._ingest_release((_domain(), []))


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
    with pytest.raises(TypeError, match="not a sklearn transformer"):
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
