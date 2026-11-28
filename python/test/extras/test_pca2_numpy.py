from typing import Any

import pytest
import opendp.prelude as dp
from opendp._lib import import_optional_dependency

pytest.importorskip("sklearn")
from opendp.extras.sklearn import decomposition2 as decomposition


def _rowwise(model, domain, metric, *, method="transform"):
    from opendp.extras.sklearn._rowwise import _make_rowwise_transform

    return _make_rowwise_transform(domain, metric, model, method=method)


def _context(data):
    domain = dp.numpy.array2_domain(
        norm=None,
        size=len(data),
        num_columns=data.shape[1],
        nan=False,
        T=float,
    )
    return dp.Context.compositor(
        data=dp.sklearn.SklearnFit(data),
        domain=dp.sklearn.sklearn_fit_domain(domain),
        privacy_unit=dp.unit_of(contributions=2),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )


@pytest.mark.parametrize("T, dtype", [(dp.f32, "float32"), (dp.f64, "float64")])
def test_pca_centers_nonzero_origin_before_sscp(T, dtype, monkeypatch):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition2 import make_private_pca

    origin = np.array([10.0, -20.0], dtype=dtype)
    domain = dp.numpy.array2_domain(
        norm=1.0, p=2, origin=origin, num_columns=2, size=2, nan=False, T=T
    )
    data = origin + np.array([[0.5, 0.0], [0.5, 0.5]], dtype=dtype)
    observations = _observe_moments(monkeypatch)
    measurement = make_private_pca(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        2,
        1.0,
        center="origin",
    )

    assert domain.member(data)
    assert measurement.check(2, 1.0)
    measurement(data)
    centered_domain, centered_rows, _ = observations[0]
    assert np.all(centered_domain.descriptor.origin == 0)
    assert np.array_equal(centered_rows, data - origin)
    assert centered_domain.member(centered_rows)


@pytest.mark.parametrize(
    "overrides, message",
    [
        ({"input_metric": dp.insert_delete_distance()}, "symmetric_distance"),
        ({"output_measure": dp.approximate(dp.max_divergence())}, "max_divergence"),
        ({"d_in": 0}, "positive integer"),
        ({"d_out": 0.0}, "must be positive"),
        ({"d_in": 3}, "Contribution bounds must be even"),
        ({"size": 1}, "at least two samples"),
        ({"num_columns": 0}, "one feature"),
        ({"norm": 0.0}, "positive finite L2 radius"),
        ({"norm": None, "p": None}, "requires an L2-bounded input domain"),
        ({"num_columns": None}, "known input-domain num_columns"),
        *[
            ({"center": center}, "center must be None or 'origin'")
            for center in ("mean", "zero", False, 0, [0.0, 0.0])
        ],
        *[
            ({"clip_norm": radius}, "positive and finite")
            for radius in (0, -1, float("inf"), float("nan"))
        ],
    ],
)
def test_pca_rejects_invalid_privacy_and_public_domain(overrides, message):
    domain_args: dict[str, Any] = dict(
        T=float, size=10, num_columns=2, norm=1.0, p=2, nan=False
    )
    make_args: dict[str, Any] = dict(
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
        d_in=2,
        d_out=1.0,
        center=None,
        clip_norm=None,
    )
    for name, value in overrides.items():
        (make_args if name in make_args else domain_args)[name] = value
    domain = dp.numpy.array2_domain(**domain_args)
    with pytest.raises(ValueError, match=message):
        decomposition.make_private_pca(domain, **make_args)


def _observe_moments(monkeypatch, *, replacement=None, is_zcdp=False):
    """Observe bounded offsets entering the joint release and optionally fix its output."""
    np = pytest.importorskip("numpy")
    from opendp._internal import _make_transformation, _new_pure_function

    name = "make_private_np_moments" if is_zcdp else "make_private_np_wishart_moments"
    original = getattr(decomposition, name)
    observed = []

    def record(domain, metric, distance, budget, *, estimate_mean):
        result = original(domain, metric, distance, budget, estimate_mean=estimate_mean)
        measurement, q = (result, None) if is_zcdp else result

        def observe(data):
            observed.append((domain, np.array(data, copy=True), measurement.map(distance)))
            return data

        identity = _make_transformation(
            domain, metric, domain, metric, observe, lambda distance: distance
        )
        measurement = identity >> measurement
        if replacement is not None:
            measurement = measurement >> _new_pure_function(lambda _: replacement)
        return measurement if is_zcdp else (measurement, q)

    monkeypatch.setattr(decomposition, name, record)
    return observed


@pytest.mark.parametrize("center", [None, "origin"])
@pytest.mark.parametrize(
    "bound, clip_norm, radius",
    [
        (1.0, None, 1.0),  # domain only
        (None, 0.6, 0.6),  # explicit clipping only
        (1.0, 0.6, 0.6),  # explicit clipping is tighter
        (0.4, 0.8, 0.4),  # original bound is tighter
    ],
)
def test_pca_effective_ball_for_joint_moments(
    monkeypatch,
    center,
    bound,
    clip_norm,
    radius,
):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition2 import make_private_pca

    origin = np.zeros(2) if bound is None else np.array([1.0, -0.5])
    offsets = np.tile([[0.7, 0.5], [0.7, 0.7], [-0.5, -0.2], [-0.2, 0.3]], (10, 1))
    if bound is not None:
        offsets *= bound  # All rows satisfy the declared domain.
    data = origin + offsets
    before = data.copy()
    domain = dp.numpy.array2_domain(
        T=float,
        size=len(data),
        num_columns=2,
        nan=False,
        norm=bound,
        p=2 if bound is not None else None,
        origin=origin if bound is not None else None,
    )
    assert domain.member(data)
    observed = _observe_moments(
        monkeypatch, replacement=(np.zeros(2), np.eye(2), len(data))
    )
    measurement = make_private_pca(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        2,
        1.0,
        clip_norm=clip_norm,
        center=center,
    )
    assert observed == []  # Construction must not evaluate private rows.
    release = measurement(data)
    expected = offsets / np.maximum(
        np.linalg.norm(offsets, axis=1, keepdims=True) / radius, 1
    )
    assert len(observed) == 1
    centered_domain, centered_rows, spent = observed[0]
    assert centered_domain.descriptor.norm == radius
    assert centered_domain.descriptor.p == 2
    assert centered_domain.member(centered_rows)
    np.testing.assert_allclose(centered_rows, expected, atol=1e-12)
    np.testing.assert_allclose(release.mean, origin, atol=1e-12)
    assert spent <= 1.0
    assert measurement.map(2) <= 1.0
    np.testing.assert_array_equal(data, before)


@pytest.mark.parametrize("is_zcdp", [False, True])
def test_pca_projects_mean_into_released_energy_ball(monkeypatch, is_zcdp):
    np = pytest.importorskip("numpy")
    origin = np.array([2.0, -1.0])
    domain = dp.numpy.array2_domain(
        T=float, size=4, num_columns=2, nan=False, norm=1, p=2, origin=origin
    )
    _observe_moments(
        monkeypatch,
        replacement=(np.array([12.0, 16.0]), np.diag([0.5, 0.5]), 4),
        is_zcdp=is_zcdp,
    )
    measure = dp.zero_concentrated_divergence() if is_zcdp else dp.max_divergence()
    release = decomposition.make_private_pca(
        domain, dp.symmetric_distance(), measure, 2, 1.0
    )(np.tile(origin, (4, 1)))
    np.testing.assert_allclose(release.mean, origin + [0.3, 0.4])
    np.testing.assert_allclose(release.singular_values, 0.0, atol=1e-7)


@pytest.mark.parametrize("d_in", [2, 4])
@pytest.mark.parametrize("is_zcdp", [False, True])
def test_pca_one_feature_origin(d_in, is_zcdp):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn import decomposition2 as decomposition

    domain = dp.numpy.array2_domain(
        T=float, size=10, num_columns=1, norm=1, p=2, nan=False
    )
    measurement = decomposition.make_private_pca(
        domain,
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence() if is_zcdp else dp.max_divergence(),
        d_in,
        1.0,
        center="origin",
    )
    release = measurement(np.ones((10, 1)))
    np.testing.assert_array_equal(release.components, [[1.0]])
    # With a public fixed size, one symmetric edit cannot connect distinct datasets.
    assert measurement.map(d_in) == pytest.approx(1.0)
    assert measurement.map(1) == 0.0
    model = decomposition.PCA(center="origin")
    assert (
        model.fit(
            np.ones((10, 1)),
            domain=dp.sklearn.sklearn_fit_domain(domain),
            privacy_unit=dp.unit_of(contributions=2),
            privacy_loss=dp.loss_of(rho=1) if is_zcdp else dp.loss_of(epsilon=1),
        )
        is model
    )
    assert model.components_.shape == (1, 1)


@pytest.mark.parametrize("center", [None, "origin"])
def test_pca_make_snapshots_center_clip_and_selection_without_mutating_state(
    monkeypatch, center
):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition2 import PCA

    data = _legacy_data() * 3
    before = data.copy()
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, size=len(data), num_columns=4, nan=False),
    )
    point = np.array([0.2, -0.1, 0.1, 0.0])
    observed = _observe_moments(
        monkeypatch, replacement=(point * len(data), np.eye(4), len(data))
    )
    model = PCA(n_components=2, clip_norm=1, center=center)
    initial = model.__dict__.copy()
    measurement = model.make_measurement(
        domain, dp.symmetric_distance(), dp.max_divergence(), 2, 1.0
    )
    assert measurement.input_domain == domain
    assert measurement.map(2) <= 1.0
    assert model.__dict__ == initial
    model.set_params(
        n_components=3, clip_norm=0.1, center="origin" if center is None else None
    )
    changed = model.__dict__.copy()
    release = measurement(dp.sklearn.SklearnFit(data))
    assert model.__dict__ == changed
    assert release.components.shape == (2, 4)
    assert len(observed) == 1
    np.testing.assert_allclose(release.mean, point if center is None else np.zeros(4))
    assert observed[0][0].descriptor.norm == 1
    assert np.max(np.linalg.norm(observed[0][1], axis=1)) > 0.5
    assert np.max(np.linalg.norm(observed[0][1], axis=1)) <= 1 + 1e-12
    np.testing.assert_array_equal(data, before)
    model._ingest_release(release)
    assert model.n_components_ == 2


@pytest.mark.parametrize(
    "selection, eigenvalues, size, expected",
    [
        ("mle", [0.0], 2, 1),
        ("mle", [1.0], 2, 1),
        (0.8, [0.0, 0.0], 10, 1),
        (0.8, [9.0, 1.0, 0.0], 10, 1),
        (0.99, [3.0, 2.0, 1.0], 2, 2),
        (None, [3.0, 2.0, 1.0], 2, 2),
        (1, [0.0, 0.0], 10, 1),
    ],
)
def test_pca_selects_components_from_released_spectrum(
    selection, eigenvalues, size, expected
):
    np = pytest.importorskip("numpy")
    assert (
        decomposition._select_num_components(selection, np.sqrt(eigenvalues), size)
        == expected
    )


@pytest.mark.parametrize(
    "n_components, size, message",
    [
        (0, 10, "between 1"),
        (5, 10, "between 1"),
        (1.0, 10, r"in \(0, 1\)"),
        (True, 10, "None, an integer"),
        ("invalid", 10, "None, an integer"),
        ("mle", 3, "n_samples >= n_features"),
    ],
)
def test_pca_component_validation(n_components, size, message):
    pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition2 import make_private_pca

    domain = dp.numpy.array2_domain(
        T=float,
        size=size,
        num_columns=4,
        norm=1,
        p=2,
        nan=False,
    )
    with pytest.raises(ValueError, match=message):
        make_private_pca(
            domain,
            dp.symmetric_distance(),
            dp.max_divergence(),
            2,
            1.0,
            num_components=n_components,
        )


def _legacy_data():
    np = import_optional_dependency("numpy")
    rng = np.random.default_rng(0)
    data = rng.normal(size=(10, 4))
    data /= np.maximum(np.linalg.norm(data, axis=1, keepdims=True), 1.0)
    return data


def test_pca_standalone_infers_unknown_size_domain():
    model = decomposition.PCA(n_components=1, clip_norm=1.0).fit(
        _legacy_data(),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1),
    )
    assert model.n_samples_ >= 2
    assert model.components_.shape == (1, 4)


def test_pca_standalone_recovers_centered_principal_component(forbid_sklearn_fitting):
    np = pytest.importorskip("numpy")

    direction = np.array([1.0, 1.0, 1.0, 1.0]) / 2
    offset = np.array([1.0, -1.0, 1.0, -1.0]) / 2
    X = np.tile(
        np.stack(
            [
                0.6 * offset + 0.45 * direction,
                0.6 * offset - 0.45 * direction,
            ]
        ),
        (512, 1),
    )
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, size=1024, num_columns=4, nan=False)
    )
    model = decomposition.PCA(n_components=1, clip_norm=1.0).fit(
        X,
        domain=domain,
        privacy_unit=dp.unit_of(contributions=2),
        privacy_loss=dp.loss_of(epsilon=8.0),
    )
    alignment = (model.components_[0] @ direction) ** 2
    assert alignment > 0.90
    assert model.singular_values_[0] == pytest.approx(
        0.45 * np.sqrt(len(X)), rel=0.15
    )


def _released_pca(*, whiten):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition2 import PCA, PCARelease

    model = PCA(n_components=2, whiten=whiten)
    model._ingest_release(
        PCARelease(
            mean=np.array([0.5, -1.0, 2.0]),
            singular_values=np.sqrt(np.array([4.0, 2.0, 1.0]) * 9),
            components=np.eye(3)[:2],
            n_samples=10,
        )
    )
    return model


@pytest.fixture
def forbid_sklearn_fitting(monkeypatch):
    from sklearn.decomposition import PCA as SklearnPCA

    def nonprivate_fit(*args, **kwargs):
        pytest.fail("sklearn's non-private PCA fitting routine was called")

    for name in ("fit", "fit_transform", "_fit", "_fit_full", "_fit_truncated"):
        monkeypatch.setattr(SklearnPCA, name, nonprivate_fit)


@pytest.mark.parametrize("n_components", [2, 0.8, "mle"])
def test_pca_clone_preserves_only_supported_parameters(n_components):
    from sklearn.base import clone
    from sklearn.base import BaseEstimator
    from sklearn.decomposition import PCA as SklearnPCA
    from opendp.extras.sklearn.decomposition2 import PCA

    model = PCA(n_components=n_components, whiten=True, clip_norm=1.5, center="origin")
    assert isinstance(model, BaseEstimator)
    assert not isinstance(model, SklearnPCA)
    assert model.get_params() == {
        "n_components": n_components,
        "whiten": True,
        "clip_norm": 1.5,
        "center": "origin",
    }
    assert clone(model).get_params() == model.get_params()


@pytest.mark.parametrize("whiten", [False, True])
def test_pca_inherited_inference_from_release_without_training(
    forbid_sklearn_fitting, whiten
):
    np = pytest.importorskip("numpy")
    model = _released_pca(whiten=whiten)
    assert model.n_samples_ == 10
    assert model.n_features_in_ == 3
    np.testing.assert_allclose(model.explained_variance_, [4.0, 2.0])
    np.testing.assert_allclose(model.explained_variance_ratio_, [4 / 7, 2 / 7])
    np.testing.assert_array_equal(model.singular_values_, [6.0, (18.0) ** 0.5])
    assert model.noise_variance_ == 1.0
    X = np.array([[1.0, 2.0, 3.0], [4.0, -1.0, 0.0]])
    expected = (X - model.mean_) @ model.components_.T
    if whiten:
        expected /= np.sqrt(model.explained_variance_)
    np.testing.assert_allclose(model.transform(X), expected)
    unscaled = expected * np.sqrt(model.explained_variance_) if whiten else expected
    np.testing.assert_allclose(
        model.inverse_transform(expected), unscaled @ model.components_ + model.mean_
    )

    # Smoke-test representative inherited methods with the released attributes;
    # sklearn owns their detailed covariance/precision/scoring algorithms.
    assert model.get_covariance().shape == (3, 3)
    assert model.get_precision().shape == (3, 3)
    assert np.isfinite(model.score(X))


@pytest.mark.parametrize("supervised", [False, True])
def test_pca_sklearn_transform_composes_with_next_private_estimator(
    supervised, forbid_sklearn_fitting
):
    np = pytest.importorskip("numpy")
    data = _legacy_data()
    X_domain = dp.numpy.array2_domain(
        T=float, size=len(data), num_columns=data.shape[1], nan=False
    )
    y_domain = (
        dp.vector_domain(dp.atom_domain(T=int), size=len(data)) if supervised else None
    )
    fit_domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain)
    inputs = dp.sklearn.SklearnFit(data.copy(), [0] * len(data) if supervised else None)
    context = dp.Context.compositor(
        inputs,
        dp.unit_of(contributions=2),
        dp.loss_of(epsilon=1.0),
        domain=fit_domain,
        split_evenly_over=2,
    )
    template = dp.sklearn.decomposition2.PCA(n_components=2, clip_norm=1.0)
    first = context.query().sklearn(template).release()
    assert not hasattr(template, "components_")
    transform = _rowwise(first, fit_domain, dp.symmetric_distance())
    result = transform(inputs)
    np.testing.assert_allclose(result.X, first.transform(data))
    assert transform.output_domain.member(result)
    assert transform.map(1) == 1
    second_query = context.query().new_with(chain=transform)
    second = dp.sklearn.decomposition2.PCA(n_components=1, clip_norm=1.0)
    fitted = second_query.sklearn(second).release()
    assert fitted is not second
    assert not hasattr(second, "components_")
    assert fitted.n_features_in_ == 2
    assert fitted.components_.shape == (1, 2)
    assert result.y is inputs.y
    np.testing.assert_array_equal(inputs.X, data)


def test_pca_rejects_unsupported_and_unfitted_rowwise_methods():
    model = decomposition.PCA()
    with pytest.raises(ValueError, match="does not support rowwise inference"):
        model.rowwise_output_domain(None, method="predict")
    with pytest.raises(ValueError, match="not fitted"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(T=float, num_columns=2), method="transform"
        )


def test_pca_inverse_transform_rejects_wrong_width():
    np = pytest.importorskip("numpy")
    model = _released_pca(whiten=False)
    with pytest.raises(ValueError, match="Input width"):
        model.inverse_transform(np.ones((2, model.n_components_ + 1)))


@pytest.mark.parametrize("whiten", [False, True])
@pytest.mark.parametrize(
    "method,input_width,output_width",
    [
        ("transform", 3, 2),
        ("inverse_transform", 2, 3),
        ("score_samples", 3, 1),
    ],
)
def test_pca_rowwise_methods_accept_empty_batches(
    whiten, method, input_width, output_width
):
    np = pytest.importorskip("numpy")
    model = _released_pca(whiten=whiten)
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=input_width)
    )
    trans = _rowwise(model, domain, dp.symmetric_distance(), method=method)
    assert trans.map(1) == 1
    assert (
        trans.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
        .X.cast(dp.numpy.NPArray2Domain)
        .num_columns
        == output_width
    )
    for rows in (0, 2):
        X = np.zeros((rows, input_width))
        actual = trans(dp.sklearn.SklearnFit(X)).X
        expected = np.asarray(getattr(model, method)(X))
        expected = expected[:, None] if expected.ndim == 1 else expected
        assert actual.shape == (rows, output_width)
        np.testing.assert_allclose(actual, expected)
        assert trans.output_domain.member(dp.sklearn.SklearnFit(actual))


@pytest.mark.parametrize(
    "method,input_width",
    [
        ("transform", 3),
        ("inverse_transform", 2),
        ("score_samples", 3),
    ],
)
def test_pca_rowwise_inference_nonfinite_and_public_width(method, input_width):
    np = pytest.importorskip("numpy")
    model = _released_pca(whiten=False)
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=input_width)
    )
    trans = _rowwise(model, domain, dp.symmetric_distance(), method=method)
    X = np.full((1, input_width), np.inf)
    result = trans(dp.sklearn.SklearnFit(X))
    assert trans.output_domain.member(result)
    assert trans.map(1) == 1

    bad_width = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=input_width + 1)
    )
    with pytest.raises(ValueError, match="fitted number of features"):
        _rowwise(model, bad_width, dp.symmetric_distance(), method=method)

    bad_dtype = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=int, num_columns=input_width)
    )
    with pytest.raises(ValueError, match="floating-point input"):
        _rowwise(model, bad_dtype, dp.symmetric_distance(), method=method)


def test_pca_selected_sklearn_features_and_model_only_score_failure():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition2 import PCA, PCARelease

    model = _released_pca(whiten=False)
    assert list(model.get_feature_names_out()) == ["pca0", "pca1"]
    with pytest.raises(ValueError, match="does not support rowwise inference"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(T=float, num_columns=3), method="score"
        )

    # A singular released covariance may cause scoring to fail, even for an
    # empty input. The failure is determined solely by the released model.
    singular = PCA(n_components=2)
    singular._ingest_release(
        PCARelease(
            mean=np.zeros(3),
            singular_values=np.array([3.0, 0.0, 0.0]),
            components=np.eye(3)[:2],
            n_samples=10,
        )
    )
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=3)
    )
    trans = _rowwise(singular, domain, dp.symmetric_distance(), method="score_samples")
    assert trans.map(1) == 1
    with pytest.raises(np.linalg.LinAlgError):
        singular.score_samples(np.zeros((0, 3)))


@pytest.mark.parametrize("is_zcdp", [False, True])
@pytest.mark.parametrize("center", [None, "origin"])
@pytest.mark.parametrize("d_in", [1, 3])
def test_pca_unknown_size_uses_private_count_and_centered_sum(
    monkeypatch, is_zcdp, center, d_in
):
    np = pytest.importorskip("numpy")
    origin = np.array([100.0, -200.0])
    observed = _observe_moments(
        monkeypatch,
        replacement=(np.array([0.75, 0.0]), np.diag([-1.0, 2.0]), 3),
        is_zcdp=is_zcdp,
    )
    domain = dp.numpy.array2_domain(
        T=float, num_columns=2, norm=1.0, p=2, origin=origin, nan=False
    )
    measure = dp.zero_concentrated_divergence() if is_zcdp else dp.max_divergence()
    measurement = decomposition.make_private_pca(
        domain, dp.symmetric_distance(), measure, d_in, 1.0, center=center
    )
    assert measurement.check(d_in, 1.0)
    assert measurement.map(0) == 0.0
    release = measurement(np.tile(origin, (5, 1)))
    assert release.n_samples == 3
    descriptor, rows, spent = observed[-1]
    assert descriptor.descriptor.size is None
    assert np.all(descriptor.descriptor.origin == 0)
    assert descriptor.descriptor.p == 2
    assert spent == pytest.approx(1.0)
    np.testing.assert_array_equal(rows, np.zeros((5, 2)))
    np.testing.assert_allclose(
        release.mean, origin + ([0.25, 0.0] if center is None else [0.0, 0.0])
    )
    centered_energy = 1.0 - (0.75**2 / 3 if center is None else 0.0)
    assert np.sum(release.singular_values**2) == pytest.approx(centered_energy)
    model = decomposition.PCA()
    model._ingest_release(release)
    assert model.n_samples_ == 3
    np.testing.assert_allclose(
        model.explained_variance_, release.singular_values**2 / 2
    )


@pytest.mark.parametrize(
    "num_components, expected_count, expected_width",
    [(None, 2, 2), (3, 3, 3), ("mle", 4, None), (0.9, 2, None)],
)
@pytest.mark.parametrize("is_zcdp", [False, True])
def test_pca_unknown_size_empty_input_and_count_floor(
    monkeypatch, num_components, expected_count, expected_width, is_zcdp
):
    np = pytest.importorskip("numpy")
    _observe_moments(
        monkeypatch,
        replacement=(np.zeros(4), np.zeros((4, 4)), 0),
        is_zcdp=is_zcdp,
    )
    domain = dp.numpy.array2_domain(T=float, num_columns=4, norm=1.0, p=2, nan=False)
    measurement = decomposition.make_private_pca(
        domain,
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence() if is_zcdp else dp.max_divergence(),
        2,
        1.0,
        num_components=num_components,
    )
    assert measurement.check(2, 1.0)
    release = measurement(np.empty((0, 4)))
    assert release.n_samples == expected_count
    model = decomposition.PCA()
    model._ingest_release(release)
    assert model.n_samples_ == expected_count
    assert 1 <= model.n_components_ <= min(4, expected_count)
    if expected_width is not None:
        assert model.n_components_ == expected_width
    assert np.isfinite(model.mean_).all()
