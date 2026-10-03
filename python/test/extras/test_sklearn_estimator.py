from typing import cast

import pytest

from opendp.extras.sklearn._estimator import _DPEstimator, _DPFitMixin, _DPXEstimator
from opendp.mod import Queryable


class _DummyEstimator(_DPEstimator):
    def __init__(self, marker="default"):
        self.marker = marker

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        raise NotImplementedError

    def _ingest_release(self, release):
        self.release_ = release


class _BadQueryEstimator(_DummyEstimator):
    def _prepare_fit_query(self, X, y=None, **fit_params):
        return object()


class _ConstantEstimator(_DPEstimator):
    """Test-only estimator with a deterministic, estimator-specific release."""

    def __init__(self, value=23):
        self.value = value

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        import opendp.prelude as dp

        return dp.m.make_user_measurement(
            input_domain,
            input_metric,
            output_measure,
            lambda _data: self.value,
            lambda _d_in: d_out,
            TO=int,
        )

    def _ingest_release(self, release):
        self.release_ = release


class _MixinOnlyEstimator(_DPFitMixin):
    """Exercises the OpenDP fitting capability without sklearn BaseEstimator."""

    def __init__(self, value=23):
        self.value = value

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        return _ConstantEstimator(self.value).make(
            input_domain, input_metric, output_measure, d_in, d_out
        )

    def _ingest_release(self, release):
        self.release_ = release


def _domain_metric():
    import opendp.prelude as dp

    return (
        dp.vector_domain(dp.atom_domain(T=int), size=3),
        dp.symmetric_distance(),
    )


def _context():
    """Build a minimal Context whose queryable invokes its supplied measurement."""
    import opendp.prelude as dp

    domain, metric = _domain_metric()
    accountant = dp.m.make_user_measurement(
        domain,
        metric,
        dp.max_divergence(),
        lambda _data: 0,
        lambda _d_in: 0.0,
        TO=int,
    )
    return dp.Context(
        accountant=accountant,
        queryable=cast(Queryable, lambda measurement: measurement([1, 2, 3])),
        d_in=1,
        d_mids=[1.0],
    )


def test_sklearn_estimator_is_abstract():
    with pytest.raises(TypeError):
        _DPEstimator()

    with pytest.raises(NotImplementedError):
        _DPEstimator.make(
            _DummyEstimator(), None, None, None, 1, 1  # type: ignore[arg-type]
        )
    with pytest.raises(NotImplementedError):
        _DPEstimator._ingest_release(_DummyEstimator(), None)


def test_sklearn_estimator_clone_and_params():
    pytest.importorskip("sklearn")
    from sklearn.base import clone

    estimator = _DummyEstimator(marker=[1, 2])
    assert estimator.get_params() == {"marker": [1, 2]}
    assert estimator.set_params(marker=[3]).marker == [3]
    cloned = clone(estimator)
    assert cloned is not estimator
    assert cloned.get_params() == {"marker": [3]}


def test_make_returns_measurement_and_direct_release_does_not_ingest():
    import opendp.prelude as dp

    domain, metric = _domain_metric()
    estimator = _ConstantEstimator()
    initial_state = estimator.__dict__.copy()
    measurement = estimator.make(domain, metric, dp.max_divergence(), 1, 1.0)

    assert isinstance(measurement, dp.Measurement)
    assert measurement.map(1) <= 1.0
    assert estimator.__dict__ == initial_state
    assert measurement([1, 2, 3]) == 23
    assert not hasattr(estimator, "release_")


def test_then_defers_make_and_does_not_mutate_estimator():
    import opendp.prelude as dp

    estimator = _ConstantEstimator()
    initial_state = estimator.__dict__.copy()
    partial = estimator.then(dp.max_divergence(), 1, 1.0)

    assert estimator.__dict__ == initial_state
    domain, metric = _domain_metric()
    measurement = partial(domain, metric)
    assert isinstance(measurement, dp.Measurement)
    assert estimator.__dict__ == initial_state


def test_fit_requires_query():
    estimator = _DummyEstimator()
    with pytest.raises(TypeError, match="expects X to be a Query"):
        estimator.fit([[1.0]])  # type: ignore[arg-type]


def test_fit_rejects_unsupported_metadata():
    import opendp.prelude as dp

    estimator = _DummyEstimator()
    query = dp.Query(
        (dp.atom_domain(T=float), dp.absolute_distance(T=float)),
        dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    with pytest.raises(TypeError, match="Unexpected fit parameters"):
        estimator.fit(query, sample_weight=[1.0])
    with pytest.raises(TypeError, match="does not accept y"):
        estimator.fit(query, y=[1.0])
    with pytest.raises(TypeError, match="must return an OpenDP Query"):
        _BadQueryEstimator().fit(query)


def test_context_fit_and_query_sklearn_release_are_equivalent():
    fit_estimator = _ConstantEstimator()
    release_estimator = _ConstantEstimator()

    assert fit_estimator.fit(_context().query()) is fit_estimator  # type: ignore[arg-type]
    assert release_estimator is _context().query().sklearn(release_estimator).release()
    assert fit_estimator.release_ == release_estimator.release_ == 23


def test_query_sklearn_accepts_fitting_mixin_and_rejects_unrelated_object():
    import opendp.prelude as dp

    domain, metric = _domain_metric()
    query = dp.Query((domain, metric), dp.max_divergence(), d_in=1, d_out=1.0)
    estimator = _MixinOnlyEstimator()
    assert query.sklearn(estimator).release(data=[1, 2, 3]) is estimator
    assert estimator.release_ == 23

    with pytest.raises(ValueError, match="OpenDP fitting capability"):
        query.sklearn(object())


def test_query_sklearn_accepts_transformed_query_and_rejects_partial_chain():
    import opendp.prelude as dp
    from opendp.context import PartialChain

    domain = dp.vector_domain(dp.atom_domain(T=float, nan=False), size=3)
    metric = dp.symmetric_distance()
    transformation = (domain, metric) >> dp.t.then_clamp((0.0, 1.0))
    transformed = dp.Query(transformation, dp.max_divergence(), d_in=1, d_out=1.0)
    assert isinstance(transformed.sklearn(_ConstantEstimator()), dp.Query)

    partial = dp.Query(
        PartialChain(lambda _scale: transformation),
        dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    with pytest.raises(ValueError, match="requires all arguments"):
        partial.sklearn(_ConstantEstimator())


class _XOnlyEstimator(_ConstantEstimator, _DPXEstimator):
    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        import opendp.prelude as dp

        assert not isinstance(getattr(input_domain, "descriptor", None), dp.sklearn.AlignedDomain)
        return super().make(input_domain, input_metric, output_measure, d_in, d_out)


@pytest.mark.parametrize("preprocess", [False, True])
def test_X_only_sklearn_bridge_projects_without_changing_original_query(preprocess):
    import numpy as np
    import opendp.prelude as dp

    dp.enable_features("contrib")
    X_domain = dp.numpy.array2_domain(T=float, num_columns=2, size=2)
    y_domain = dp.vector_domain(dp.atom_domain(T=int), size=2)
    domain = dp.sklearn.aligned_domain(X_domain, y_domain)
    data = dp.sklearn.Aligned(np.ones((2, 2)), [0, 1])
    context = dp.Context.compositor(
        data, dp.unit_of(contributions=1), dp.loss_of(epsilon=1.0),
        domain=domain, split_evenly_over=2,
    )
    query = context.query()
    if preprocess:
        clamp = dp.numpy.make_np_clamp(X_domain, dp.symmetric_distance(), norm=1.0, p=2)
        query = query.lift_X(clamp)
    original_chain = query._chain
    estimator = _XOnlyEstimator()
    fitted_query = query.sklearn(estimator)
    measurement = fitted_query.resolve()
    assert measurement.input_domain == domain
    assert measurement.map(1) <= 0.5
    assert fitted_query.release() is estimator
    assert estimator.release_ == 23
    second_estimator = _XOnlyEstimator()
    assert second_estimator.fit(cast(dp.Query, query)) is second_estimator
    assert second_estimator.release_ == 23
    assert query._chain is original_chain
    assert data.y == [0, 1]


def test_X_only_sklearn_bridge_leaves_natural_query_unchanged():
    estimator = _XOnlyEstimator()
    assert estimator.fit(_context().query()) is estimator
    assert estimator.release_ == 23


def test_X_only_bridge_receives_preprocessed_X_and_upstream_stability():
    import numpy as np
    import opendp.prelude as dp
    from opendp._internal import _make_transformation

    dp.enable_features("contrib")
    metric = dp.symmetric_distance()
    X_domain = dp.numpy.array2_domain(T=float, num_columns=2, size=2)
    domain = dp.sklearn.aligned_domain(X_domain)
    duplicated_X = dp.numpy.array2_domain(T=float, num_columns=2, size=4)
    duplicate = _make_transformation(
        domain, metric, dp.sklearn.aligned_domain(duplicated_X), metric,
        lambda data: dp.sklearn.Aligned(np.repeat(data.X, 2, axis=0)),
        lambda distance: distance * 2,
    )
    clip = dp.numpy.make_np_clamp(duplicated_X, metric, norm=1.0, p=2)
    data = dp.sklearn.Aligned(np.array([[3.0, 4.0], [0.0, 2.0]]))
    expected = clip(np.repeat(data.X, 2, axis=0))

    class CheckingEstimator(_DPXEstimator):
        def make(self, input_domain, input_metric, output_measure, d_in, d_out):
            assert input_domain == clip.output_domain
            assert d_in == 2

            def release(X):
                np.testing.assert_array_equal(X, expected)
                return 23  # A constant release; assertions only validate routing.

            return dp.m.make_user_measurement(
                input_domain, input_metric, output_measure, release,
                lambda distance: d_out * distance / d_in, TO=int,
            )

        def _ingest_release(self, release):
            self.release_ = release

    query = dp.Query(duplicate, dp.max_divergence(), d_in=1, d_out=1.0).lift_X(clip)
    estimator = CheckingEstimator()
    fitted_query = query.sklearn(estimator)
    assert fitted_query.resolve().map(1) == 1.0
    assert fitted_query.release(data=data) is estimator
    assert estimator.release_ == 23
