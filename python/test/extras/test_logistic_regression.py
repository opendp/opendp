from fractions import Fraction

import opendp.prelude as dp
import pytest

np = pytest.importorskip("numpy")

from opendp.extras.sklearn.linear_model._make_private_logistic_regression import (
    make_private_logistic_regression,
    per_example_gradients,
)


@pytest.fixture
def model_type():
    pytest.importorskip("sklearn")
    from opendp.extras.sklearn.linear_model import LogisticRegression

    return LogisticRegression


@pytest.fixture
def expit():
    return pytest.importorskip("scipy.special").expit


def _domain(size=None, y_size=None, **features):
    return dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(**(dict(T=float, num_columns=2, size=size) | features)),
        dp.vector_domain(dp.atom_domain(T=float), size=y_size),
    )


def _make(model, domain, d_in=2, rho=1.0):
    return model.make(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(), d_in, rho
    )


@pytest.fixture
def data():
    rng = np.random.default_rng(0)
    X = rng.standard_normal((100, 2))
    y = (X[:, 0] + rng.standard_normal(100) > 0).astype(float)
    return dp.sklearn.SklearnFit(X, y)


def test_gradients_and_row_stability(expit):
    X = np.array([[1, np.nan], [3, 4], [5, 6]], dtype=np.float32)
    y = np.array([False, True, False])
    theta = np.array([0.2, -0.1, 0.3])
    gradients = per_example_gradients(dp.sklearn.SklearnFit(X, y), theta)
    Xb = np.column_stack([np.nan_to_num(X), np.ones(3)])
    np.testing.assert_allclose(gradients, (expit(Xb @ theta) - y)[:, None] * Xb)
    changed_X, changed_y = X.copy(), y.copy()
    changed_X[0], changed_y[0] = [-1, -2], True
    changed = per_example_gradients(dp.sklearn.SklearnFit(changed_X, changed_y), theta)
    assert not np.array_equal(changed[0], gradients[0])
    np.testing.assert_array_equal(changed[1:], gradients[1:])
    extended = dp.sklearn.SklearnFit(np.vstack([X, X[:1]]), np.concatenate([y, y[:1]]))
    np.testing.assert_allclose(
        per_example_gradients(extended, theta)[:-1], gradients, rtol=1e-14, atol=1e-14
    )


def test_gradients_with_wide_integer_targets():
    # NumPy infers object dtype for Python integers beyond its native range.
    y = [2**100, -(2**100), 0]
    assert np.asarray(y).dtype.kind == "O"
    X = np.array([[1, 2], [-1, 0], [2, 4]], dtype=float)
    gradients = per_example_gradients(dp.sklearn.SklearnFit(X, y), np.zeros(3))
    assert gradients.dtype == np.dtype(float)
    assert np.isfinite(gradients).all()
    np.testing.assert_array_equal(
        gradients,
        np.array(
            [[-(2**100), -(2**101), -(2**100)], [-(2**100), 0, 2**100], [1, 2, 0.5]],
            dtype=float,
        ),
    )


def test_nonfinite_gradients_and_fitting(model_type):
    largest, smallest = np.finfo(float).max, np.finfo(float).smallest_subnormal
    X = np.array(
        [[np.inf, -np.inf], [largest, -largest], [smallest, -smallest], [1, 2], [1, 2]]
    )
    y = np.array([1, largest, -2, np.nan, np.inf])
    with np.errstate(all="raise"):
        gradients = per_example_gradients(
            dp.sklearn.SklearnFit(X, y), np.full(3, largest)
        )
    assert np.isfinite(gradients).all()
    np.testing.assert_array_equal(gradients[3:], np.zeros((2, 3)))
    model = model_type(reference_size=11, n_iters=2).fit(
        X, y, privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(rho=1.0)
    )
    assert np.isfinite(model.coef_).all() and np.isfinite(model.intercept_).all()
    with np.errstate(all="raise"):
        assert np.isfinite(model.predict_proba(X)).all()


@pytest.mark.parametrize(
    "x_size, y_size, reference_size, learning_rate, alpha, l1_ratio, clip_norm",
    [
        (3, None, None, 0.3, 0, 0, 10),
        (3, 3, 6, Fraction(3, 5), 2, 0, 10),
        (None, 3, None, 0.3, 0.75, 1, 10),
        (None, None, 6, 0.6, 0.75, 0.5, 0.5),
    ],
)
def test_updates_and_privacy_map(
    model_type,
    expit,
    x_size,
    y_size,
    reference_size,
    learning_rate,
    alpha,
    l1_ratio,
    clip_norm,
):
    X, y = (
        np.array([[1, 0], [0, 2], [1, 1]], dtype=float),
        np.array([1, 0, 1], dtype=float),
    )
    domain = _domain(
        x_size,
        y_size,
        norm=3,
        p=2,
        cardinalities=[2, 3],
    )
    model = model_type(
        n_iters=2,
        learning_rate=learning_rate,
        clip_norm=clip_norm,
        alpha=alpha,
        l1_ratio=l1_ratio,
        reference_size=reference_size,
    )
    d_in = 1 if x_size is None and y_size is None else 2
    measurement = _make(model, domain, d_in, rho=1e12)
    assert measurement.input_domain == domain
    assert measurement.map(d_in) <= 1e12
    expected = np.zeros(3)
    Xb = np.column_stack([X, np.ones(3)])
    for _ in range(2):
        gradients = (expit(Xb @ expected) - y)[:, None] * Xb
        gradients /= np.maximum(
            1, np.linalg.norm(gradients, axis=1, keepdims=True) / clip_norm
        )
        penalty = alpha * (1 - l1_ratio) * expected
        penalty[-1] = 0
        expected -= 0.1 * (gradients.sum(axis=0) + penalty)
        expected[:-1] = np.sign(expected[:-1]) * np.maximum(
            np.abs(expected[:-1]) - 0.1 * alpha * l1_ratio, 0
        )
    actual = measurement(dp.sklearn.SklearnFit(X, y))
    np.testing.assert_allclose(actual, expected, atol=1e-4, rtol=0)
    if l1_ratio > 0:
        assert actual[0] > 0 and actual[1] == 0
        assert actual[-1] != 0  # The intercept is not soft-thresholded.


def test_unsized_empty_fit_has_noise(model_type):
    from sklearn.base import clone

    X, y = np.empty((0, 2)), np.empty(0)
    model = model_type(reference_size=11, n_iters=2)
    kwargs = dict(
        privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(rho=1.0)
    )
    first = model.fit(X, y, **kwargs)
    second = clone(model).fit(X, y, **kwargs)
    assert np.isfinite(first.coef_).all()
    assert first.predict_proba(X).shape == (0, 2)
    # Empty private data still needs noise under add/remove adjacency.
    assert not np.array_equal(first.coef_, second.coef_)


def test_unsized_requires_reference_size():
    with pytest.raises(ValueError, match="reference_size is required"):
        make_private_logistic_regression(
            _domain(),
            dp.symmetric_distance(),
            dp.zero_concentrated_divergence(),
            1,
            1.0,
            n_iters=1,
            learning_rate=0.1,
            clip_norm=1.0,
        )


def test_input_domain_validation(model_type):
    with pytest.raises(ValueError, match="requires a target domain y"):
        _make(
            model_type(),
            dp.sklearn.sklearn_fit_domain(
                dp.numpy.array2_domain(T=float, num_columns=2)
            ),
        )
    targets = dp.vector_domain(dp.atom_domain(T=float))
    options = dict(T=float, num_columns=2, size=3)
    cases: list[tuple[dict, dp.Domain, str]] = [
        ({"T": str}, targets, "numeric element types"),
        ({}, dp.vector_domain(dp.atom_domain(T=str)), "numeric element types"),
        ({}, dp.numpy.array2_domain(T=float), "VectorDomain"),
        ({}, dp.vector_domain(dp.atom_domain(T=float), size=2), "matching row counts"),
        ({"num_columns": None}, targets, "specify num_columns"),
    ]
    for features, target, error in cases:
        domain = dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(**(options | features)), target
        )
        with pytest.raises(ValueError, match=error):
            _make(model_type(), domain)
    with pytest.raises(ValueError, match="d_in must be even"):
        _make(model_type(), _domain(3), d_in=1)


def test_nonatomic_target_validation(model_type, monkeypatch):
    # Synthesize a non-atomic element domain to exercise the defensive guard.
    domain = _domain(3)
    element_domain = dp.numpy.array2_domain(T=float)
    monkeypatch.setattr(
        dp.VectorDomain, "element_domain", property(lambda _: element_domain)
    )
    with pytest.raises(ValueError, match="^y must have numeric or boolean elements$"):
        _make(model_type(), domain)


@pytest.mark.parametrize(
    "parameter, values",
    [
        ("alpha", [-1, float("nan"), float("inf")]),
        ("l1_ratio", [-0.1, 1.1, float("nan"), float("inf")]),
        ("reference_size", [0, -1, 1.5, True, float("nan"), float("inf"), 10**1000]),
        ("n_iters", [0, 1.5, True, float("inf")]),
        ("learning_rate", [0, float("nan"), float("inf"), "0.1", 10**1000]),
        ("clip_norm", [-1, float("nan"), float("inf"), None, 10**1000]),
    ],
)
def test_public_parameter_validation(model_type, parameter, values):
    domain = _domain(3)
    for value in values:
        with pytest.raises(ValueError, match=parameter):
            _make(model_type(**{parameter: value}), domain)


@pytest.mark.parametrize(
    "route, as_list", [("standalone", False), ("standalone", True), ("context", False)]
)
def test_fitting_routes(model_type, data, route, as_list):
    from sklearn.base import is_classifier

    model = model_type(n_iters=3, reference_size=37 if route == "standalone" else None)
    if route == "standalone":
        fitted = model.fit(
            data.X,
            data.y.astype(int).tolist() if as_list else data.y.astype(bool),
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(rho=1.0),
        )
    else:
        context = dp.Context.compositor(
            data=data,
            domain=_domain(100, 100),
            privacy_unit=dp.unit_of(contributions=2),
            privacy_loss=dp.loss_of(rho=1.0),
            split_evenly_over=1,
        )
        fitted = context.query().sklearn(model).release()
    assert is_classifier(fitted)
    assert fitted.coef_.shape == (1, 2) and fitted.intercept_.shape == (1,)
    assert fitted.n_features_in_ == 2 and list(fitted.classes_) == [0, 1]
    proba, preds = fitted.predict_proba(data.X), fitted.predict(data.X)
    np.testing.assert_allclose(proba.sum(axis=1), 1)
    assert set(np.unique(preds)).issubset({0, 1})
    assert fitted.score(data.X, data.y) == np.mean(preds == data.y)
    missing_X, imputed_X = data.X.copy(), data.X.copy()
    missing_X[0], imputed_X[0] = [np.nan, np.inf], 0
    np.testing.assert_array_equal(
        fitted.predict_proba(missing_X), fitted.predict_proba(imputed_X)
    )
    np.testing.assert_array_equal(fitted.predict(missing_X), fitted.predict(imputed_X))


def test_convergence_matches_sklearn(model_type, data):
    from sklearn.linear_model import LogisticRegression

    reference = LogisticRegression(C=1, max_iter=1000, tol=1e-8).fit(data.X, data.y)
    model = model_type(n_iters=150, learning_rate=0.5, clip_norm=10)
    theta = _make(model, _domain(100, 100), rho=1e12)(data)
    np.testing.assert_allclose(
        theta, np.r_[reference.coef_[0], reference.intercept_], atol=0.03, rtol=0
    )
