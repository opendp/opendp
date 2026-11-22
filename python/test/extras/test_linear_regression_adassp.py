"""Tests for native zCDP and pure-DP regression."""

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
    from opendp.extras.sklearn.linear_model2 import LinearRegression

    return LinearRegression(**({"clip_norm": 2.0, "y_bound": 3.0} | kwargs))


def _measure(model=None, *, rho=True, budget=2.0, d_in=2, domain=None):
    measure = dp.zero_concentrated_divergence() if rho else dp.max_divergence()
    return (model or _model()).make_measurement(
        _domain() if domain is None else domain,
        dp.symmetric_distance(),
        measure,
        d_in,
        budget,
    )

@pytest.mark.parametrize("gaussian", [True, False])
def test_numerical_postprocessing_without_stabilization(gaussian):
    theta = _solve(
        (np.array([3.0, 0.0, 3.0]), np.array([6.0, 9.0])),
        dimension=2,
        gram_scale=1e-9,
        failure_probability=0.05,
        gaussian=gaussian,
    )
    np.testing.assert_allclose(theta, [2.0, 3.0], rtol=1e-6)

@pytest.mark.parametrize("gaussian", [True, False])
def test_numerical_postprocessing_stabilizes_singular_gram(gaussian):
    theta = _solve(
        (np.zeros(3), np.array([2.0, 3.0])),
        dimension=2,
        gram_scale=1.0,
        failure_probability=0.05,
        gaussian=gaussian,
    )
    assert np.isfinite(theta).all()
    assert theta.shape == (2,)

@pytest.mark.parametrize("rho", [True, False])
@pytest.mark.parametrize("fit_intercept", [True, False])
def test_accounting_stays_in_native_measure(rho, fit_intercept):
    dp.enable_features("contrib", "idealized-numerics")
    budget = 2.0
    measurement = _measure(
        model=_model(fit_intercept=fit_intercept), rho=rho, budget=budget
    )
    expected = dp.zero_concentrated_divergence() if rho else dp.max_divergence()
    assert measurement.output_measure == expected
    assert measurement.check(2, budget)
    assert measurement.map(2) <= budget

@pytest.mark.parametrize("rho", [True, False])
@pytest.mark.parametrize("fit_intercept", [True, False])
def test_sklearn_regressor_fits_and_predicts(rho, fit_intercept):
    dp.enable_features("contrib", "idealized-numerics")
    pytest.importorskip("sklearn")
    from sklearn.base import clone, is_regressor

    X = np.stack([np.linspace(-1, 1, 100), np.linspace(1, 0, 100)], axis=1)
    y = X[:, 0] - 2 * X[:, 1] + 0.5
    model = _model(fit_intercept=fit_intercept)
    assert is_regressor(model)
    clone(model)
    privacy_loss = dp.loss_of(rho=8.0) if rho else dp.loss_of(epsilon=8.0)
    assert model.fit(
        X,
        y,
        privacy_unit=dp.unit_of(contributions=2),
        privacy_loss=privacy_loss,
    ) is model
    assert model.coef_.shape == (2,)
    assert np.isfinite(model.coef_).all()
    assert np.isfinite(model.intercept_)
    if not fit_intercept:
        assert model.intercept_ == 0.0
    assert model.predict(X[:3]).shape == (3,)
    assert model.predict(X[:0]).shape == (0,)

    output = model.rowwise_output_domain(
        dp.numpy.array2_domain(T=float, num_columns=2), method="predict"
    )
    assert output.descriptor.num_columns == 1
    with pytest.raises(ValueError, match="Method"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(T=float, num_columns=2), method="score"
        )


def test_unsupported_privacy_measures_and_bounds():
    dp.enable_features("contrib", "idealized-numerics")
    model = _model()
    with pytest.raises(ValueError, match="zero_concentrated_divergence"):
        model.make_measurement(
            _domain(),
            dp.symmetric_distance(),
            dp.approximate(dp.max_divergence()),
            2,
            (2.0, 1e-6),
        )
    with pytest.raises(ValueError, match="d_out"):
        _measure(budget=0.0)
    with pytest.raises(ValueError, match="clip_norm"):
        _measure(model=_model(clip_norm=0.0))
    with pytest.raises(ValueError, match="Sized data requires even"):
        _measure(d_in=1, domain=_domain(n=3, y_size=3))


def test_reject_unknown_target_or_feature_type():
    dp.enable_features("contrib", "idealized-numerics")
    with pytest.raises(ValueError, match="targets"):
        _measure(domain=dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(T=float, num_columns=2)
        ))
    with pytest.raises(ValueError, match="numeric"):
        _measure(domain=dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(T=float, num_columns=2),
            dp.vector_domain(dp.atom_domain(T="String")),
        ))

@pytest.mark.parametrize("gaussian", [True, False])
def test_weighted_upper_triangle_reconstruction(gaussian):
    # svec([[3, 1], [1, 3]]) = [3, sqrt(2), 3].
    theta = _solve(
        (np.array([3.0, np.sqrt(2.0), 3.0]), np.array([5.0, 10.0])),
        dimension=2,
        gram_scale=1e-9,
        failure_probability=0.05,
        gaussian=gaussian,
    )
    np.testing.assert_allclose(theta, [0.625, 3.125], rtol=1e-6)

@pytest.mark.parametrize("gaussian", [True, False])
def test_regularization_uses_released_gram_only(gaussian):
    # Flooring leaves a strong eigenvalue untouched, and places the weak
    # direction at precisely the public spectral margin.
    d, alpha, scale = 2, 0.05, 1.0
    if gaussian:
        bound = scale * min(
            np.sqrt(2 * d) + np.sqrt(-2 * np.log(alpha)),
            np.sqrt((d + 1) * np.log(d / alpha)),
        )
    else:
        t = np.log(d / alpha)
        u = np.sqrt(2 * t / (d + 2 * t))
        bound = scale * min(
            (t - np.log1p(-u * u) - (d - 1) * np.log1p(-u * u / 2)) / u,
            d * np.log((d * (d + 1) // 2) / alpha),
        )
    theta = _solve(
        (np.array([10.0, 0.0, 0.0]), np.array([10.0, 10.0])),
        dimension=d,
        gram_scale=scale,
        failure_probability=alpha,
        gaussian=gaussian,
    )
    np.testing.assert_allclose(theta, [1.0, 10.0 / bound])

@pytest.mark.parametrize("gaussian", [True, False])
def test_regularization_zero_for_well_conditioned_release(gaussian):
    theta = _solve(
        (np.array([100.0, 0.0, 100.0]), np.array([40.0, 30.0])),
        dimension=2,
        gram_scale=1.0,
        failure_probability=0.05,
        gaussian=gaussian,
    )
    np.testing.assert_allclose(theta, [0.4, 0.3], atol=1e-12)

@pytest.mark.parametrize("gaussian", [True, False])
def test_flooring_does_not_shrink_strong_directions(gaussian):
    # Only the weak eigenvalue is floored. A uniform ridge shift would
    # unnecessarily shrink the first coefficient.
    theta = _solve(
        (np.array([100.0, 0.0, 0.1]), np.array([100.0, 0.1])),
        dimension=2,
        gram_scale=1.0,
        failure_probability=0.05,
        gaussian=gaussian,
    )
    np.testing.assert_allclose(theta[0], 1.0)
    assert 0.0 < theta[1] < 1.0

@pytest.mark.parametrize("gaussian", [True, False])
def test_public_count_replaces_noisy_intercept_gram_entry(gaussian):
    release = (np.array([100.0, 0.0, 500.0]), np.array([40.0, 50.0]))
    params = dict(dimension=2, gram_scale=1e-9,
                  failure_probability=0.05, gaussian=gaussian)
    corrected = _solve(release, public_count=100, **params)
    uncorrected = _solve(release, **params)
    np.testing.assert_allclose(corrected, [0.4, 0.5])
    np.testing.assert_allclose(uncorrected, [0.4, 0.1])

@pytest.mark.parametrize("rho", [True, False])
def test_public_count_accounting(rho):
    dp.enable_features("contrib", "idealized-numerics")
    measurement = _measure(rho=rho, domain=_domain(n=100, y_size=100))
    assert measurement.check(2, 2.0)


def test_laplace_tail_uses_exact_matrix_mgf():
    # With zero noisy Gram, eigenvalue flooring divides by the public
    # noise margin. At d=10 the matrix bound beats the old row-sum bound.
    d, alpha, b = 10, 0.05, 2.0
    t = np.log(d / alpha)
    u = np.sqrt(2 * t / (d + 2 * t))
    expected = b * min(
        (t - np.log1p(-u*u) - (d - 1)*np.log1p(-u*u/2)) / u,
        d * np.log((d * (d + 1) // 2) / alpha),
    )
    out = _solve(
        (np.zeros(d * (d + 1) // 2), np.ones(d)),
        dimension=d, gram_scale=b, failure_probability=alpha,
        gaussian=False,
    )
    np.testing.assert_allclose(out, np.full(d, 1 / expected))

def test_gaussian_small_dim_uses_matrix_mgf_bound():
    d, alpha, scale = 2, 0.05, 1.0
    bound = scale * np.sqrt((d + 1) * np.log(d / alpha))
    theta = _solve(
        (np.zeros(3), np.ones(2)),
        dimension=d, gram_scale=scale,
        failure_probability=alpha, gaussian=True,
    )
    np.testing.assert_allclose(theta, np.full(d, 1 / bound))

def test_joint_substitution_sensitivity_is_sharp():
    # Exact equality at an angle with cosine 1/4.
    x = np.array([1.0, 0.0])
    z = np.array([0.25, np.sqrt(15.0) / 4.0])
    delta_gram = np.outer(x, x) - np.outer(z, z)
    delta_cross = x + z  # label replacement 1 -> -1
    squared = (
        np.linalg.norm(delta_gram, ord="fro")**2 / 2
        + np.linalg.norm(delta_cross)**2 / 4
    )
    np.testing.assert_allclose(squared, (5.0 / 4.0)**2)


def test_joint_substitution_sensitivity_random_rows():
    rng = np.random.default_rng(51)
    for _ in range(300):
        x, z = rng.normal(size=(2, 4))
        x /= max(np.linalg.norm(x), 1.0)
        z /= max(np.linalg.norm(z), 1.0)
        y, y_prime = rng.uniform(-1.0, 1.0, size=2)
        gram_diff = np.outer(x, x) - np.outer(z, z)
        cross_diff = x * y - z * y_prime
        squared = (
            np.linalg.norm(gram_diff, ord="fro")**2 / 2
            + np.linalg.norm(cross_diff)**2 / 4
        )
        assert squared <= 25.0 / 16.0 + 1e-12


@pytest.mark.parametrize(
    "gaussian, expected_margin",
    [
        (True, 1.6448536269514726),
        (False, np.log(10.0)),
    ],
)
def test_scalar_gram_uses_exact_one_sided_noise_quantile(
    gaussian, expected_margin
):
    out = _solve(
        (np.zeros(1), np.ones(1)),
        dimension=1,
        gram_scale=1.0,
        failure_probability=0.05,
        gaussian=gaussian,
    )
    np.testing.assert_allclose(out, [1.0 / expected_margin])


@pytest.mark.parametrize("gaussian", [True, False])
def test_scalar_quantile_fallback_for_large_failure_probability(gaussian):
    out = _solve(
        (np.zeros(1), np.ones(1)),
        dimension=1,
        gram_scale=1.0,
        failure_probability=0.7,
        gaussian=gaussian,
    )
    assert np.isfinite(out).all()


@pytest.mark.parametrize("rho", [True, False])
def test_scalar_private_regression_fixed_size_accounting(rho):
    dp.enable_features("contrib", "idealized-numerics")
    measurement = _measure(
        model=_model(fit_intercept=False),
        rho=rho,
        domain=_domain(n=5, d=1, y_size=5),
    )
    assert measurement.check(2, 2.0)


def test_joint_laplace_substitution_sensitivity():
    # Each normalized L1 block is bounded by its L2 sensitivity.
    # The joint L2 bound is 5/4, giving an L1 bound of 5 sqrt(2)/4.
    rng = np.random.default_rng(121)
    for dimension in (2, 5, 10):
        num_entries = dimension * (dimension + 1) // 2
        upper = np.triu_indices(dimension)
        offdiag = upper[0] != upper[1]
        for _ in range(400):
            x, z = rng.normal(size=(2, dimension))
            x /= max(np.linalg.norm(x), 1.0)
            z /= max(np.linalg.norm(z), 1.0)
            y, y_prime = rng.uniform(-1.0, 1.0, size=2)
            gram = np.outer(x, x) - np.outer(z, z)
            svec = gram[upper].copy()
            svec[offdiag] *= np.sqrt(2.0)
            cross = x * y - z * y_prime
            normalized_l1 = (
                np.abs(svec).sum() / np.sqrt(2.0 * num_entries)
                + np.abs(cross).sum() / (2.0 * np.sqrt(dimension))
            )
            assert normalized_l1 <= 5.0 * np.sqrt(2.0) / 4.0 + 1e-12


def test_scalar_joint_laplace_substitution_bound_is_attained():
    # x=R, x'=R/4, opposite extreme labels saturate the bound.
    x, x_prime, y, y_prime = 1.0, 0.25, 1.0, -1.0
    sensitivity = (
        abs(x * x - x_prime * x_prime)
        + abs(x * y - x_prime * y_prime) / 2.0
    )
    assert sensitivity == 25.0 / 16.0


def test_scalar_joint_gaussian_substitution_bound():
    # The simple certified bound sqrt(4/3) exceeds the exact maximum.
    rng = np.random.default_rng(121)
    for _ in range(1_000):
        x, x_prime, y, y_prime = rng.uniform(-1.0, 1.0, size=4)
        squared = (
            (x * x - x_prime * x_prime)**2
            + (x * y - x_prime * y_prime)**2 / 4.0
        )
        assert squared <= 4.0 / 3.0 + 1e-12
