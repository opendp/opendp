import pickle

import numpy as np
import pytest
import opendp.prelude as dp

pytest.importorskip("sklearn")
from sklearn.base import BaseEstimator, TransformerMixin, clone

from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin


@pytest.fixture(autouse=True)
def allow_test_rowwise_callbacks():
    # Test doubles assert their own rowwise contracts instead of using library code.
    from opendp.mod import GLOBAL_FEATURES

    enabled = "honest-but-curious" in GLOBAL_FEATURES
    dp.enable_features("honest-but-curious")
    yield
    if not enabled:
        dp.disable_features("honest-but-curious")


class _ConstantEstimator(MeasurementMixin, BaseEstimator):
    def __init__(self, value=23):
        self.value = value

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        value = self.value
        return dp.m.make_user_measurement(
            input_domain,
            input_metric,
            output_measure,
            lambda _data: value,
            lambda d: d_out * d / d_in,
            TO=int,
        )

    def _ingest_release(self, release):
        self.release_ = release


class _OffsetTransformer(
    MeasurementMixin, TransformationMixin, TransformerMixin, BaseEstimator
):
    def __init__(self, offset):
        self.offset = offset

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        offset = self.offset.copy()
        return dp.m.make_user_measurement(
            input_domain,
            input_metric,
            output_measure,
            lambda _data: offset.copy(),
            lambda d: d_out * d / d_in,
            TO="Vec<f64>",
        )

    def _ingest_release(self, release):
        self.offset_ = np.array(release, copy=True)

    def transform(self, X):
        return X + self.offset_

    def rowwise_output_domain(self, input_domain, *, method):
        if method != "transform":
            raise ValueError(f"Unsupported method {method!r}")
        self.offset_
        desc = input_domain.cast(dp.numpy.NPArray2Domain)
        return dp.numpy.array2_domain(
            T=float, size=desc.size, num_columns=desc.num_columns
        )


def _space():
    return (
        dp.sklearn.sklearn_fit_domain(dp.vector_domain(dp.atom_domain(T=int))),
        dp.symmetric_distance(),
    )


def _context():
    return dp.Context.compositor(
        dp.sklearn.SklearnFit([1, 2, 3]),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1),
        domain=_space()[0],
        split_evenly_over=1,
    )


def test_fit_requires_explicit_privacy():
    for kwargs in (
        {},
        {"privacy_unit": dp.unit_of(contributions=1)},
        {"privacy_loss": dp.loss_of(epsilon=1)},
    ):
        with pytest.raises(TypeError, match="required keyword-only"):
            _ConstantEstimator().fit([1, 2, 3], **kwargs)


def test_sklearn_requires_an_estimator_and_resolved_input():
    from opendp.context import PartialChain

    with pytest.raises(TypeError, match="OpenDP fitting"):
        _context().query().sklearn(object())
    partial = dp.Query(
        PartialChain(lambda _: None), dp.max_divergence(), d_in=1, d_out=1
    )
    with pytest.raises(ValueError, match="requires all arguments"):
        partial.sklearn(_ConstantEstimator())


def test_standalone_fit_and_sklearn_compatibility():
    model = _ConstantEstimator().set_params(value=24)
    model.release_ = 99
    assert (
        model.fit(
            [1, 2, 3],
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1),
        )
        is model
    )
    assert model.release_ == 24
    assert model.get_params() == {"value": 24}
    assert pickle.loads(pickle.dumps(model)).release_ == 24
    assert clone(model).get_params() == model.get_params()
    assert not hasattr(clone(model), "release_")


def test_context_constructs_once_and_executes_only_on_release():
    calls = []

    class RecordingEstimator(_ConstantEstimator):
        def make_measurement(self, *args):
            calls.append("make")

            def execute(value):
                calls.append("execute")
                return value

            return super().make_measurement(*args) >> execute

    context = _context()
    model = RecordingEstimator()
    query = context.query().sklearn(model)
    query.resolve()
    query.resolve()
    model.set_params(value=24)
    assert calls == ["make"]
    assert context.current_privacy_loss() == []
    fitted = query.release()
    assert fitted.value == fitted.release_ == 23
    assert calls == ["make", "execute"]
    assert context.current_privacy_loss() == [1.0]
    with pytest.raises(dp.OpenDPException, match="out of queries"):
        query.release()
    assert calls == ["make", "execute"]


def test_context_releases_independent_clones_without_mutating_model():
    X = np.ones((2, 2))
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=2)
    )
    model = _OffsetTransformer(np.array([1.0, 2.0]))
    model.offset_ = np.array([99.0, 99.0])
    context = dp.Context.compositor(
        dp.sklearn.SklearnFit(X),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=2),
        domain=domain,
        split_evenly_over=2,
    )
    query = context.query().sklearn(model)
    model.offset[:] = 9
    first = query.release()
    np.testing.assert_array_equal(first.offset_, [1.0, 2.0])
    first.offset[:] = 10
    first.offset_[:] = 20
    second = query.release()
    assert first is not second and first is not model and second is not model
    np.testing.assert_array_equal(second.offset, [1.0, 2.0])
    np.testing.assert_array_equal(second.offset_, [1.0, 2.0])
    np.testing.assert_array_equal(first.offset_, [20.0, 20.0])
    np.testing.assert_array_equal(model.offset, [9.0, 9.0])
    np.testing.assert_array_equal(model.offset_, [99.0, 99.0])
    assert clone(second).get_params()["offset"] is not second.offset
    assert context.current_privacy_loss() == [1.0, 1.0]


def test_resolved_measurement_supports_postprocessing():
    import gc
    import weakref

    data = dp.sklearn.SklearnFit([1, 2, 3])
    reference = weakref.ref(data)
    query = dp.Query(_space(), dp.max_divergence(), d_in=1, d_out=1.0).sklearn(
        _ConstantEstimator()
    )
    measurement = query.resolve()
    assert measurement(data).release_ == 23
    # Serialize inside DP postprocessing, before the separate FFI invocation-
    # metadata layer. That layer's private-input retention is fixed separately.
    serialized = measurement >> pickle.dumps
    fitted = pickle.loads(query.new_with(chain=serialized).release(data=data))
    assert fitted.release_ == 23
    assert vars(fitted) == {"value": 23, "release_": 23}
    assert serialized.map(1) == measurement.map(1) == 1.0
    del data
    gc.collect()
    assert reference() is None


def test_sklearn_receives_transformed_domain_metric_and_distance():
    from opendp._internal import _make_transformation

    input_domain, input_metric = _space()
    output_domain = dp.sklearn.sklearn_fit_domain(
        dp.vector_domain(dp.atom_domain(T=float))
    )
    output_metric = dp.insert_delete_distance()
    transform = _make_transformation(
        input_domain,
        input_metric,
        output_domain,
        output_metric,
        lambda data: dp.sklearn.SklearnFit([float(x * 2) for x in data.X]),
        lambda d: d * 4,
    )

    class CheckingEstimator(_ConstantEstimator):
        def make_measurement(self, domain, metric, measure, d_in, d_out):
            assert (domain, metric, measure, d_in, d_out) == (
                output_domain,
                output_metric,
                dp.max_divergence(),
                4,
                1.0,
            )
            return dp.m.make_user_measurement(
                domain,
                metric,
                measure,
                lambda data: int(sum(data.X)),
                lambda d: d_out * d / d_in,
                TO=int,
            )

    context = _context()
    query = context.query().new_with(chain=transform).sklearn(CheckingEstimator())
    assert query.resolve().map(1) == 1.0
    assert query.release().release_ == 12
    assert context.current_privacy_loss() == [1.0]


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_fitting_rejects_excess_privacy_loss(route):
    executions = []

    class BudgetedEstimator(_ConstantEstimator):
        def make_measurement(self, domain, metric, measure, d_in, d_out):
            assert (domain, metric, measure, d_in, d_out) == (
                *_space(),
                dp.max_divergence(),
                1,
                1.0,
            )

            def execute(_data):
                executions.append(True)
                return 23

            return dp.m.make_user_measurement(
                domain,
                metric,
                measure,
                execute,
                lambda d: 2 * d_out * d / d_in,
                TO=int,
            )

    context, model = _context(), BudgetedEstimator()
    model.release_ = 99

    def fit():
        if route == "context":
            return context.query().sklearn(model).release()
        return model.fit(
            [1, 2, 3],
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1),
        )

    with pytest.raises(ValueError, match="exceeds"):
        fit()
    assert executions == []
    assert model.release_ == 99
    assert context.current_privacy_loss() == []


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_sklearn_preserves_measure_conversion(route):
    class PureEstimator(_ConstantEstimator):
        def make_measurement(self, domain, metric, measure, d_in, d_out):
            assert measure == dp.zero_concentrated_divergence()
            return super().make_measurement(
                domain, metric, dp.max_divergence(), d_in, 1.0
            )

    model = PureEstimator()
    if route == "context":
        context = dp.Context.compositor(
            dp.sklearn.SklearnFit([1, 2, 3]),
            dp.unit_of(contributions=1),
            dp.loss_of(rho=1),
            domain=_space()[0],
            split_evenly_over=1,
        )
        query = context.query().sklearn(model)
        assert query.resolve().output_measure == dp.zero_concentrated_divergence()
        assert query.resolve().map(1) == pytest.approx(0.5)
        fitted = query.release()
        assert context.current_privacy_loss() == [1.0]
    else:
        fitted = model.fit(
            [1, 2, 3],
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(rho=1),
        )
    assert fitted.release_ == 23


def test_framework_constructors_snapshot_parameters_without_fitting():
    X, y = np.zeros((2, 2)), [0, 1]
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, norm=1, p=2, size=2, num_columns=2),
        dp.vector_domain(dp.atom_domain(T=int)),
    )
    data, model = dp.sklearn.SklearnFit(X, y), _OffsetTransformer(np.array([2.0, 3.0]))
    measurement = model.make_measurement(
        domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
    )
    model.offset[:] = 9
    parameters = measurement(data)
    assert not hasattr(model, "offset_")
    model._ingest_release(parameters)
    from opendp.extras.sklearn._rowwise import _make_rowwise_transform

    transform = _make_rowwise_transform(domain, dp.symmetric_distance(), model)
    model.offset_[:] = 10
    transformed = (transform >> dp.t.make_identity(*transform.output_space))(data)
    np.testing.assert_array_equal(transformed.X, X + [2.0, 3.0])
    assert transformed.y is y
    assert transform.output_domain.member(transformed)
    assert transform.map(2) == 2
    assert (
        model.fit(
            X,
            y,
            domain=domain,
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1),
        )
        is model
    )
    np.testing.assert_array_equal(model.transform(X), X + 9)


def test_conditional_private_fitting_shortcuts_follow_sklearn_roles():
    from sklearn.base import ClusterMixin

    assert not hasattr(_ConstantEstimator(), "fit_transform")
    assert not hasattr(_ConstantEstimator(), "fit_predict")

    transformer = _OffsetTransformer(np.array([1.0, 2.0]))
    assert hasattr(transformer, "fit_transform")
    assert not hasattr(transformer, "fit_predict")
    with pytest.raises(NotImplementedError, match="Private fit_transform"):
        transformer.fit_transform(np.ones((2, 2)))
    assert not hasattr(transformer, "offset_")

    class Cluster(MeasurementMixin, ClusterMixin, BaseEstimator):
        def make_measurement(self, *args):
            return _ConstantEstimator().make_measurement(*args)

        def _ingest_release(self, release):
            self.release_ = release

    cluster = Cluster()
    assert hasattr(cluster, "fit_predict")
    assert not hasattr(cluster, "fit_transform")
    with pytest.raises(NotImplementedError, match="Private fit_predict"):
        cluster.fit_predict(np.ones((2, 2)))
    assert not hasattr(cluster, "release_")


def test_sklearn_fit_metadata_clears_stale_public_column_names():
    from opendp.extras.sklearn._estimator import _set_fit_metadata

    model = _ConstantEstimator()
    named = dp.sklearn.sklearn_fit_domain(
        dp.dataframe_domain(
            [
                dp.series_domain("first", dp.atom_domain(T=dp.f64)),
                dp.series_domain("second", dp.atom_domain(T=dp.f64)),
            ]
        )
    )
    _set_fit_metadata(model, named)
    assert model.n_features_in_ == 2
    np.testing.assert_array_equal(model.feature_names_in_, ["first", "second"])

    unnamed = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2)
    )
    _set_fit_metadata(model, unnamed)
    assert model.n_features_in_ == 2
    assert not hasattr(model, "feature_names_in_")


def test_standalone_sklearn_fit_requires_eager_input_domain():
    lazy = dp.lazyframe_domain([dp.series_domain("feature", dp.atom_domain(T=dp.f64))])
    domain = dp.sklearn.sklearn_fit_domain(lazy)
    with pytest.raises(TypeError, match="Explicitly collect"):
        _ConstantEstimator().fit(
            np.ones((1, 1)),
            domain=domain,
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1),
        )
