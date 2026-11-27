"""Privacy accounting, numerical postprocessing, and sklearn API tests."""

import numpy as np
import pytest

import opendp.prelude as dp
from opendp.extras.sklearn.linear_model2._make_private_linear_regression import (
    _solve,
)


def _domain(n=None, d=2, y_size=None):
    return dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=d, size=n),
        dp.vector_domain(dp.atom_domain(T=float), size=y_size),
    )


def _model(**kwargs):
    pytest.importorskip("sklearn")
    from opendp.extras.sklearn.linear_model2 import LinearRegression

    return LinearRegression(**({"clip_norm": 2.0, "y_bound": 3.0} | kwargs))


def _measure(*, rho=True, model=None, domain=None, d_in=2, budget=2.0):
    measure = dp.zero_concentrated_divergence() if rho else dp.max_divergence()
    return (model or _model()).make_measurement(
        _domain() if domain is None else domain,
        dp.symmetric_distance(),
        measure,
        d_in,
        budget,
    )


@pytest.mark.parametrize("gaussian", [True, False])
def test_svec_reconstruction_and_flooring(gaussian):
    # svec([[3, 1], [1, 3]]) = [3, sqrt(2), 3].
    solve_kwargs = dict(
        dimension=2,
        gram_scale=1e-9,
        failure_probability=0.05,
        gaussian=gaussian,
    )
    theta = _solve(
        (np.array([3.0, np.sqrt(2.0), 3.0]), np.array([5.0, 10.0])),
        **solve_kwargs,
    )
    np.testing.assert_allclose(theta, [0.625, 3.125], rtol=1e-6)

    # Flooring preserves the strong eigenvalue and stabilizes the weak one.
    solve_kwargs["gram_scale"] = 1.0
    theta = _solve(
        (np.array([100.0, 0.0, 0.1]), np.array([100.0, 0.1])),
        **solve_kwargs,
    )
    np.testing.assert_allclose(theta[0], 1.0)
    assert 0.0 < theta[1] < 1.0

    theta = _solve((np.zeros(3), np.array([2.0, 3.0])), **solve_kwargs)
    assert theta.shape == (2,)
    assert np.isfinite(theta).all()


@pytest.mark.parametrize("gaussian", [True, False])
def test_public_count_postprocessing(gaussian):
    release = (np.array([100.0, 0.0, 500.0]), np.array([40.0, 50.0]))
    kwargs = dict(
        dimension=2,
        gram_scale=1e-9,
        failure_probability=0.05,
        gaussian=gaussian,
    )
    np.testing.assert_allclose(_solve(release, public_count=100, **kwargs), [0.4, 0.5])
    np.testing.assert_allclose(_solve(release, **kwargs), [0.4, 0.1])


@pytest.mark.parametrize("gaussian", [True, False])
def test_spectral_floor_responds_to_confidence(gaussian):
    # Higher confidence must give stronger flooring, for both noise laws.
    def solve(alpha):
        return _solve(
            (np.zeros(6), np.ones(3)),
            dimension=3,
            gram_scale=1.0,
            failure_probability=alpha,
            gaussian=gaussian,
        )

    conservative, permissive = solve(0.01), solve(0.2)
    assert np.isfinite(conservative).all()
    assert np.all((0 < conservative) & (conservative < permissive))


@pytest.mark.parametrize("rho", [True, False])
@pytest.mark.parametrize("size_source", ["variable", "X", "y", "single"])
def test_privacy_accounting(rho, size_source):
    dp.enable_features("contrib", "idealized-numerics")
    fixed = size_source != "variable"
    single = size_source == "single"
    domain = _domain(
        n=5 if size_source in ("X", "single") else None,
        d=1 if single else 2,
        y_size=5 if size_source in ("y", "single") else None,
    )
    measurement = _measure(
        rho=rho, model=_model(fit_intercept=not single), domain=domain
    )
    expected = dp.zero_concentrated_divergence() if rho else dp.max_divergence()
    assert measurement.output_measure == expected
    assert 0 < measurement.map(2) <= 2.0
    if fixed:
        # Fixed-size inputs admit substitutions (two symmetric edits), not one edit.
        assert measurement.map(1) == 0
    else:
        assert measurement.map(1) > 0


@pytest.mark.parametrize("rho", [True, False])
@pytest.mark.parametrize("fit_intercept", [True, False])
def test_fit_recovers_well_conditioned_linear_model(rho, fit_intercept):
    dp.enable_features("contrib", "idealized-numerics")
    # Balanced design: independent features, with or without an intercept.
    X = np.tile([[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]], (256, 1))
    offset = 0.5 if fit_intercept else 0.0
    y = X @ np.array([1.25, -0.75]) + offset
    model = _model(fit_intercept=fit_intercept)
    loss = dp.loss_of(rho=8.0) if rho else dp.loss_of(epsilon=8.0)
    assert (
        model.fit(
            X,
            y,
            privacy_unit=dp.unit_of(contributions=2),
            privacy_loss=loss,
        )
        is model
    )
    np.testing.assert_allclose(model.coef_, [1.25, -0.75], atol=0.15, rtol=0)
    assert model.intercept_ == pytest.approx(offset, abs=0.15)
    np.testing.assert_allclose(model.predict(X[:4]), y[:4], atol=0.3, rtol=0)


def test_reject_invalid_privacy_domains_and_bounds():
    dp.enable_features("contrib", "idealized-numerics")
    with pytest.raises(ValueError, match="zero_concentrated_divergence"):
        _model().make_measurement(
            _domain(),
            dp.symmetric_distance(),
            dp.approximate(dp.max_divergence()),
            2,
            (2.0, 1e-6),
        )
    with pytest.raises(ValueError, match="d_out"):
        _measure(budget=0)
    with pytest.raises(ValueError, match="clip_norm"):
        _measure(model=_model(clip_norm=0))
    with pytest.raises(ValueError, match="Sized data requires even"):
        _measure(domain=_domain(n=3, y_size=3), d_in=1)
    with pytest.raises(ValueError, match="matching public row counts"):
        _measure(domain=_domain(n=3, y_size=4))
    with pytest.raises(ValueError, match="failure_probability"):
        _measure(model=_model(failure_probability=1.0))
    with pytest.raises(ValueError, match="targets"):
        _measure(
            domain=dp.sklearn.sklearn_fit_domain(
                dp.numpy.array2_domain(T=float, num_columns=2)
            )
        )
    with pytest.raises(ValueError, match="numeric"):
        _measure(
            domain=dp.sklearn.sklearn_fit_domain(
                dp.numpy.array2_domain(T=float, num_columns=2),
                dp.vector_domain(dp.atom_domain(T="String")),
            )
        )


def test_validate_remaining_public_contract_errors():

    with pytest.raises(ValueError, match="symmetric_distance"):
        _model().make_measurement(
            _domain(),
            dp.l1_distance(T=float),
            dp.max_divergence(),
            2,
            2.0,
        )
    with pytest.raises(ValueError, match="positive number of row edits"):
        _measure(d_in=0)
    with pytest.raises(ValueError, match="fit_intercept must be boolean"):
        _measure(model=_model(fit_intercept=1))
    with pytest.raises(ValueError, match="X must contain numeric"):
        _measure(
            domain=dp.sklearn.sklearn_fit_domain(
                dp.numpy.array2_domain(T="String", num_columns=2),
                dp.vector_domain(dp.atom_domain(T=float)),
            )
        )
    with pytest.raises(ValueError, match="one-dimensional numeric vector"):
        _measure(
            domain=dp.sklearn.sklearn_fit_domain(
                dp.numpy.array2_domain(T=float, num_columns=2),
                dp.atom_domain(T=float),
            )
        )
    with pytest.raises(ValueError, match="positive, public number of features"):
        _measure(domain=_domain(d=None))


def test_reject_invalid_calibration(monkeypatch):
    monkeypatch.setattr(dp, "binary_search_param", lambda *args, **kwargs: 1e-6)
    with pytest.raises(ValueError, match="Privacy calibration exceeds"):
        _measure()


def test_regression_clipped_statistics(monkeypatch):
    from opendp.extras.sklearn.linear_model2 import (
        _make_private_linear_regression as private_lr,
    )

    captured = []
    original = private_lr.make_np_joint_moments

    def capture(*args, **kwargs):
        result = original(*args, **kwargs)
        captured.extend(result)
        return result

    monkeypatch.setattr(private_lr, "make_np_joint_moments", capture)
    _measure(domain=_domain(n=2, y_size=2))
    joint, gram_unit, cross_unit = captured
    assert joint.map(2) == pytest.approx(13 / 9)
    data = dp.sklearn.SklearnFit(
        np.array([[3.0, 4.0], [0.0, -5.0]]), np.array([10.0, -9.0])
    )
    release = np.asarray(joint(data))
    np.testing.assert_allclose(release[6:] * cross_unit, [3.6, 10.8, 0.0], atol=1e-12)
    np.testing.assert_allclose(
        release[:6] * gram_unit,
        [1.44, 1.92 * np.sqrt(2), 1.2 * np.sqrt(2), 6.56, -0.4 * np.sqrt(2), 2.0],
        atol=1e-12,
    )


def test_prediction_contract_and_refitting():
    pytest.importorskip("sklearn")
    from sklearn.exceptions import NotFittedError

    model = _model()
    with pytest.raises(NotFittedError):
        model.predict([[0.0, 0.0]])

    # A fitted regression has sklearn's single-target attribute shapes.
    model._ingest_release(np.array([1.0, -2.0, 0.5]))
    np.testing.assert_array_equal(model.coef_, [1.0, -2.0])
    assert model.intercept_ == 0.5
    assert model.n_features_in_ == 2
    np.testing.assert_allclose(
        model.predict([[1.0, 2.0], [0.0, -1.0]]),
        [-2.5, 2.5],
    )
    np.testing.assert_allclose(
        model.predict([[np.nan, np.inf]]),
        [0.5],
    )
    with pytest.raises(ValueError, match="features"):
        model.predict([[0.0, 0.0, 0.0]])
    output_domain = model.rowwise_output_domain(
        dp.numpy.array2_domain(num_columns=2, T=float, nan=True),
        method="predict",
    )
    assert output_domain.member(model.predict([[np.nan, np.inf]])[:, None])
    with pytest.raises(ValueError, match="Method"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(num_columns=2, T=float),
            method="score",
        )
    with pytest.raises(ValueError, match="numeric"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(num_columns=2, T="String"),
            method="predict",
        )
    with pytest.raises(ValueError, match="fitted column count"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(num_columns=3, T=float),
            method="predict",
        )

    # Re-fitting resets model width and intercept, as sklearn expects.
    model.set_params(fit_intercept=False)
    model._ingest_release(np.array([3.0]))
    np.testing.assert_array_equal(model.coef_, [3.0])
    assert model.intercept_ == 0.0
    assert model.n_features_in_ == 1
    assert model.predict([[2.0]]).shape == (1,)


@pytest.mark.parametrize("route", ["standalone", "context"])
@pytest.mark.parametrize("rho", [True, False])
def test_public_regression_fitting_and_inference_without_hbc(route, rho):
    pytest.importorskip("sklearn")
    from opendp.extras.sklearn._rowwise import _make_rowwise_transform

    X = np.array([[1.0, 0.0], [0.0, 2.0]])
    y = np.array([0.0, 1.0])
    domain = _domain(n=2, y_size=2)
    data = dp.sklearn.SklearnFit(X, y)
    privacy_unit = dp.unit_of(contributions=2)
    privacy_loss = dp.loss_of(rho=1.0) if rho else dp.loss_of(epsilon=1.0)
    dp.disable_features("honest-but-curious")
    try:
        model = _model()
        if route == "standalone":
            fitted = model.fit(
                X,
                y,
                domain=domain,
                privacy_unit=privacy_unit,
                privacy_loss=privacy_loss,
            )
        else:
            context = dp.Context.compositor(
                data,
                domain=domain,
                split_evenly_over=1,
                privacy_unit=privacy_unit,
                privacy_loss=privacy_loss,
            )
            fitted = context.query().sklearn(model).release()
            assert not hasattr(model, "coef_")
        trans = _make_rowwise_transform(
            domain, dp.symmetric_distance(), fitted, method="predict"
        )
        result = trans(data)
        assert trans.map(1) == 1
        assert trans.output_domain.member(result)
        np.testing.assert_allclose(result.X[:, 0], fitted.predict(X))
        assert result.y is y
    finally:
        dp.enable_features("honest-but-curious")
