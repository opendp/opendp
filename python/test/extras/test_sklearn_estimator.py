import pickle
import subprocess
import sys

import numpy as np
import pytest
import opendp.prelude as dp
pytest.importorskip("sklearn")
from sklearn.base import TransformerMixin, clone
from sklearn.utils.validation import check_is_fitted

from opendp.extras.sklearn._estimator import _DPEstimator, _DPTransformer


class _ConstantEstimator(_DPEstimator):
    def __init__(self, value=23):
        self.value = value

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        value = self.value
        return dp.m.make_user_measurement(
            input_domain, input_metric, output_measure,
            lambda _data: value, lambda distance: d_out * distance / d_in, TO=int,
        )

    def _ingest_release(self, release):
        self.release_ = release

    def predict(self, X):
        check_is_fitted(self)
        return np.full(len(X), self.release_)


class _OffsetTransformer(TransformerMixin, _DPTransformer):
    def __init__(self, offset):
        self.offset = offset

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        offset = self.offset.copy()
        return dp.m.make_user_measurement(
            input_domain, input_metric, output_measure,
            lambda _data: offset.copy(), lambda distance: d_out * distance / d_in, TO="Vec<f64>",
        )

    def _ingest_release(self, release):
        self.offset_ = np.asarray(release).copy()

    def transform(self, X):
        check_is_fitted(self)
        return X + self.offset_

    def make_transform(self, input_domain, input_metric):
        from opendp._internal import _make_transformation

        check_is_fitted(self)
        offset = self.offset_.copy()
        return _make_transformation(
            input_domain, input_metric, input_domain, input_metric,
            lambda data: dp.sklearn.SklearnFit(data.X + offset, data.y), lambda distance: distance,
        )


def _space():
    return dp.sklearn.sklearn_fit_domain(dp.vector_domain(dp.atom_domain(T=int))), dp.symmetric_distance()


def _context():
    return dp.Context.compositor(
        dp.sklearn.SklearnFit([1, 2, 3]), dp.unit_of(contributions=1), dp.loss_of(epsilon=1),
        domain=_space()[0], split_evenly_over=1,
    )


def test_estimator_is_abstract_and_uses_sklearn_params():
    with pytest.raises(TypeError):
        _DPEstimator()
    estimator = _ConstantEstimator(value=[1, 2])
    assert estimator.get_params() == {"value": [1, 2]}
    assert estimator.set_params(value=[3]) is estimator
    cloned = clone(estimator)
    assert cloned is not estimator
    assert cloned.value == [3] and cloned.value is not estimator.value


def test_fit_requires_explicit_privacy():
    for kwargs in ({}, {"privacy_unit": dp.unit_of(contributions=1)},
                   {"privacy_loss": dp.loss_of(epsilon=1)}):
        with pytest.raises(TypeError, match="explicit privacy_unit and privacy_loss"):
            _ConstantEstimator().fit([1, 2, 3], **kwargs)
    with pytest.raises(TypeError, match="unexpected keyword argument 'unknown'"):
        _ConstantEstimator().fit(  # type: ignore[call-arg]
            [1, 2, 3], privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1), unknown=True,
        )


@pytest.mark.parametrize("parameter", ["sample_weight", "groups"])
def test_fit_rejects_unsupported_parameters(parameter):
    with pytest.raises(TypeError, match=f"unexpected keyword argument '{parameter}'"):
        _ConstantEstimator().fit(
            [1, 2, 3], privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1), **{parameter: [1, 1, 1]},
        )


def test_standalone_and_context_use_identical_measurement_arguments():
    calls = []

    class RecordingEstimator(_ConstantEstimator):
        def make(self, *args):
            calls.append(args)
            return super().make(*args)

    domain = _space()[0]
    standalone = RecordingEstimator()
    assert standalone.fit(
        [1, 2, 3], domain=domain, privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1),
    ) is standalone
    context = dp.Context.compositor(
        dp.sklearn.SklearnFit([1, 2, 3]), dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1), domain=domain, split_evenly_over=1,
    )
    model = RecordingEstimator()
    fitted = context.query().sklearn(model).release()
    assert calls[0] == calls[1]
    assert fitted.release_ == standalone.release_ == 23
    assert fitted is model


def test_make_does_not_fit_or_mutate_estimator():
    estimator = _ConstantEstimator()
    domain, metric = _space()
    measurement = estimator.make(domain, metric, dp.max_divergence(), 1, 1.0)
    estimator.set_params(value=24)
    assert measurement(dp.sklearn.SklearnFit([1, 2, 3])) == 23
    assert measurement.map(1) == 1
    assert vars(estimator) == {"value": 24}


def test_context_constructs_once_and_executes_only_on_release():
    calls = []

    class RecordingEstimator(_ConstantEstimator):
        def make(self, input_domain, input_metric, output_measure, d_in, d_out):
            calls.append("make")
            value = self.value

            def release(_data):
                calls.append("execute")
                return value

            return dp.m.make_user_measurement(
                input_domain, input_metric, output_measure, release,
                lambda distance: d_out * distance / d_in, TO=int,
            )

    context = _context()
    estimator = RecordingEstimator()
    estimator.release_ = 99
    query = context.query().sklearn(estimator)
    query.resolve()
    query.resolve()
    assert calls == ["make"]
    assert estimator.release_ == 99
    assert context.current_privacy_loss() == []
    estimator.set_params(value=24)
    assert query.release() is estimator
    assert estimator.release_ == 23
    assert calls == ["make", "execute"]
    assert context.current_privacy_loss() == [1.0]
    with pytest.raises(dp.OpenDPException, match="out of queries"):
        query.release()
    assert estimator.release_ == 23
    assert calls == ["make", "execute"]
    assert context.current_privacy_loss() == [1.0]
    with pytest.raises(ValueError, match="exhausted"):
        context.query()


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_refitting_replaces_state_and_preserves_sklearn_compatibility(route):
    model = _ConstantEstimator()
    domain = _space()[0]
    context = dp.Context.compositor(
        dp.sklearn.SklearnFit([1, 2, 3]), dp.unit_of(contributions=1),
        dp.loss_of(epsilon=2), domain=domain, split_evenly_over=2,
    )
    for value in (23, 24):
        model.set_params(value=value)
        if route == "standalone":
            fitted = model.fit(
                [1, 2, 3], domain=domain, privacy_unit=dp.unit_of(contributions=1),
                privacy_loss=dp.loss_of(epsilon=1),
            )
        else:
            fitted = context.query().sklearn(model).release()
        assert fitted is model
        assert model.release_ == value
        assert model.get_params() == {"value": value}
        np.testing.assert_array_equal(model.predict([0, 1]), [value, value])
        restored = pickle.loads(pickle.dumps(model))
        np.testing.assert_array_equal(restored.predict([0, 1]), [value, value])
        assert set(vars(restored)) == {"value", "release_"}
        assert clone(model).get_params() == model.get_params()
        assert not hasattr(clone(model), "release_")
    assert context.current_privacy_loss() == ([1.0, 1.0] if route == "context" else [])


def test_repeated_query_release_refits_the_same_estimator():
    model = _ConstantEstimator()
    query = dp.Query(_space(), dp.max_divergence(), d_in=1, d_out=1.0).sklearn(model)
    assert query.release(data=dp.sklearn.SklearnFit([1, 2, 3])) is model
    model.release_ = 99
    model.set_params(value=24)
    assert query.release(data=dp.sklearn.SklearnFit([1, 2, 3])) is model
    assert model.release_ == 23


def test_sklearn_receives_transformed_domain_metric_and_distance():
    from opendp._internal import _make_transformation

    input_domain = dp.sklearn.sklearn_fit_domain(dp.numpy.array2_domain(T=float, num_columns=2))
    output_domain = dp.sklearn.sklearn_fit_domain(dp.numpy.array2_domain(T=float, num_columns=1))
    input_metric, output_metric = dp.change_one_distance(), dp.symmetric_distance()
    duplicate = _make_transformation(
        input_domain, input_metric, output_domain, output_metric,
        lambda data: dp.sklearn.SklearnFit(np.repeat(data.X[:, :1], 2, axis=0)),
        lambda distance: distance * 4,
    )
    data = dp.sklearn.SklearnFit(np.ones((2, 2)))

    class CheckingEstimator(_ConstantEstimator):
        def make(self, input_domain, input_metric, output_measure, d_in, d_out):
            assert input_domain == output_domain and input_metric == output_metric
            assert output_measure == dp.max_divergence()
            assert d_in == 4 and d_out == 1.0

            def release(value):
                np.testing.assert_array_equal(value.X, np.repeat(data.X[:, :1], 2, axis=0))
                return 23

            return dp.m.make_user_measurement(
                input_domain, input_metric, output_measure, release,
                lambda distance: d_out * distance / d_in, TO=int,
            )

    context = dp.Context.compositor(
        data, (input_metric, 1), dp.loss_of(epsilon=1),
        domain=input_domain, split_evenly_over=1,
    )
    query = context.query().new_with(chain=duplicate)
    model = CheckingEstimator()
    fitted_query = query.sklearn(model)
    assert fitted_query.resolve().map(1) == 1
    assert fitted_query.release() is model
    assert model.release_ == 23
    assert query._chain is duplicate
    assert context.current_privacy_loss() == [1.0]


def test_rejected_measurement_leaves_fitted_state_and_budget_unchanged():
    calls = []

    class OverbudgetEstimator(_ConstantEstimator):
        def make(self, input_domain, input_metric, output_measure, d_in, d_out):
            value = self.value

            def release(_data):
                calls.append("execute")
                return value

            return dp.m.make_user_measurement(
                input_domain, input_metric, output_measure, release,
                lambda distance: 2 * d_out * distance / d_in, TO=int,
            )

    context = _context()
    model = OverbudgetEstimator()
    model.release_ = 99
    query = context.query().sklearn(model)
    with pytest.raises(dp.OpenDPException):
        query.release()
    assert calls == []
    assert vars(model) == {"value": 23, "release_": 99}
    assert context.current_privacy_loss() == []
    assert context.query().sklearn(_ConstantEstimator()).release().release_ == 23


def test_sklearn_preserves_measure_conversion():
    class PureEstimator(_ConstantEstimator):
        def make(self, input_domain, input_metric, output_measure, d_in, d_out):
            assert output_measure == dp.zero_concentrated_divergence()
            assert d_in == 1 and d_out == 1.0
            return super().make(input_domain, input_metric, dp.max_divergence(), d_in, 1.0)

    context = dp.Context.compositor(
        dp.sklearn.SklearnFit([1, 2, 3]), dp.unit_of(contributions=1),
        dp.loss_of(rho=1), domain=_space()[0], split_evenly_over=1,
    )
    model = PureEstimator()
    query = context.query().sklearn(model)
    measurement = query.resolve()
    assert measurement.output_measure == dp.zero_concentrated_divergence()
    assert measurement.map(1) == pytest.approx(0.5)
    assert query.release() is model
    assert model.release_ == 23
    assert context.current_privacy_loss() == [1.0]


def test_sklearn_rejects_unrelated_estimators_and_unresolved_chains():
    from opendp.context import PartialChain

    query = _context().query()
    with pytest.raises(ValueError, match="OpenDP fitting capability"):
        query.sklearn(object())
    partial = dp.Query(PartialChain(lambda _: None), dp.max_divergence(), d_in=1, d_out=1)
    with pytest.raises(ValueError, match="requires all arguments"):
        partial.sklearn(_ConstantEstimator())


def test_fit_infers_public_dtype_and_width_and_keeps_targets_private():
    X, y = np.ones((2, 1)), [0, 1]

    class SupervisedEstimator(_ConstantEstimator):
        def make(self, input_domain, input_metric, output_measure, d_in, d_out):
            desc = input_domain.cast(dp.sklearn.SklearnFitDomain)
            assert (desc.X.descriptor.size, desc.X.descriptor.num_columns, desc.X.descriptor.norm) == (None, 1, None)
            assert desc.X.descriptor.T == "f64"
            assert desc.y is not None

            def release(data):
                assert data.X is X and data.y is y
                return 23

            return dp.m.make_user_measurement(
                input_domain, input_metric, output_measure, release,
                lambda distance: distance * d_out / d_in, TO=int,
            )

    estimator = SupervisedEstimator()
    estimator.fit(X, y, privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(epsilon=1))
    assert estimator.release_ == 23
    assert set(vars(estimator)) == {"value", "release_"}


def test_transformer_constructors_snapshot_configuration_and_fitted_parameters():
    data = dp.sklearn.SklearnFit(np.ones((2, 2)), [0, 1])
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=2), dp.vector_domain(dp.atom_domain(T=int)),
    )
    model = _OffsetTransformer(np.array([1.0, 2.0]))
    measurement = model.make(domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0)
    assert not hasattr(model, "offset_")
    model.offset[:] = [3.0, 4.0]
    model.set_params(offset=np.array([5.0, 6.0]))
    assert measurement.map(1) == 1.0
    model._ingest_release(measurement(data))
    np.testing.assert_array_equal(model.offset_, [1.0, 2.0])

    transform = model.make_transform(domain, dp.symmetric_distance())
    assert transform.output_domain == domain
    assert transform.output_metric == dp.symmetric_distance()
    model.offset_[:] = 99
    model.fit(data.X, data.y, domain=domain,
              privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(epsilon=1))
    result = transform(data)
    np.testing.assert_array_equal(result.X, data.X + [1.0, 2.0])
    assert result.y is data.y
    assert transform.output_domain.member(result)
    assert transform.map(2) == 2
    np.testing.assert_array_equal(model.transform(data.X), data.X + [5.0, 6.0])


def test_inherited_fit_transform_uses_dp_fit():
    X, y = np.ones((2, 2)), [0, 1]
    model = _OffsetTransformer(np.array([1.0, 2.0]))
    result = model.fit_transform(
        X, y, privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(epsilon=1),
    )
    np.testing.assert_array_equal(result, X + [1.0, 2.0])
    np.testing.assert_array_equal(model.offset_, [1.0, 2.0])
    assert clone(model).get_params()["offset"] is not model.offset
    restored = pickle.loads(pickle.dumps(model))
    np.testing.assert_array_equal(restored.transform(X), result)


def test_other_opendp_apis_import_without_sklearn():
    script = '''
import importlib.abc
import sys
class NoSklearn(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname == "sklearn" or fullname.startswith("sklearn."):
            raise ModuleNotFoundError("sklearn is unavailable", name=fullname)
sys.meta_path.insert(0, NoSklearn())
import opendp.prelude as dp
assert dp.symmetric_distance() is not None
try:
    dp.sklearn.decomposition
except ImportError:
    pass
else:
    raise AssertionError("sklearn functionality must require sklearn")
'''
    subprocess.run([sys.executable, "-c", script], check=True)


def test_deferred_sklearn_fit_api_is_absent():
    assert not hasattr(dp.sklearn, "make_sklearn_fit")
    assert not hasattr(dp.sklearn, "then_sklearn_fit")
    assert not hasattr(dp.Query, "sklearn_fit")
