import pytest

from opendp.extras.sklearn import SklearnEstimator


class _DummyEstimator(SklearnEstimator):
    def __init__(self, marker="default"):
        self.marker = marker

    def make(
        self, input_domain, input_metric, output_measure, d_in, d_out
    ):
        raise NotImplementedError

    def _ingest_release(self, release):
        self.release_ = release


class _BadQueryEstimator(_DummyEstimator):
    def _prepare_fit_query(self, X, y=None, **fit_params):
        return object()


class _CountEstimator(SklearnEstimator):
    def __init__(self, scale=1.0):
        self.scale = scale

    def make(
        self, input_domain, input_metric, output_measure, d_in, d_out
    ):
        import opendp.prelude as dp

        return (
            (input_domain, input_metric)
            >> dp.t.then_count()
            >> dp.m.then_laplace(self.scale)
        )

    def _ingest_release(self, release):
        self.count_ = release


def test_sklearn_estimator_is_abstract():
    with pytest.raises(TypeError):
        SklearnEstimator()

    with pytest.raises(NotImplementedError):
        SklearnEstimator.make(
            _DummyEstimator(), None, None, None, 1, 1  # type: ignore[arg-type]
        )
    with pytest.raises(NotImplementedError):
        SklearnEstimator._ingest_release(_DummyEstimator(), None)


def test_sklearn_estimator_clone_and_params():
    pytest.importorskip("sklearn")
    from sklearn.base import clone

    estimator = _DummyEstimator(marker=[1, 2])
    assert estimator.get_params() == {"marker": [1, 2]}
    assert estimator.set_params(marker=[3]).marker == [3]
    cloned = clone(estimator)
    assert cloned is not estimator
    assert cloned.get_params() == {"marker": [3]}


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


def test_query_sklearn_rejects_non_estimator():
    import opendp.prelude as dp

    query = dp.Query(
        (dp.atom_domain(T=float), dp.absolute_distance(T=float)),
        dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    with pytest.raises(ValueError, match="SklearnEstimator"):
        query.sklearn(object())


def test_query_sklearn_accepts_transformed_query_and_rejects_partial_chain():
    import opendp.prelude as dp
    from opendp.context import PartialChain

    domain = dp.vector_domain(dp.atom_domain(T=float, nan=False), size=3)
    metric = dp.symmetric_distance()
    transformation = (domain, metric) >> dp.t.then_clamp((0.0, 1.0))
    transformed = dp.Query(
        transformation, dp.max_divergence(), d_in=1, d_out=1.0
    )
    assert isinstance(transformed.sklearn(_CountEstimator()), dp.Query)

    partial = dp.Query(
        PartialChain(lambda _scale: transformation),
        dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    with pytest.raises(ValueError, match="requires all arguments"):
        partial.sklearn(_CountEstimator())


def test_direct_measurement_and_context_fit_share_estimator_path():
    import opendp.prelude as dp

    domain = dp.vector_domain(dp.atom_domain(T=int), size=3)
    metric = dp.symmetric_distance()
    estimator = _CountEstimator(scale=1.0)
    measurement = estimator.make(
        domain, metric, dp.max_divergence(), 1, 1.0
    )
    assert measurement.map(1) <= 1.0

    context = dp.Context.compositor(
        data=[1, 2, 3],
        domain=domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    assert estimator.fit(context.query()) is estimator  # type: ignore[arg-type]
    assert isinstance(estimator.count_, int)
