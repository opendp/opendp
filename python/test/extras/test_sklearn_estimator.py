import pickle

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
        with pytest.raises(TypeError, match="required keyword-only"):
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


def test_logistic_regression_rejects_missing_target():
    from opendp.extras.sklearn.linear_model import LogisticRegression

    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(num_columns=3, size=100, nan=False, T=float),
    )
    with pytest.raises(ValueError, match="requires a target domain y"):
        LogisticRegression().make(
            domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(), 2, 1.0
        )


@pytest.mark.parametrize("target_type", [bool, float])
def test_logistic_regression_gradients_preserve_aligned_rows(target_type):
    from opendp.extras.sklearn.linear_model._make_private_logistic_regression import (
        make_private_logistic_regression,
        per_example_gradients,
    )

    X = np.array([[1, 2], [3, 4], [5, 6]], dtype=np.float32)
    y = np.array([0, 1, 0], dtype=target_type)
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(
            num_columns=2,
            size=3,
            nan=False,
            T=np.float32,
            norm=10,
            p=2,
            cardinalities=[3, 3],
        ),
        dp.numpy.array1_domain(T=target_type, nan=False),
    )
    data = dp.sklearn.SklearnFit(X, y)
    assert domain.member(data)
    assert not domain.member(dp.sklearn.SklearnFit(X, y[:-1]))
    theta = np.array([0.2, -0.1, 0.3])
    gradients = per_example_gradients(data, theta)
    features = np.column_stack((X, np.ones(3)))
    expected = (1 / (1 + np.exp(-(features @ theta))) - y)[:, None] * features
    np.testing.assert_allclose(gradients, expected)
    order = [2, 0, 1]
    np.testing.assert_allclose(
        per_example_gradients(dp.sklearn.SklearnFit(X[order], y[order]), theta),
        gradients[order],
    )
    changed_X, changed_y = X.copy(), y.copy()
    changed_X[0] = [-1, -2]
    changed_y[0] = 1
    changed = dp.sklearn.SklearnFit(changed_X, changed_y)
    assert domain.member(changed)
    changed_gradients = per_example_gradients(changed, theta)
    assert not np.array_equal(changed_gradients[0], gradients[0])
    np.testing.assert_array_equal(changed_gradients[1:], gradients[1:])
    measurement = make_private_logistic_regression(
        domain,
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        2,
        0.5,
        n_iters=1,
        learning_rate=0.1,
        clip_norm=1.0,
    )
    assert measurement.input_domain == domain
    assert measurement.map(2) <= 0.5


@pytest.mark.parametrize("extreme_weights", [False, True])
def test_logistic_regression_nonfinite_gradients(extreme_weights):
    from scipy.special import expit  # type: ignore[import-untyped]
    from opendp.extras.sklearn.linear_model import LogisticRegression
    from opendp.extras.sklearn.linear_model._make_private_logistic_regression import (
        per_example_gradients,
    )

    largest = np.finfo(float).max
    smallest = np.finfo(float).smallest_subnormal
    X = np.array(
        [
            [np.nan, 2],
            [np.inf, -np.inf],
            [largest, largest],
            [1, 2],
            [1, 2],
            [1, 2],
            [largest, -largest],
            [smallest, -smallest],
        ]
    )
    y = np.array([1, 0, largest, np.nan, np.inf, -np.inf, -2, 1])
    original_X, original_y = X.copy(), y.copy()
    theta = np.full(3, largest) if extreme_weights else np.array([0.2, -0.1, 0.3])
    data = dp.sklearn.SklearnFit(X, y)
    with np.errstate(all="raise"):
        gradients = per_example_gradients(data, theta)
    assert gradients.shape == (8, 3)
    assert np.isfinite(gradients).all()
    assert dp.numpy.array2_domain(T=float, num_columns=3, nan=False).member(gradients)
    np.testing.assert_array_equal(gradients[3:6], np.zeros((3, 3)))
    imputed = np.array([[0, 2, 1], [0, 0, 1]], dtype=float)
    with np.errstate(over="ignore", invalid="ignore"):
        expected = (expit(imputed @ theta) - y[:2])[:, None] * imputed
    np.testing.assert_allclose(gradients[:2], expected)
    order = [6, 3, 1, 0, 7, 5, 2, 4]
    np.testing.assert_array_equal(
        per_example_gradients(dp.sklearn.SklearnFit(X[order], y[order]), theta),
        gradients[order],
    )
    extended = dp.sklearn.SklearnFit(np.vstack([X, X[:1]]), np.concatenate([y, y[:1]]))
    np.testing.assert_array_equal(
        per_example_gradients(extended, theta)[:-1], gradients
    )
    np.testing.assert_array_equal(X, original_X)
    np.testing.assert_array_equal(y, original_y)
    model = LogisticRegression(reference_size=11, n_iters=2).fit(
        X,
        y,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(rho=1.0),
    )
    assert np.isfinite(model.coef_).all() and np.isfinite(model.intercept_).all()
    with np.errstate(all="raise"):
        proba = model.predict_proba(X)
    assert np.isfinite(proba).all()
    np.testing.assert_allclose(proba.sum(axis=1), 1)


@pytest.mark.parametrize(
    "x_size, y_size, reference_size, learning_rate",
    [
        (3, 3, None, 0.3),
        (3, 3, 3, 0.3),
        (3, None, np.int64(6), 0.6),
        (None, 3, None, 0.3),
        (None, None, 6, 0.6),
    ],
)
def test_logistic_regression_public_normalizer(
    x_size, y_size, reference_size, learning_rate
):
    from scipy.special import expit  # type: ignore[import-untyped]
    from opendp.extras.sklearn.linear_model import LogisticRegression

    # Both initial coefficient updates are nonzero: L1's sign is discontinuous
    # at zero, even with arbitrarily small DP noise.
    X = np.array([[1, 0], [0, 2], [1, 1]], dtype=float)
    y = np.array([1, 0, 1], dtype=float)
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, size=x_size, num_columns=2),
        dp.numpy.array1_domain(T=float, size=y_size),
    )
    model = LogisticRegression(
        n_iters=2,
        learning_rate=learning_rate,
        clip_norm=10,
        alpha=2,
        l1_ratio=0.5,
        reference_size=reference_size,
    )
    d_in = 1 if x_size is None and y_size is None else 2
    measurement = model.make(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(), d_in, 1e12
    )
    assert measurement.map(d_in) <= 1e12
    theta = measurement(dp.sklearn.SklearnFit(X, y))
    expected = np.zeros(3)
    Xb = np.column_stack([X, np.ones(3)])
    for _ in range(2):
        gradients = (expit(Xb @ expected) - y)[:, None] * Xb
        penalty = np.sign(expected) + expected  # alpha=2, l1_ratio=0.5
        penalty[-1] = 0
        expected -= 0.1 * (gradients.sum(axis=0) + penalty)
    np.testing.assert_allclose(theta, expected, atol=1e-4, rtol=0)


def test_logistic_regression_unsized_empty_fit():
    from opendp.extras.sklearn.linear_model import LogisticRegression

    X, y = np.empty((0, 2)), np.empty(0)
    model = LogisticRegression(reference_size=11, n_iters=2).fit(
        X,
        y,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(rho=1.0),
    )
    assert np.isfinite(model.coef_).all()
    assert model.predict_proba(X).shape == (0, 2)
    other = clone(model).fit(
        X,
        y,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(rho=1.0),
    )
    # Empty private data still needs noise under add/remove adjacency.
    assert not np.array_equal(model.coef_, other.coef_)


def test_logistic_regression_validates_input_domains():
    from opendp.extras.sklearn.linear_model import LogisticRegression

    options = dict(num_columns=2, size=3, nan=False, T=float)
    targets = dp.numpy.array1_domain(T=float, nan=False)
    cases: list[tuple[dict, dp.Domain, str]] = [
        ({"T": str}, targets, "numeric element types"),
        ({}, dp.vector_domain(dp.atom_domain(T=float, nan=False)), "NPArray1Domain"),
        (
            {},
            dp.numpy.array2_domain(T=float, size=3, num_columns=1, nan=False),
            "NPArray1Domain",
        ),
        ({}, dp.numpy.array1_domain(T=str), "numeric element types"),
        (
            {},
            dp.numpy.array1_domain(T=float, nan=False, size=2),
            "matching row counts",
        ),
        ({"num_columns": None}, targets, "specify num_columns"),
        ({"size": None}, targets, "reference_size is required"),
        ({"size": 0}, targets, "finite positive integer"),
    ]
    for overrides, y_domain, error in cases:
        domain = dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(**(options | overrides)), y_domain
        )
        with pytest.raises(ValueError, match=error):
            LogisticRegression().make(
                domain,
                dp.symmetric_distance(),
                dp.zero_concentrated_divergence(),
                2,
                1.0,
            )


@pytest.mark.parametrize(
    "params",
    [
        {"alpha": -1.0},
        {"alpha": float("nan")},
        {"alpha": float("inf")},
        {"l1_ratio": -0.1},
        {"l1_ratio": 1.1},
        {"l1_ratio": float("nan")},
        {"l1_ratio": float("inf")},
        {"reference_size": 0},
        {"reference_size": -1},
        {"reference_size": 1.5},
        {"reference_size": True},
        {"reference_size": float("nan")},
        {"reference_size": float("inf")},
        {"reference_size": 10**1000},
    ],
)
def test_logistic_regression_rejects_invalid_public_parameters(params):
    from opendp.extras.sklearn.linear_model import LogisticRegression

    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, size=3, num_columns=2, nan=False),
        dp.numpy.array1_domain(T=float, size=3, nan=False),
    )
    with pytest.raises(ValueError, match="alpha|l1_ratio|reference_size"):
        LogisticRegression(**params).make(
            domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(), 2, 1.0
        )


@pytest.mark.parametrize("sized", [False, True])
@pytest.mark.parametrize("route", ["context", "standalone"])
def test_logistic_regression_fitting(route, sized):
    import opendp.prelude as dp
    from opendp.extras.sklearn.linear_model import LogisticRegression

    np = pytest.importorskip("numpy")
    pytest.importorskip("scipy")

    rng = np.random.default_rng(0)
    X = rng.standard_normal((100, 3))
    y = (X[:, 0] + rng.standard_normal(100) > 0).astype(float)
    data = dp.sklearn.SklearnFit(X, y)

    domain = (
        dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(num_columns=3, size=100, nan=False, T=float),
            dp.numpy.array1_domain(T=float, nan=False, size=100),
        )
        if sized
        else dp.domain_of(data, infer=True)
    )
    d_in = 2 if sized else 1
    metric = dp.symmetric_distance()
    estimator = LogisticRegression(
        n_iters=50,
        learning_rate=0.3,
        clip_norm=5.0,
        reference_size=None if sized else 37,
    )

    measurement = estimator.make(
        domain, metric, dp.zero_concentrated_divergence(), d_in, 1.0
    )
    assert measurement.map(d_in) <= 1.0
    if sized:
        with pytest.raises(ValueError, match="d_in must be even"):
            estimator.make(domain, metric, dp.zero_concentrated_divergence(), 1, 1.0)

    if route == "context":
        context = dp.Context.compositor(
            data=data,
            domain=domain if sized else None,
            privacy_unit=dp.unit_of(contributions=d_in),
            privacy_loss=dp.loss_of(rho=1.0),
            split_evenly_over=1,
        )
        fitted = context.query().sklearn(estimator).release()
        assert fitted is not estimator
        assert not hasattr(estimator, "coef_")
        assert context.current_privacy_loss() == [1.0]
    else:
        fitted = estimator.fit(
            X,
            y,
            domain=domain if sized else None,
            privacy_unit=dp.unit_of(contributions=d_in),
            privacy_loss=dp.loss_of(rho=1.0),
        )
        assert fitted is estimator
    from sklearn.base import clone, is_classifier

    assert is_classifier(estimator) and is_classifier(fitted)
    assert fitted.coef_.shape == (1, 3)
    assert fitted.intercept_.shape == (1,)
    assert fitted.n_features_in_ == 3
    assert list(fitted.classes_) == [0, 1]
    assert clone(fitted).get_params() == fitted.get_params()

    proba = fitted.predict_proba(X)
    assert proba.shape == (100, 2)
    np.testing.assert_allclose(proba.sum(axis=1), 1)
    preds = fitted.predict(X)
    assert set(np.unique(preds)).issubset({0, 1})
    assert fitted.score(X, y) == np.mean(preds == y)
    missing_X = X.copy()
    missing_X[0] = [np.nan, np.inf, -np.inf]
    imputed_X = X.copy()
    imputed_X[0] = 0.0
    np.testing.assert_array_equal(
        fitted.predict_proba(missing_X), fitted.predict_proba(imputed_X)
    )
    np.testing.assert_array_equal(fitted.predict(missing_X), fitted.predict(imputed_X))


# sklearn 1.8 warns on its recommended C=inf path for unregularized fitting.
@pytest.mark.filterwarnings(
    "ignore:Setting penalty=None will ignore the C and l1_ratio parameters:UserWarning"
)
@pytest.mark.parametrize("alpha", [0.0, 1.0])
def test_logistic_regression_matches_sklearn_regularized(alpha):
    import opendp.prelude as dp

    np = pytest.importorskip("numpy")
    pytest.importorskip("scipy")
    pytest.importorskip("sklearn")
    from sklearn.linear_model import LogisticRegression as SkLR
    from opendp.extras.sklearn.linear_model import LogisticRegression

    rng = np.random.default_rng(0)
    X = rng.standard_normal((100, 3))
    y = (X[:, 0] + rng.standard_normal(100) > 0).astype(float)
    data = dp.sklearn.SklearnFit(X, y)
    dom = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(num_columns=3, size=100, nan=False, T=float),
        dp.numpy.array1_domain(T=float, nan=False, size=100),
    )

    est = LogisticRegression(
        n_iters=2000, learning_rate=0.5, clip_norm=100.0, alpha=alpha
    )
    theta_dp = np.asarray(
        est.make(
            dom, dp.symmetric_distance(), dp.zero_concentrated_divergence(), 2, 1e8
        )(data)
    )

    Xb = np.hstack([X, np.ones((100, 1))])
    sk = SkLR(
        C=1.0 / alpha if alpha else np.inf, fit_intercept=False, max_iter=10000
    ).fit(Xb, y)

    print("DP:", np.round(theta_dp, 3), "sklearn:", np.round(sk.coef_[0], 3))
    assert np.max(np.abs(theta_dp - sk.coef_[0])) < 0.5


@pytest.mark.parametrize("l1_ratio", [0.5, 1.0])
def test_logistic_regression_l1_sparsity(l1_ratio):
    import opendp.prelude as dp

    np = pytest.importorskip("numpy")
    pytest.importorskip("scipy")
    from opendp.extras.sklearn.linear_model import LogisticRegression

    rng = np.random.default_rng(0)
    X = rng.standard_normal((200, 3))
    y = (X[:, 0] + 0.3 * rng.standard_normal(200) > 0).astype(float)
    data = dp.sklearn.SklearnFit(X, y)
    dom = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(num_columns=3, size=200, nan=False, T=float),
        dp.numpy.array1_domain(T=float, nan=False, size=200),
    )
    est = LogisticRegression(
        n_iters=2000,
        learning_rate=0.5,
        clip_norm=100.0,
        alpha=20.0,
        l1_ratio=l1_ratio,
    )
    theta = np.asarray(
        est.make(
            dom, dp.symmetric_distance(), dp.zero_concentrated_divergence(), 2, 1e8
        )(data)
    )

    coef = theta[:-1]
    assert abs(coef[0]) > abs(coef[1])
    assert abs(coef[0]) > abs(coef[2])
    assert abs(coef[1]) < 0.1  # check that noisy features are close to 0
    assert abs(coef[2]) < 0.1
