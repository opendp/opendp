"""Objective perturbation tests: numerical certificate and OpenDP privacy maps."""

import numpy as np
import opendp.prelude as dp
import pytest

pytest.importorskip("sklearn")
pytest.importorskip("scipy")
from sklearn.base import clone
from opendp.extras.sklearn.linear_model import LogisticRegression
from opendp.extras.sklearn.linear_model._objective_perturbation import (
    clip_training_data,
    logistic_objective_and_gradient,
    optimize_perturbed_logistic,
    sample_spherical_laplace,
)


@pytest.fixture(autouse=True)
def enabled():
    dp.enable_features("contrib", "honest-but-curious", "idealized-numerics")


def domain(size=None):
    return dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=2, size=size),
        dp.vector_domain(dp.atom_domain(T=float), size=size),
    )


def data():
    rng = np.random.default_rng(14)
    x = rng.normal(size=(80, 2))
    y = (x[:, 0] - x[:, 1] + 0.1 * rng.normal(size=80) > 0).astype(float)
    return dp.sklearn.SklearnFit(x, y)


def test_clipping_and_gradient_bound():
    d = dp.sklearn.SklearnFit(
        np.array([[3., 4.], [np.inf, np.nan], [0., 0.]]),
        np.array([1., 0., 0.9]),
    )
    x, y = clip_training_data(d, 2., True)
    np.testing.assert_allclose(np.linalg.norm(x[:, :-1], axis=1), [2, 0, 0])
    np.testing.assert_array_equal(y, [1, 0, 1])
    assert np.max(np.linalg.norm(x, axis=1)) <= np.sqrt(5) + 1e-12


def test_residual_certificate_even_with_one_lbfgs_step():
    rng = np.random.default_rng(1)
    x = rng.normal(size=(100, 3))
    y = rng.integers(0, 2, 100).astype(float)
    ridge, linear = 3., rng.normal(size=3)
    w = optimize_perturbed_logistic(x, y, ridge, linear, 1e-8, max_iter=1)
    residual = logistic_objective_and_gradient(w, x, y, ridge, linear)[1]
    assert np.linalg.norm(residual) <= 1e-8


def test_spherical_laplace_radius_mean():
    rng = np.random.default_rng(4)
    radii = [np.linalg.norm(sample_spherical_laplace(3, 2., rng)) for _ in range(2000)]
    assert abs(np.mean(radii) - 6.) < 0.4


@pytest.mark.parametrize("fit_intercept", [False, True])
@pytest.mark.parametrize("size,d_in", [(None, 1), (80, 2)])
def test_privacy_map_and_shape(fit_intercept, size, d_in):
    model = LogisticRegression(fit_intercept=fit_intercept, C=1., tol=1e-7, max_iter=5)
    m = model.make_measurement(
        domain(size), dp.symmetric_distance(), dp.max_divergence(), d_in, 2.0
    )
    assert m.map(d_in) <= 2. + 1e-10
    release = np.asarray(m(data()))
    assert release.shape == (2 + int(fit_intercept),)
    assert np.isfinite(release).all()


def test_zcdp_conversion_and_clone():
    model = LogisticRegression(feature_norm=2, C=3, max_iter=10)
    assert clone(model).get_params() == model.get_params()
    m = model.make_measurement(
        domain(80), dp.symmetric_distance(), dp.zero_concentrated_divergence(), 2, 1.0
    )
    assert m.map(2) <= 1. + 1e-10


def test_sized_inputs_reject_add_remove_distance_one():
    with pytest.raises(ValueError, match="even symmetric distance"):
        LogisticRegression().make_measurement(
            domain(80), dp.symmetric_distance(), dp.max_divergence(), 1, 1.
        )


@pytest.mark.parametrize("name,value", [
    ("C", 0), ("C", -1), ("tol", 0), ("feature_norm", float("nan")),
    ("max_iter", 0),
])
def test_invalid_public_parameters(name, value):
    with pytest.raises(ValueError, match=name):
        LogisticRegression(**{name: value}).make_measurement(
            domain(), dp.symmetric_distance(), dp.max_divergence(), 1, 1.
        )


def test_prediction_interface():
    model = LogisticRegression()
    model._ingest_release(np.array([0.4, -0.2, 0.1]))
    p = model.predict_proba(np.zeros((3, 2)))
    assert p.shape == (3, 2)
    np.testing.assert_allclose(p.sum(axis=1), 1)
    assert set(model.predict(np.zeros((3, 2)))).issubset({0, 1})

def test_objective_rowwise_inference_is_trusted():
    from opendp.extras.sklearn._rowwise import _make_rowwise_transform
    model = LogisticRegression()
    model._ingest_release(np.array([2., -1., 0.]))
    input_domain = domain()
    trans = _make_rowwise_transform(
        input_domain, dp.symmetric_distance(), model, method="predict"
    )
    assert trans.map(1) == 1
    out = trans(data())
    assert out.X.shape == (80, 1)
