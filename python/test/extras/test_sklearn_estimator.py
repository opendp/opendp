from typing import cast

import pytest

from opendp.extras.sklearn._estimator import _DPEstimator, _DPFitMixin
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
