from fractions import Fraction

import opendp.prelude as dp
import pytest

np = pytest.importorskip("numpy")

from opendp.extras.sklearn.linear_model._make_private_sgd_classifier import (
    make_private_sgd_classifier,
    per_example_gradients,
)


@pytest.fixture
def model_type():
    pytest.importorskip("sklearn")
    from opendp.extras.sklearn.linear_model import SGDClassifier

    return SGDClassifier


@pytest.fixture
def expit():
    return pytest.importorskip("scipy.special").expit


def _supported(model_type, **kwargs):
    """Construct a classifier using the logistic loss for legacy test cases."""
    params = dict(loss="log_loss", classes=(0, 1))
    return model_type(**(params | kwargs))


def _domain(size=None, y_size=None, **features):
    return dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(**(dict(T=float, num_columns=2, size=size) | features)),
        dp.vector_domain(dp.atom_domain(T=float), size=y_size),
    )


def _make(model, domain, d_in=2, rho=1.0):
    return model.make_measurement(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(), d_in, rho
    )


@pytest.fixture
def data():
    rng = np.random.default_rng(0)
    X = rng.standard_normal((100, 2))
    y = (X[:, 0] + rng.standard_normal(100) > 0).astype(float)
    return dp.sklearn.SklearnFit(X, y)


@pytest.mark.parametrize("fit_intercept", [True, False])
def test_hinge_subgradients_and_row_stability(fit_intercept):
    X = np.array([[2, 0], [0.25, 1], [-2, 0], [1, 1], [0, 0], [1, 0]])
    y = np.array([1, 1, 0, 0, 1, 1])
    theta = np.array([1.0, 0.0, 0.0] if fit_intercept else [1.0, 0.0])
    actual = per_example_gradients(
        dp.sklearn.SklearnFit(X, y), theta, fit_intercept, loss="hinge"
    )
    expected = np.array(
        [
            [0, 0, 0],
            [-0.25, -1, -1],
            [0, 0, 0],
            [1, 1, 1],
            [0, 0, -1],
            [0, 0, 0],  # exactly at the hinge: choose zero subgradient
        ],
        dtype=float,
    )
    np.testing.assert_array_equal(
        actual, expected if fit_intercept else expected[:, :2]
    )
    modified = y.copy()
    modified[1] = 0
    changed = per_example_gradients(
        dp.sklearn.SklearnFit(X, modified), theta, fit_intercept, loss="hinge"
    )
    np.testing.assert_array_equal(changed[2:], actual[2:])
    assert not np.array_equal(changed[1], actual[1])


@pytest.mark.parametrize("fit_intercept", [True, False])
def test_gradients_and_row_stability(expit, fit_intercept):
    X = np.array([[1, np.nan], [3, 4], [5, 6]], dtype=np.float32)
    y = np.array([False, True, False])
    theta = np.array([0.2, -0.1, 0.3] if fit_intercept else [0.2, -0.1])
    gradients = per_example_gradients(dp.sklearn.SklearnFit(X, y), theta, fit_intercept)
    Xb = np.nan_to_num(X)
    if fit_intercept:
        Xb = np.column_stack([Xb, np.ones(3)])
    np.testing.assert_allclose(gradients, (expit(Xb @ theta) - y)[:, None] * Xb)
    changed_X, changed_y = X.copy(), y.copy()
    changed_X[0], changed_y[0] = [-1, -2], True
    changed = per_example_gradients(
        dp.sklearn.SklearnFit(changed_X, changed_y), theta, fit_intercept
    )
    assert not np.array_equal(changed[0], gradients[0])
    np.testing.assert_array_equal(changed[1:], gradients[1:])
    extended = dp.sklearn.SklearnFit(np.vstack([X, X[:1]]), np.concatenate([y, y[:1]]))
    np.testing.assert_allclose(
        per_example_gradients(extended, theta, fit_intercept)[:-1],
        gradients,
        rtol=1e-14,
        atol=1e-14,
    )


def test_gradients_with_wide_integer_targets():
    pytest.importorskip("scipy.special")
    # NumPy infers object dtype for Python integers beyond its native range.
    y = [2**100, -(2**100), 0]
    assert np.asarray(y).dtype.kind == "O"
    X = np.array([[1, 2], [-1, 0], [2, 4]], dtype=float)
    gradients = per_example_gradients(dp.sklearn.SklearnFit(X, y), np.zeros(3))
    assert gradients.dtype == np.dtype(float)
    assert np.isfinite(gradients).all()
    np.testing.assert_array_equal(
        gradients,
        np.array([[0, 0, 0], [0, 0, 0], [1, 2, 0.5]], dtype=float),
    )


@pytest.mark.parametrize("loss", ["hinge", "log_loss"])
@pytest.mark.parametrize("fit_intercept", [True, False])
def test_nonfinite_gradients_and_fitting(model_type, fit_intercept, loss):
    largest, smallest = np.finfo(float).max, np.finfo(float).smallest_subnormal
    X = np.array(
        [[np.inf, -np.inf], [largest, -largest], [smallest, -smallest], [1, 2], [1, 2]]
    )
    y = np.array([1, largest, -2, np.nan, np.inf])
    with np.errstate(all="raise"):
        gradients = per_example_gradients(
            dp.sklearn.SklearnFit(X, y),
            np.full(2 + int(fit_intercept), largest),
            fit_intercept,
            loss=loss,
        )
    assert np.isfinite(gradients).all()
    np.testing.assert_array_equal(gradients[3:], np.zeros((2, 2 + int(fit_intercept))))
    model = _supported(
        model_type,
        loss=loss,
        reference_size=11,
        max_iter=2,
        fit_intercept=fit_intercept,
    ).fit(
        X, y, privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(rho=1.0)
    )
    assert np.isfinite(model.coef_).all() and np.isfinite(model.intercept_).all()
    with np.errstate(all="raise"):
        assert np.isfinite(model.decision_function(X)).all()
        if loss == "log_loss":
            assert np.isfinite(model.predict_proba(X)).all()


@pytest.mark.parametrize("loss", ["log_loss", "hinge"])
@pytest.mark.parametrize("fit_intercept", [True, False])
@pytest.mark.parametrize(
    "x_size, y_size, reference_size, eta0, alpha, l1_ratio, clip_norm",
    [
        (3, None, None, 0.3, 0, 0, 10),
        (3, 3, 6, Fraction(3, 5), 2, 0, 10),
        (None, 3, None, 0.3, 0.25, 1, 10),
        (None, None, 6, 0.6, 0.125, 0.5, 0.5),
    ],
)
def test_updates_and_privacy_map(
    model_type,
    expit,
    x_size,
    y_size,
    reference_size,
    eta0,
    alpha,
    l1_ratio,
    clip_norm,
    fit_intercept,
    loss,
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
    model = _supported(
        model_type,
        loss=loss,
        max_iter=2,
        fit_intercept=fit_intercept,
        eta0=eta0,
        clip_norm=clip_norm,
        alpha=alpha,
        penalty="l1" if l1_ratio == 1 else "elasticnet" if l1_ratio else "l2",
        l1_ratio=l1_ratio,
        reference_size=reference_size,
    )
    d_in = 1 if x_size is None and y_size is None else 2
    measurement = _make(model, domain, d_in, rho=1e12)
    assert measurement.input_domain == domain
    assert measurement.map(d_in) <= 1e12
    normalizer = reference_size if reference_size is not None else 3
    expected = np.zeros(2 + int(fit_intercept))
    Xb = np.column_stack([X, np.ones(3)]) if fit_intercept else X
    for _ in range(2):
        if loss == "log_loss":
            residual = expit(Xb @ expected) - y
        else:
            signed_y = 2 * y - 1
            residual = np.where(signed_y * (Xb @ expected) < 1, -signed_y, 0.0)
        gradients = residual[:, None] * Xb
        gradients /= np.maximum(
            1, np.linalg.norm(gradients, axis=1, keepdims=True) / clip_norm
        )
        penalty = alpha * normalizer * (1 - l1_ratio) * expected
        if fit_intercept:
            penalty[-1] = 0
        # The constructor converts real-valued eta0 (including Fraction) to float.
        step_size = float(eta0) / normalizer
        expected -= step_size * (gradients.sum(axis=0) + penalty)
        expected[:2] = np.sign(expected[:2]) * np.maximum(
            np.abs(expected[:2]) - step_size * alpha * normalizer * l1_ratio, 0
        )
    actual = measurement(dp.sklearn.SklearnFit(X, y))
    np.testing.assert_allclose(actual, expected, atol=1e-4, rtol=0)
    if l1_ratio > 0 and loss == "log_loss":
        assert actual[0] > 0 and actual[1] == 0
        if fit_intercept:
            assert actual[-1] != 0  # The intercept is not soft-thresholded.


@pytest.mark.parametrize("fit_intercept", [True, False])
def test_unsized_empty_fit_has_noise(model_type, fit_intercept):
    from sklearn.base import clone

    X, y = np.empty((0, 2)), np.empty(0)
    model = _supported(
        model_type, reference_size=11, max_iter=2, fit_intercept=fit_intercept
    )
    kwargs = dict(
        privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(rho=1.0)
    )
    first = model.fit(X, y, **kwargs)
    second = clone(model).fit(X, y, **kwargs)
    assert np.isfinite(first.coef_).all()
    assert first.predict_proba(X).shape == (0, 2)
    # Empty private data still needs noise under add/remove adjacency.
    assert not np.array_equal(first.coef_, second.coef_)


def test_constructor_signature_and_cloning(model_type):
    import inspect
    from sklearn.base import clone

    # Retain meaningful sklearn-style options and full-batch optimizer defaults.
    defaults = dict(
        loss="hinge",
        classes=(0, 1),
        penalty="l2",
        alpha=0.0001,
        l1_ratio=0.15,
        fit_intercept=True,
        max_iter=100,
        eta0=0.1,
        clip_norm=1.0,
        reference_size=None,
    )
    parameters = inspect.signature(model_type).parameters
    assert list(parameters) == list(defaults)
    assert parameters["classes"].default is inspect.Parameter.empty
    with pytest.raises(TypeError, match="classes"):
        model_type()
    assert parameters["loss"].kind == inspect.Parameter.POSITIONAL_OR_KEYWORD
    assert all(
        p.kind == inspect.Parameter.KEYWORD_ONLY
        for name, p in parameters.items()
        if name != "loss"
    )
    assert model_type(classes=(0, 1)).get_params() == defaults
    assert model_type("log_loss", classes=(0, 1)).loss == "log_loss"
    model = model_type(classes=(0, 1), fit_intercept=False, max_iter=7)
    assert clone(model).get_params() == defaults | dict(fit_intercept=False, max_iter=7)
    assert model.set_params(max_iter=3, fit_intercept=True) is model
    assert model.max_iter == 3 and model.fit_intercept is True
    with pytest.raises(TypeError, match="positional"):
        model_type("log_loss", "l2", classes=(0, 1))
    with pytest.raises(TypeError, match="n_iters"):
        model_type(classes=(0, 1), n_iters=3)
    with pytest.raises(TypeError, match="learning_rate"):
        model_type(classes=(0, 1), learning_rate="constant")
    with pytest.raises(TypeError, match="tol"):
        model_type(classes=(0, 1), loss="log_loss", tol=None)


def test_default_hinge_loss(model_type):
    domain = _domain(size=3, y_size=3)
    assert _make(model_type(classes=(0, 1), max_iter=1), domain).map(2) <= 1.0
    log_loss = model_type(classes=(0, 1), loss="log_loss", max_iter=1)
    assert _make(log_loss, domain).map(2) <= 1.0


@pytest.mark.parametrize(
    "parameter, value",
    [
        ("shuffle", True),
        ("verbose", 1),
        ("epsilon", 0.1),
        ("n_jobs", 2),
        ("random_state", 0),
        ("power_t", 0.5),
        ("early_stopping", True),
        ("validation_fraction", 0.1),
        ("n_iter_no_change", 5),
        ("class_weight", "balanced"),
        ("warm_start", True),
        ("average", True),
    ],
)
def test_unsupported_constructor_options(model_type, parameter, value):
    with pytest.raises(TypeError, match=parameter):
        model_type(classes=(0, 1), **{parameter: value})


def test_low_level_rejects_unsupported_loss():
    # The low-level constructor must enforce its own loss contract.
    with pytest.raises(ValueError, match="loss"):
        make_private_sgd_classifier(
            _domain(3, 3),
            dp.symmetric_distance(),
            dp.zero_concentrated_divergence(),
            2,
            1.0,
            max_iter=1,
            eta0=0.1,
            clip_norm=1.0,
            loss="modified_huber",
        )


def test_regularization_overflow_and_no_penalty(model_type):
    domain = _domain(3, 3)
    with pytest.raises(ValueError, match="alpha times reference_size"):
        _make(_supported(model_type, alpha=1e308, max_iter=1), domain)

    # When no penalty is requested, alpha is immaterial, even if scaling
    # it by the public reference size would overflow.
    measurement = _make(
        _supported(model_type, penalty=None, alpha=1e308, max_iter=1), domain
    )
    assert measurement.map(2) <= 1.0


def test_irrelevant_l1_ratio_none(model_type):
    domain = _domain(size=3, y_size=3)
    # sklearn permits l1_ratio=None when not using elasticnet.
    _make(_supported(model_type, penalty="l2", l1_ratio=None, max_iter=1), domain)
    with pytest.raises(ValueError, match="l1_ratio"):
        _make(
            _supported(model_type, penalty="elasticnet", l1_ratio=None, max_iter=1),
            domain,
        )


@pytest.mark.parametrize("loss", ["log_loss", "hinge"])
@pytest.mark.parametrize("fit_intercept", [True, False])
@pytest.mark.parametrize("max_iter", [1, 3])
def test_exact_steps_and_released_coordinates(
    model_type, monkeypatch, fit_intercept, max_iter, loss
):
    import opendp.extras.sklearn.linear_model._make_private_sgd_classifier as impl

    gradients, sums = [], []
    original_gradients, original_sum = (
        impl.per_example_gradients,
        impl.make_private_np_sum,
    )

    def observe_gradients(data, theta, fit_intercept, loss, *, classes):
        result = original_gradients(
            data, theta, fit_intercept, loss=loss, classes=classes
        )
        gradients.append(result)
        return result

    def observe_sum(domain, *args):
        sums.append(domain.cast(dp.numpy.NPArray2Domain).num_columns)
        return original_sum(domain, *args)

    monkeypatch.setattr(impl, "per_example_gradients", observe_gradients)
    monkeypatch.setattr(impl, "make_private_np_sum", observe_sum)
    measurement = _make(
        _supported(
            model_type,
            fit_intercept=fit_intercept,
            max_iter=max_iter,
            alpha=0,
            loss=loss,
        ),
        _domain(3, 3),
    )
    # Zero features keep the coefficient signal at zero, but must not stop training.
    data = dp.sklearn.SklearnFit(np.zeros((3, 2)), np.zeros(3))
    released = measurement(data)
    dimension = 2 + int(fit_intercept)
    assert len(gradients) == max_iter
    assert all(g.shape == (3, dimension) for g in gradients)
    assert sums and set(sums) == {dimension}
    assert released.shape == (dimension,)


def test_unsized_requires_reference_size():
    with pytest.raises(ValueError, match="reference_size is required"):
        make_private_sgd_classifier(
            _domain(),
            dp.symmetric_distance(),
            dp.zero_concentrated_divergence(),
            1,
            1.0,
            max_iter=1,
            eta0=0.1,
            clip_norm=1.0,
        )


def test_input_domain_validation(model_type):
    with pytest.raises(ValueError, match="requires a target domain y"):
        _make(
            _supported(model_type),
            dp.sklearn.sklearn_fit_domain(
                dp.numpy.array2_domain(T=float, num_columns=2)
            ),
        )
    targets = dp.vector_domain(dp.atom_domain(T=float))
    options = dict(T=float, num_columns=2, size=3)
    cases: list[tuple[dict, dp.Domain, str]] = [
        ({"T": str}, targets, "numeric element types"),
        ({}, dp.vector_domain(dp.atom_domain(T=str)), "classes must match"),
        ({}, dp.numpy.array2_domain(T=float), "VectorDomain"),
        ({}, dp.vector_domain(dp.atom_domain(T=float), size=2), "matching row counts"),
        ({"num_columns": None}, targets, "positive num_columns"),
        ({"num_columns": 0}, targets, "positive num_columns"),
    ]
    for features, target, error in cases:
        domain = dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(**(options | features)), target
        )
        with pytest.raises(ValueError, match=error):
            _make(_supported(model_type), domain)
    with pytest.raises(ValueError, match="d_in must be even"):
        _make(_supported(model_type), _domain(3), d_in=1)


def test_nonatomic_target_validation(model_type, monkeypatch):
    # Synthesize a non-atomic element domain to exercise the defensive guard.
    domain = _domain(3)
    element_domain = dp.numpy.array2_domain(T=float)
    monkeypatch.setattr(
        dp.VectorDomain, "element_domain", property(lambda _: element_domain)
    )
    with pytest.raises(
        ValueError, match="y must have numeric, boolean or string elements"
    ):
        _make(_supported(model_type), domain)


@pytest.mark.parametrize(
    "parameter, values",
    [
        ("alpha", [-1, float("nan"), float("inf")]),
        ("l1_ratio", [-0.1, 1.1, float("nan"), float("inf")]),
        ("reference_size", [0, -1, 1.5, True, float("nan"), float("inf"), 10**1000]),
        ("max_iter", [0, 1.5, True, float("inf")]),
        ("eta0", [0, float("nan"), float("inf"), "0.1", 10**1000]),
        ("loss", ["modified_huber", None]),
        ("penalty", ["nuclear", 1]),
        ("clip_norm", [-1, float("nan"), float("inf"), None, 10**1000]),
    ],
)
def test_public_parameter_validation(model_type, parameter, values):
    domain = _domain(3)
    for value in values:
        with pytest.raises(ValueError, match=parameter):
            _make(_supported(model_type, **{parameter: value}), domain)


@pytest.mark.parametrize("fit_intercept", [True, False])
@pytest.mark.parametrize(
    "route, as_list", [("standalone", False), ("standalone", True), ("context", False)]
)
def test_fitting_routes(model_type, data, route, as_list, fit_intercept):
    from sklearn.base import is_classifier

    model = _supported(
        model_type,
        max_iter=3,
        fit_intercept=fit_intercept,
        reference_size=37 if route == "standalone" else None,
    )
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
    if not fit_intercept:
        np.testing.assert_array_equal(fitted.intercept_, np.zeros(1))
    if route == "standalone":
        assert fitted is model
    else:
        assert fitted is not model and not hasattr(model, "coef_")
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


def test_hinge_training_and_prediction(model_type):
    X = np.array([[-2.0], [-1.0], [1.0], [2.0]])
    y = np.array([0.0, 0.0, 1.0, 1.0])
    model = model_type(
        classes=(0, 1), max_iter=100, alpha=0, clip_norm=3, reference_size=4
    )
    assert not hasattr(model, "predict_proba")
    fitted = model.fit(
        X,
        y,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(rho=1e12),
    )
    scores = fitted.decision_function(X)
    assert scores.shape == (4,)
    np.testing.assert_array_equal(fitted.predict(X), y)
    np.testing.assert_array_equal(fitted.predict(X), (scores > 0).astype(int))
    with pytest.raises(AttributeError):
        fitted.predict_proba(X)


@pytest.mark.parametrize("fit_intercept", [True, False])
def test_convergence_matches_sklearn(model_type, data, fit_intercept):
    from sklearn.linear_model import LogisticRegression

    reference = LogisticRegression(
        C=1, max_iter=1000, tol=1e-8, fit_intercept=fit_intercept
    ).fit(data.X, data.y)
    model = _supported(
        model_type,
        max_iter=150,
        eta0=0.5,
        alpha=0.01,
        clip_norm=10,
        fit_intercept=fit_intercept,
    )
    theta = _make(model, _domain(100, 100), rho=1e12)(data)
    expected = (
        np.r_[reference.coef_[0], reference.intercept_]
        if fit_intercept
        else reference.coef_[0]
    )
    np.testing.assert_allclose(theta, expected, atol=0.03, rtol=0)


@pytest.mark.parametrize("loss", ["hinge", "log_loss"])
def test_public_classes(model_type, loss):
    X = np.ones((3, 2))
    y = ["positive", "negative", "unknown"]
    data = dp.sklearn.SklearnFit(X, y)
    classes = ("negative", "positive")
    gradients = per_example_gradients(data, np.zeros(3), loss=loss, classes=classes)
    magnitude = 1.0 if loss == "hinge" else 0.5
    np.testing.assert_array_equal(
        gradients, [[-magnitude] * 3, [magnitude] * 3, [0.0] * 3]
    )

    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=2),
        dp.vector_domain(dp.atom_domain(T=str)),
    )
    model = model_type(classes=classes[::-1], loss=loss, reference_size=3, max_iter=1)
    for invalid in (
        [],
        ("same", "same"),
        (0, np.nan),
        (0, "positive"),
        (0.1, 0.2),
        (np.uint64(2**63 + 1), np.int64(-1)),
    ):
        with pytest.raises(ValueError, match="classes"):
            _make(model_type(classes=invalid, reference_size=3), _domain(), d_in=1)
    model.fit(
        X,
        y,
        domain=domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(rho=1.0),
    )
    np.testing.assert_array_equal(model.classes_, classes)
    np.testing.assert_array_equal(
        model.predict(X), model.classes_[(model.decision_function(X) > 0).astype(int)]
    )
    if loss == "log_loss":
        np.testing.assert_array_equal(
            model.predict(X),
            model.classes_[(model.predict_proba(X)[:, 1] > 0.5).astype(int)],
        )
    predicted = model.rowwise_output_domain(
        domain.cast(dp.sklearn.SklearnFitDescriptor).X, method="predict"
    )
    assert predicted.member(model.predict(X)[:, None])
    assert model.score(X[:2], y[:2]) == 0.5
