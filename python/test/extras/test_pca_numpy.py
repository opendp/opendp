from dataclasses import replace
from typing import Any

import pytest
import opendp.prelude as dp
from opendp._lib import import_optional_dependency
from ..helpers import optional_dependency

pytest.importorskip("sklearn")
from opendp.extras.sklearn import decomposition


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


@pytest.mark.parametrize("d_in", [1, 2])
def test_pca_measurement_is_calibrated(d_in):
    make_private_pca = decomposition.make_private_pca
    then_private_pca = decomposition.then_private_pca

    num_columns = 4
    num_rows = 100
    with optional_dependency("numpy"):
        space = (
            dp.numpy.array2_domain(
                norm=1,
                p=2,
                origin=0,
                num_columns=num_columns,
                size=num_rows,
                nan=False,
                T=float,
            ),
            dp.symmetric_distance(),
        )
    with optional_dependency("scipy.linalg"):
        m_pca = space >> then_private_pca(
            output_measure=dp.max_divergence(), d_in=d_in, d_out=1.0
        )
    assert m_pca.check(d_in, 1.0)
    assert m_pca.map(d_in) <= 1.0

    with optional_dependency("scipy.linalg"):
        direct_pca = make_private_pca(
            space[0], space[1], dp.max_divergence(), d_in=d_in, d_out=1.0
        )
    assert direct_pca.map(d_in) <= 1.0
    estimator = dp.sklearn.decomposition.PCA()
    measurement = estimator.make(
        dp.sklearn.sklearn_fit_domain(space[0]),
        space[1],
        dp.max_divergence(),
        d_in,
        1.0,
    )
    assert measurement.map(d_in) <= 1.0


def test_pca_tutorial_examples():
    import doctest
    from pathlib import Path

    tutorial = (
        Path(__file__).resolve().parents[3]
        / "docs/source/getting-started/statistical-modeling/pca.rst"
    )
    assert doctest.testfile(str(tutorial), module_relative=False).failed == 0


@pytest.mark.parametrize("T, dtype", [(dp.f32, "float32"), (dp.f64, "float64")])
def test_pca_centers_nonzero_origin_before_sscp(T, dtype, monkeypatch):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import make_private_pca

    origin = np.array([10.0, -20.0], dtype=dtype)
    domain = dp.numpy.array2_domain(
        norm=1.0, p=2, origin=origin, num_columns=2, size=2, nan=False, T=T
    )
    data = origin + np.array([[0.5, 0.0], [0.5, 0.5]], dtype=dtype)
    observations = _capture_sscp_rows(monkeypatch)
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
    centered_domain, centered_rows = observations[0]
    assert np.all(centered_domain.descriptor.origin == 0)
    assert np.array_equal(centered_rows, data - origin)
    assert centered_domain.member(centered_rows)


@pytest.mark.parametrize(
    "overrides, message",
    [
        ({"input_metric": dp.insert_delete_distance()}, "symmetric_distance"),
        ({"output_measure": dp.zero_concentrated_divergence()}, "max_divergence"),
        ({"d_in": 0}, "must be positive"),
        ({"d_out": 0.0}, "must be positive"),
        ({"d_in": 3}, "d_in=1 or d_in=2"),
        ({"size": 1}, "at least two samples"),
        ({"num_columns": 0}, "one feature"),
        ({"norm": 0.0}, "positive finite L2 radius"),
        ({"size": None}, "known input-domain size"),
        ({"num_columns": None}, "known input-domain size"),
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


def test_pca_rejects_malformed_origin_descriptor():
    import numpy as np
    from opendp._internal import _extrinsic_domain

    domain = dp.numpy.array2_domain(T=float, size=10, num_columns=2, norm=1, p=2)
    # array2_domain validates this already; keep just one defensive PCA check
    # for an externally constructed domain, not a matrix of synthetic shapes.
    malformed = _extrinsic_domain(
        "NPArray2Domain(malformed origin)",
        domain.member,
        replace(domain.descriptor, origin=np.zeros(1)),
    )
    with pytest.raises(ValueError, match="domain origin must be scalar or have shape"):
        decomposition.make_private_pca(
            malformed,
            dp.symmetric_distance(),
            dp.max_divergence(),
            2,
            1.0,
        )


@pytest.mark.parametrize(
    "clip_norm, message",
    [
        (None, "requires an L2-bounded input domain"),
        (0.0, "positive"),
    ],
)
def test_pca_context_rejects_unbounded_or_invalid_clipping(clip_norm, message):
    context = _context(_legacy_data())
    with pytest.raises(ValueError, match=message):
        context.query().sklearn(decomposition.PCA(clip_norm=clip_norm))
    assert context.current_privacy_loss() == []


def test_pca_release_dimensionality_matches_component_mode():
    np = pytest.importorskip("numpy")
    pytest.importorskip("scipy.linalg")
    domain = dp.numpy.array2_domain(
        norm=1,
        p=2,
        origin=0,
        num_columns=4,
        size=20,
        nan=False,
        T=float,
    )
    rng = np.random.default_rng(0)
    data = rng.normal(size=(20, 4))
    data /= np.maximum(np.linalg.norm(data, axis=1, keepdims=True), 1.0)

    for n_components, expected_rows in ((2, 2), (None, 4), (0.8, 4), ("mle", 4)):
        estimator = dp.sklearn.decomposition.PCA(
            n_components=n_components,
        )
        initial_state = estimator.__dict__.copy()
        measurement = estimator.make(
            dp.sklearn.sklearn_fit_domain(domain),
            dp.symmetric_distance(),
            dp.max_divergence(),
            1,
            1.0,
        )
        assert estimator.__dict__ == initial_state
        release = measurement(dp.sklearn.SklearnFit(data))
        assert estimator.__dict__ == initial_state
        assert release.components.shape == (expected_rows, 4)
        assert release.n_samples == 20
        assert release.n_components == n_components


def _capture_sscp_rows(monkeypatch):
    """Observe the actual centered rows without replacing eigendecomposition."""
    np = pytest.importorskip("numpy")
    from opendp._internal import _make_transformation
    from opendp.extras.sklearn import _make_eigendecomposition as eigen

    observations = []
    original = eigen.make_np_sscp

    def record(input_domain, input_metric):
        def observe(data):
            observations.append((input_domain, np.array(data, copy=True)))
            return data

        identity = _make_transformation(
            input_domain,
            input_metric,
            input_domain,
            input_metric,
            observe,
            lambda distance: distance,
        )
        return identity >> original(input_domain, input_metric)

    monkeypatch.setattr(eigen, "make_np_sscp", record)
    return observations


def _observe_private_mean(monkeypatch, *, replacement):
    """Observe real DP mean execution, then replace its output with a public point."""
    np = pytest.importorskip("numpy")
    from opendp._internal import _make_transformation, _new_pure_function
    from opendp.extras.sklearn import decomposition

    original = decomposition.make_private_np_mean
    observations = []

    def record(input_domain, input_metric, *args, **kwargs):
        measurement = original(input_domain, input_metric, *args, **kwargs)

        def observe_rows(data):
            observations.append(
                {
                    "domain": input_domain,
                    "rows": np.array(data, copy=True),
                    "map_at_two": measurement.map(2),
                }
            )
            return data

        def observe_mean(mean):
            observations[-1]["mean"] = np.array(mean, copy=True)
            return np.array(replacement, dtype=float)

        identity = _make_transformation(
            input_domain,
            input_metric,
            input_domain,
            input_metric,
            observe_rows,
            lambda distance: distance,
        )
        return identity >> measurement >> _new_pure_function(observe_mean)

    monkeypatch.setattr(decomposition, "make_private_np_mean", record)
    return observations


@pytest.mark.parametrize("center", [None, "origin"])
@pytest.mark.parametrize(
    "bound, clip_norm, radius",
    [
        (1.0, None, 1.0),  # domain only
        (None, 0.6, 0.6),  # explicit clipping only
        (1.0, 0.6, 0.6),  # explicit clipping is tighter
        (0.4, 0.8, 0.4),  # original certification is tighter
    ],
)
def test_pca_effective_ball_and_same_rows_for_mean_and_eigens(
    monkeypatch,
    center,
    bound,
    clip_norm,
    radius,
):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import make_private_pca

    origin = np.zeros(2) if bound is None else np.array([1.0, -0.5])
    offsets = np.tile([[0.7, 0.5], [0.7, 0.7], [-0.5, -0.2], [-0.2, 0.3]], (10, 1))
    if bound is not None:
        offsets *= bound  # All rows satisfy the original certified domain.
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
    means = _observe_private_mean(monkeypatch, replacement=origin)
    sscp = _capture_sscp_rows(monkeypatch)
    measurement = make_private_pca(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        2,
        1.0,
        clip_norm=clip_norm,
        center=center,
    )
    assert means == []  # Construction must not evaluate private rows.
    release = measurement(data)
    expected = offsets / np.maximum(
        np.linalg.norm(offsets, axis=1, keepdims=True) / radius, 1
    )
    assert len(sscp) == 1
    centered_domain, centered_rows = sscp[0]
    assert centered_domain.descriptor.norm == radius
    assert centered_domain.descriptor.p == 2
    assert centered_domain.member(centered_rows)
    np.testing.assert_allclose(centered_rows, expected, atol=1e-12)
    np.testing.assert_allclose(release.mean, origin, atol=1e-12)
    if center is None:
        assert len(means) == 1
        np.testing.assert_allclose(means[0]["rows"], origin + expected, atol=1e-12)
        mean_desc = means[0]["domain"].descriptor
        assert mean_desc.p == 1
        assert mean_desc.norm > np.sqrt(2) * radius  # Upward-rounded certification.
        np.testing.assert_array_equal(mean_desc.origin, origin)
    else:
        assert means == []
    assert measurement.map(2) <= 1.0
    np.testing.assert_array_equal(data, before)


def test_pca_projects_actual_private_mean_then_reclips_near_boundary(monkeypatch):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import make_private_pca

    origin = np.array([2.0, -1.0])
    public_point = origin + [3.0, 4.0]  # Deliberately outside the unit ball.
    projected = origin + [0.6, 0.8]
    offsets = np.tile([[-0.6, -0.79], [-0.8, -0.5], [0.7, 0.4], [0.2, 0.9]], (10, 1))
    data = origin + offsets
    domain = dp.numpy.array2_domain(
        T=float,
        size=len(data),
        num_columns=2,
        nan=False,
        norm=1,
        p=2,
        origin=origin,
    )
    means = _observe_private_mean(monkeypatch, replacement=public_point)
    sscp = _capture_sscp_rows(monkeypatch)
    release = make_private_pca(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        2,
        1.0,
    )(data)
    assert len(means) == 1 and "mean" in means[0]  # An actual DP mean was evaluated.
    np.testing.assert_allclose(release.mean, projected, atol=1e-12)
    assert np.linalg.norm(release.mean - origin) <= 1 + 1e-12
    expected = data - projected
    assert np.max(np.linalg.norm(expected, axis=1)) > 1.9
    expected /= np.maximum(np.linalg.norm(expected, axis=1, keepdims=True), 1)
    centered_domain, rows = sscp[0]
    assert centered_domain.descriptor.norm == 1
    np.testing.assert_allclose(rows, expected, atol=1e-12)
    assert centered_domain.member(rows)


@pytest.mark.parametrize(
    "center, width", [(None, 4), ("origin", 4), (None, 1), ("origin", 1)]
)
@pytest.mark.parametrize("d_in", [1, 2])
def test_pca_fixed_budget_split_and_canonical_two_edit_calibration(
    monkeypatch, center, width, d_in
):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn import decomposition

    budget = 0.9
    allocations = []
    original_eigen = decomposition.then_private_np_eigendecomposition

    def record_eigen(values, vectors, **kwargs):
        allocations.append((values, vectors))
        return original_eigen(values, vectors, **kwargs)

    monkeypatch.setattr(
        decomposition, "then_private_np_eigendecomposition", record_eigen
    )
    means = _observe_private_mean(monkeypatch, replacement=np.zeros(width))
    data = _legacy_data() if width == 4 else np.linspace(0.2, 0.8, 10)[:, None]
    domain = dp.numpy.array2_domain(
        T=float,
        size=len(data),
        num_columns=width,
        norm=1,
        p=2,
        nan=False,
    )
    measurement = decomposition.make_private_pca(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        d_in,
        budget,
        center=center,
        num_components=2 if width == 4 else None,
    )
    measurement(data)
    eigen_budget = budget * 2 / 3 if center is None else budget
    if center is None:
        assert len(means) == 1
        mean_budget = means[0]["map_at_two"] * d_in / 2
        assert mean_budget <= budget / 3
        assert mean_budget == pytest.approx(budget / 3)
    else:
        assert means == []
    values, vectors = allocations[-1]
    assert len(vectors) == (2 if width == 4 else 0)
    assert len(set(vectors)) <= 1
    normalized_values = values * d_in / 2
    assert normalized_values == pytest.approx(
        eigen_budget / 2 if vectors else eigen_budget
    )
    if vectors:
        assert sum(vectors) * d_in / 2 == pytest.approx(eigen_budget / 2)
    assert measurement.check(d_in, budget)
    assert measurement.map(d_in) <= budget


@pytest.mark.parametrize(
    "center, d_in, width, budget",
    [
        (None, 1, 17, 1e-6),
        ("origin", 1, 8, 0.1),
        (None, 2, 129, 3.7),
        ("origin", 2, 33, 1.0),
    ],
)
def test_pca_many_samples_rounding_stays_within_budget(
    monkeypatch, center, d_in, width, budget
):
    import numpy as np
    from opendp._internal import _make_measurement
    from opendp.extras._utilities import to_then
    from opendp.extras.sklearn._make_eigendecomposition import (
        make_private_np_eigendecomposition,
    )

    eigen_maps = []

    def record_eigen(input_domain, input_metric, values, vectors):
        actual = make_private_np_eigendecomposition(
            input_domain, input_metric, values, vectors
        )
        eigen_maps.append(actual.map(d_in))
        # Keep the real stage's privacy map; skip only the expensive sampler.
        return _make_measurement(
            input_domain,
            input_metric,
            actual.output_measure,
            lambda data: (np.ones(width), np.eye(width)),
            actual.map,
        )

    monkeypatch.setattr(
        decomposition, "then_private_np_eigendecomposition", to_then(record_eigen)
    )
    _observe_private_mean(monkeypatch, replacement=np.zeros(width))
    domain = dp.numpy.array2_domain(
        T=float,
        size=10,
        num_columns=width,
        norm=1,
        p=2,
        nan=False,
    )
    measurement = decomposition.make_private_pca(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        d_in,
        budget,
        center=center,
    )
    measurement(np.zeros((10, width)))  # Also construct the lazy private-mean stage.
    eigen_budget = budget * 2 / 3 if center is None else budget
    assert eigen_maps and eigen_maps[-1] <= eigen_budget
    assert measurement.check(d_in, budget)
    assert measurement.map(d_in) <= budget


@pytest.mark.parametrize("d_in", [1, 2])
def test_pca_one_feature_origin_uses_no_private_mean_or_vector_samples(
    monkeypatch, d_in
):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn import decomposition
    from opendp.extras.sklearn import _make_eigendecomposition as eigen

    def unexpected(*args, **kwargs):
        pytest.fail(
            "one-feature origin PCA needs neither a private mean nor a vector sample"
        )

    monkeypatch.setattr(decomposition, "make_private_np_mean", unexpected)
    monkeypatch.setattr(eigen, "then_private_eigenvectors", unexpected)
    domain = dp.numpy.array2_domain(
        T=float, size=10, num_columns=1, norm=1, p=2, nan=False
    )
    measurement = decomposition.make_private_pca(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        d_in,
        1.0,
        center="origin",
    )
    release = measurement(np.ones((10, 1)))
    np.testing.assert_array_equal(release.components, [[1.0]])
    assert measurement.map(d_in) == pytest.approx(1.0)
    model = decomposition.PCA(center="origin")
    assert (
        model.fit(
            np.ones((10, 1)),
            domain=dp.sklearn.sklearn_fit_domain(domain),
            privacy_unit=dp.unit_of(contributions=d_in),
            privacy_loss=dp.loss_of(epsilon=1),
        )
        is model
    )
    assert model.components_.shape == (1, 1)


@pytest.mark.parametrize("center", [None, "origin"])
def test_pca_make_snapshots_center_clip_and_selection_without_mutating_state(
    monkeypatch, center
):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCA

    data = _legacy_data() * 3
    before = data.copy()
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, size=len(data), num_columns=4, nan=False),
    )
    point = np.array([0.2, -0.1, 0.1, 0.0])
    means = _observe_private_mean(monkeypatch, replacement=point)
    sscp = _capture_sscp_rows(monkeypatch)
    model = PCA(n_components=2, clip_norm=1, center=center)
    initial = model.__dict__.copy()
    measurement = model.make(
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
    assert release.n_components == 2
    assert release.components.shape == (2, 4)
    assert len(means) == (1 if center is None else 0)
    np.testing.assert_allclose(release.mean, point if center is None else np.zeros(4))
    assert sscp[0][0].descriptor.norm == 1
    assert np.max(np.linalg.norm(sscp[0][1], axis=1)) > 0.5
    assert np.max(np.linalg.norm(sscp[0][1], axis=1)) <= 1 + 1e-12
    np.testing.assert_array_equal(data, before)
    model._ingest_release(release)
    assert model.n_components_ == 2


def test_pca_fit_transform_rejects_before_fitting(monkeypatch):
    import numpy as np
    from opendp.extras.sklearn.decomposition import PCA

    def unexpected_fit(*args, **kwargs):
        pytest.fail("fit_transform must not fit or spend privacy budget")

    monkeypatch.setattr(PCA, "fit", unexpected_fit)
    model = PCA(n_components=1, center="origin")
    with pytest.raises(NotImplementedError, match="make_transform"):
        model.fit_transform(
            np.array([[0.1], [0.2]]),
            privacy_unit=dp.unit_of(contributions=2),
            privacy_loss=dp.loss_of(epsilon=1),
        )
    assert not hasattr(model, "components_")


@pytest.mark.parametrize("variance", [0.0, 1.0])
def test_pca_mle_one_feature_retains_one_component(variance):
    import numpy as np
    from opendp.extras.sklearn.decomposition import PCA, PCARelease

    model = PCA(n_components="mle")
    model._ingest_release(
        PCARelease(
            mean=np.zeros(1),
            singular_values=np.array([variance**0.5]),
            components=np.ones((1, 1)),
            n_samples=2,
            n_components="mle",
        )
    )
    assert model.n_components_ == 1
    assert model.components_.shape == (1, 1)
    np.testing.assert_array_equal(model.explained_variance_, [variance])


def test_pca_ingest_release_error_paths():
    np = pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    from opendp.extras.sklearn.decomposition import PCA, PCARelease

    def release(*, n_samples, n_components=None):
        return PCARelease(
            mean=np.zeros(3),
            singular_values=np.ones(3),
            components=np.eye(3),
            n_samples=n_samples,
            n_components=n_components,
        )

    with pytest.raises(TypeError, match="expected a PCARelease"):
        PCA()._ingest_release(object())  # type: ignore[arg-type]
    with pytest.raises(ValueError, match="at least two samples"):
        PCA()._ingest_release(release(n_samples=1))

    with pytest.raises(ValueError, match="n_samples >= n_features"):
        PCA()._ingest_release(release(n_samples=2, n_components="mle"))

    with pytest.raises(ValueError, match=r"in \(0, 1\)"):
        PCA()._ingest_release(release(n_samples=10, n_components=1.0))

    with pytest.raises(ValueError, match="between 1"):
        PCA()._ingest_release(release(n_samples=10, n_components=0))

    with pytest.raises(ValueError, match="None, an integer"):
        PCA()._ingest_release(release(n_samples=10, n_components="invalid"))

    short_release = PCARelease(
        mean=np.zeros(3),
        singular_values=np.ones(1),
        components=np.ones((1, 3)),
        n_samples=10,
    )
    with pytest.raises(ValueError, match="incompatible"):
        PCA()._ingest_release(short_release)

    valid_release = PCARelease(
        mean=np.zeros(3),
        singular_values=np.array([3.0, 2.0, 1.0]),
        components=np.eye(3),
        n_samples=10,
    )
    for estimator in (PCA(n_components="mle"), PCA(n_components=0.8)):
        estimator._ingest_release(
            replace(valid_release, n_components=estimator.n_components)
        )
        assert estimator.n_components_ >= 1


@pytest.mark.parametrize(
    "n_components,expected", [(1, 1), (0.8, 1), ("mle", 1), (None, 2)]
)
def test_pca_zero_released_variance(n_components, expected):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCARelease

    model = dp.sklearn.decomposition.PCA(n_components=n_components, whiten=True)
    model._ingest_release(
        PCARelease(
            mean=np.zeros(2),
            singular_values=np.zeros(2),
            components=np.eye(2),
            n_samples=10,
            n_components=n_components,
        )
    )
    assert model.n_components_ == expected
    np.testing.assert_array_equal(model.explained_variance_ratio_, np.zeros(expected))
    assert np.isfinite(model.transform(np.ones((1, 2)))).all()


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
    from opendp.extras.sklearn.decomposition import PCA, make_private_pca

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
            1,
            1.0,
            num_components=n_components,
        )
    with pytest.raises(ValueError, match=message):
        PCA(n_components=n_components).make(
            dp.sklearn.sklearn_fit_domain(domain),
            dp.symmetric_distance(),
            dp.max_divergence(),
            1,
            1.0,
        )


def _legacy_data():
    np = import_optional_dependency("numpy")
    rng = np.random.default_rng(0)
    data = rng.normal(size=(10, 4))
    data /= np.maximum(np.linalg.norm(data, axis=1, keepdims=True), 1.0)
    return data


def test_pca_standalone_requires_privacy_and_public_size():
    from inspect import signature
    from opendp.extras.sklearn._estimator import DPEstimator
    from opendp.extras.sklearn.decomposition import PCA

    assert signature(PCA.fit) == signature(DPEstimator.fit)
    model = PCA(clip_norm=1.0)
    with pytest.raises(TypeError, match="required keyword-only"):
        model.fit(_legacy_data())  # type: ignore[call-arg]
    with pytest.raises(ValueError, match="known input-domain size"):
        model.fit(
            _legacy_data(),
            privacy_unit=dp.unit_of(contributions=2),
            privacy_loss=dp.loss_of(epsilon=1),
        )


def test_pca_fitted_model_pickle_preserves_inference():
    import pickle
    import numpy as np
    from sklearn.base import clone

    model = dp.sklearn.decomposition.PCA(n_components=2, clip_norm=1.0)
    data = _legacy_data()
    query = _context(data).query().sklearn(model)

    # Serialize in DP postprocessing, before the separate FFI invocation-
    # metadata layer, whose private-input retention is being fixed separately.
    def serialize(fitted):
        serialized = pickle.dumps(fitted)
        restored = pickle.loads(serialized)
        np.testing.assert_allclose(restored.transform(data), fitted.transform(data))
        np.testing.assert_allclose(restored.get_covariance(), fitted.get_covariance())
        return serialized

    fitted = pickle.loads(query.new_with(chain=query.resolve() >> serialize).release())
    assert clone(fitted).get_params() == model.get_params()
    assert not hasattr(clone(fitted), "components_")
    assert fitted is not model
    assert not hasattr(model, "components_")


def _released_pca(*, whiten):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCA, PCARelease

    model = PCA(n_components=2, whiten=whiten)
    model._ingest_release(
        PCARelease(
            mean=np.array([0.5, -1.0, 2.0]),
            singular_values=np.sqrt(np.array([4.0, 2.0, 1.0]) * 9),
            components=np.eye(3),
            n_samples=10,
            n_components=2,
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
    from sklearn.decomposition import PCA as SklearnPCA
    from opendp.extras.sklearn.decomposition import PCA

    model = PCA(n_components=n_components, whiten=True, clip_norm=1.5, center="origin")
    assert isinstance(model, SklearnPCA)
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


@pytest.mark.parametrize("use_fit", [False, True])
@pytest.mark.parametrize("whiten", [False, True])
def test_pca_sklearn_fit_and_transform(forbid_sklearn_fitting, use_fit, whiten):
    np = pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    pytest.importorskip("scipy.linalg")
    data = _legacy_data() * 100  # Clipping is part of fitting, not a prior query step.
    size, n_features = data.shape
    X_domain = dp.numpy.array2_domain(T=float, size=size, num_columns=n_features)
    y_domain = dp.vector_domain(dp.atom_domain(T=int), size=size)
    domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain)
    aligned = dp.sklearn.SklearnFit(data.copy(), list(range(size)))
    context = dp.Context.compositor(
        aligned,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )
    query = context.query()
    original_chain = query._chain
    model = dp.sklearn.decomposition.PCA(n_components=2, whiten=whiten, clip_norm=1.0)
    if use_fit:
        assert (
            model.fit(
                aligned.X,
                aligned.y,
                domain=domain,
                privacy_unit=dp.unit_of(contributions=1),
                privacy_loss=dp.loss_of(epsilon=1.0),
            )
            is model
        )
    else:
        fitted_query = query.sklearn(model)
        assert fitted_query.resolve().map(1) <= 1.0
        assert context.current_privacy_loss() == []
        model.set_params(n_components=3, clip_norm=0.1, center="origin")
        template = model
        model = fitted_query.release()
        assert model is not template
        assert not hasattr(template, "components_")
        assert model.get_params() == dict(
            n_components=2, whiten=whiten, clip_norm=1.0, center=None
        )
        assert context.current_privacy_loss() == [1.0]
    assert query._chain is original_chain
    assert model.n_features_in_ == n_features
    assert model.n_samples_ == size

    measurement = model.make(
        domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
    )
    assert measurement.input_domain == domain

    transform = model.make_transform(domain, dp.symmetric_distance())
    result = transform(aligned)
    np.testing.assert_allclose(result.X, model.transform(data))
    assert result.X.shape == (size, 2)
    assert transform.output_domain.member(result)
    assert transform.map(1) == 1
    assert result.y is aligned.y
    np.testing.assert_array_equal(aligned.X, data)

    # The transformation freezes both parameters and learned arrays.
    model.set_params(whiten=not whiten)
    model.mean_[:] = 99
    model.components_[:] = 0
    model.explained_variance_[:] = 1
    np.testing.assert_allclose(transform(aligned).X, result.X)


def test_pca_make_transform_validation_and_empty_data():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCARelease

    model = dp.sklearn.decomposition.PCA(n_components=1)
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=2)
    )
    metric = dp.symmetric_distance()
    with pytest.raises(ValueError, match="not fitted"):
        model.make_transform(domain, metric)
    model._ingest_release(
        PCARelease(
            mean=np.zeros(2),
            singular_values=np.array([2.0, 1.0]),
            components=np.eye(2),
            n_samples=10,
            n_components=1,
        )
    )
    transformation = model.make_transform(domain, metric)
    assert transformation.map(1) == 1
    assert transformation(dp.sklearn.SklearnFit(np.zeros((0, 2)))).X.shape == (0, 1)
    with pytest.raises(ValueError, match="symmetric_distance"):
        model.make_transform(domain, dp.change_one_distance())
    with pytest.raises(ValueError, match="number of features"):
        model.make_transform(
            dp.sklearn.sklearn_fit_domain(
                dp.numpy.array2_domain(T=float, num_columns=3)
            ),
            metric,
        )


@pytest.mark.parametrize("supervised", [False, True])
def test_pca_sklearn_transform_composes_with_next_private_estimator(supervised):
    np = pytest.importorskip("numpy")
    data = _legacy_data()
    X_domain = dp.numpy.array2_domain(
        T=float, size=len(data), num_columns=data.shape[1]
    )
    y_domain = (
        dp.vector_domain(dp.atom_domain(T=int), size=len(data)) if supervised else None
    )
    fit_domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain)
    inputs = dp.sklearn.SklearnFit(data.copy(), [0] * len(data) if supervised else None)
    context = dp.Context.compositor(
        inputs,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=fit_domain,
        split_evenly_over=2,
    )
    first = dp.sklearn.decomposition.PCA(n_components=2, clip_norm=1.0)
    first = context.query().sklearn(first).release()
    transform = first.make_transform(fit_domain, dp.symmetric_distance())
    second_query = context.query().new_with(chain=transform)
    second = dp.sklearn.decomposition.PCA(n_components=1, clip_norm=1.0)
    fitted = second_query.sklearn(second).release()
    assert fitted is not second
    assert not hasattr(second, "components_")
    assert fitted.n_features_in_ == 2
    assert fitted.components_.shape == (1, 2)
    assert transform(inputs).y is inputs.y
    np.testing.assert_array_equal(inputs.X, data)


def test_pca_sklearn_transform_leaves_retained_complete_rows_unchanged():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCARelease

    model = dp.sklearn.decomposition.PCA(n_components=1)
    model._ingest_release(
        PCARelease(
            mean=np.array([0.5, 0.0]),
            singular_values=np.array([2.0, 1.0]),
            components=np.eye(2),
            n_samples=10,
        )
    )
    X_domain = dp.numpy.array2_domain(T=float, num_columns=2)
    y_domain = dp.vector_domain(dp.atom_domain(T=int))
    domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain)
    transform = model.make_transform(domain, dp.symmetric_distance())
    before = dp.sklearn.SklearnFit(np.array([[1.0, 2.0]]), [0])
    after = dp.sklearn.SklearnFit(np.array([[1.0, 2.0], [100.0, 200.0]]), [0, 1])
    out_before, out_after = transform(before), transform(after)
    np.testing.assert_array_equal(out_before.X, out_after.X[:1])
    assert out_before.y == out_after.y[:1]
    assert transform.map(1) == 1
