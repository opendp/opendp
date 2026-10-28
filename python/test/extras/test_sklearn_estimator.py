import pickle
import subprocess
import sys

import numpy as np
import pytest
import opendp.prelude as dp

pytest.importorskip("sklearn")
from sklearn.base import TransformerMixin, clone

from opendp.extras.sklearn._estimator import DPEstimator, DPTransformer


class _ConstantEstimator(DPEstimator):
    def __init__(self, value=23):
        self.value = value

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
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


class _OffsetTransformer(TransformerMixin, DPTransformer):
    def __init__(self, offset):
        self.offset = offset

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
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

    def make_transform(self, input_domain, input_metric):
        from opendp._internal import _make_transformation

        offset = self.offset_.copy()
        desc = input_domain.cast(dp.sklearn.SklearnFitDomain)
        X_desc = desc.X.cast(dp.numpy.NPArray2Domain)
        output_domain = dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(
                T=float, size=X_desc.size, num_columns=X_desc.num_columns
            ),
            desc.y,
        )
        return _make_transformation(
            input_domain,
            input_metric,
            output_domain,
            input_metric,
            lambda data: dp.sklearn.SklearnFit(data.X + offset, data.y),
            lambda d: d,
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
        with pytest.raises(TypeError, match="explicit privacy_unit and privacy_loss"):
            _ConstantEstimator().fit([1, 2, 3], **kwargs)


def test_sklearn_requires_an_estimator_and_resolved_input():
    from opendp.context import PartialChain

    with pytest.raises(ValueError, match="OpenDP fitting capability"):
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
        def make(self, *args):
            calls.append("make")

            def execute(value):
                calls.append("execute")
                return value

            return super().make(*args) >> execute

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
        def make(self, domain, metric, measure, d_in, d_out):
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
@pytest.mark.parametrize("loss_factor", [0.5, 2.0])
def test_fitting_enforces_privacy_relation(route, loss_factor):
    executions = []

    class BudgetedEstimator(_ConstantEstimator):
        def make(self, domain, metric, measure, d_in, d_out):
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
                lambda d: loss_factor * d_out * d / d_in,
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

    if loss_factor > 1:
        with pytest.raises(ValueError, match="exceeds"):
            fit()
        assert executions == []
        assert model.release_ == 99
        assert context.current_privacy_loss() == []
    else:
        fitted = fit()
        assert fitted.release_ == 23 and executions == [True]
        assert (fitted is model) == (route == "standalone")
        assert model.release_ == (23 if route == "standalone" else 99)
        assert context.current_privacy_loss() == ([1.0] if route == "context" else [])


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_sklearn_preserves_measure_conversion(route):
    class PureEstimator(_ConstantEstimator):
        def make(self, domain, metric, measure, d_in, d_out):
            assert measure == dp.zero_concentrated_divergence()
            return super().make(domain, metric, dp.max_divergence(), d_in, 1.0)

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
    measurement = model.make(
        domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
    )
    model.offset[:] = 9
    parameters = measurement(data)
    assert not hasattr(model, "offset_")
    model._ingest_release(parameters)
    transform = model.make_transform(domain, dp.symmetric_distance())
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


def test_other_opendp_apis_import_without_sklearn():
    script = """
import importlib.abc
import sys
import pytest
class NoSklearn(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname == "sklearn" or fullname.startswith("sklearn."):
            raise ModuleNotFoundError("sklearn is unavailable", name=fullname)
sys.meta_path.insert(0, NoSklearn())
import opendp.prelude as dp
assert dp.symmetric_distance() is not None
dp.enable_features("contrib")
data = dp.sklearn.SklearnFit([1, 2], [0, 1])
assert dp.domain_of(data, infer=True).member(data)
for name in ("decomposition", "linear_model"):
    with pytest.raises(ImportError):
        getattr(dp.sklearn, name)
with pytest.raises(ModuleNotFoundError):
    importlib.import_module("opendp.extras.sklearn._estimator")
"""
    subprocess.run([sys.executable, "-c", script], check=True)
