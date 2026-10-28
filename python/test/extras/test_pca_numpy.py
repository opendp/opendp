from dataclasses import replace

import pytest
import opendp.prelude as dp
from opendp._lib import import_optional_dependency
from ..helpers import optional_dependency


def _context(data, *, n_components=None, whiten=False, split=1):
    domain = dp.numpy.array2_domain(
        norm=None,
        size=len(data),
        num_columns=data.shape[1],
        nan=False,
        T=float,
    )
    context = dp.Context.compositor(
        data=dp.sklearn.SklearnFit(data),
        domain=dp.sklearn.sklearn_fit_domain(domain),
        privacy_unit=dp.unit_of(contributions=2),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=split,
    )
    model = dp.sklearn.decomposition.PCA(
        n_components=n_components,
        whiten=whiten,
        clip_norm=1.0,
    )
    return context, model


@pytest.mark.parametrize("d_in", [1, 2])
def test_pca_measurement_is_calibrated(d_in):
    from opendp.extras.sklearn.decomposition import (
        make_private_pca,
        then_private_pca,
    )

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
    with pytest.raises(ValueError, match="PCA currently supports d_in=1 or d_in=2"):
        make_private_pca(
            space[0], space[1], dp.max_divergence(), d_in=3, d_out=1.0
        )

    estimator = dp.sklearn.decomposition.PCA()
    measurement = estimator.make(
        dp.sklearn.sklearn_fit_domain(space[0]), space[1], dp.max_divergence(), d_in, 1.0
    )
    assert measurement.map(d_in) <= 1.0


def test_pca_tutorial_examples():
    import doctest
    from pathlib import Path

    tutorial = Path(__file__).resolve().parents[3] / "docs/source/getting-started/statistical-modeling/pca.rst"
    assert doctest.testfile(str(tutorial), module_relative=False).failed == 0


def _standalone_pca_query():
    from opendp.context import Query

    domain = dp.numpy.array2_domain(
        size=10,
        num_columns=4,
        nan=False,
        T=float,
    )
    return Query(
        chain=(dp.sklearn.sklearn_fit_domain(domain), dp.symmetric_distance()),
        output_measure=dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )


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
        domain, dp.symmetric_distance(), dp.max_divergence(), 2, 1.0,
        center="origin",
    )

    assert domain.member(data)
    assert measurement.check(2, 1.0)
    measurement(data)
    centered_domain, centered_rows = observations[0]
    assert np.all(centered_domain.descriptor.origin == 0)
    assert np.array_equal(centered_rows, data - origin)
    assert centered_domain.member(centered_rows)


def test_pca_requires_known_domain_shape():
    pytest.importorskip("numpy")
    pytest.importorskip("scipy.linalg")
    domain = dp.numpy.array2_domain(num_columns=4, nan=False, T=float)
    with pytest.raises(ValueError, match="known input-domain size"):
        dp.sklearn.decomposition.PCA().make(
            dp.sklearn.sklearn_fit_domain(domain), dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
        )


def test_pca_context_requires_bounded_domain_before_release(monkeypatch):
    pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    from opendp.context import Query

    released = []
    monkeypatch.setattr(Query, "release", lambda self: released.append(True))
    with pytest.raises(ValueError, match="requires an L2-bounded input domain"):
        _standalone_pca_query().sklearn(dp.sklearn.decomposition.PCA(n_components=2))
    assert released == []


def test_pca_context_rejects_invalid_clip_norm_before_release(monkeypatch):
    pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    from opendp.context import Query

    released = []
    monkeypatch.setattr(Query, "release", lambda self: released.append(True))
    model = dp.sklearn.decomposition.PCA(n_components=2, clip_norm=0.0)
    with pytest.raises(ValueError, match="positive"):
        _standalone_pca_query().sklearn(model)
    assert released == []


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
            dp.sklearn.sklearn_fit_domain(domain), dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
        )
        assert estimator.__dict__ == initial_state
        release = measurement(dp.sklearn.SklearnFit(data))
        assert estimator.__dict__ == initial_state
        assert release.components.shape == (expected_rows, 4)
        assert release.n_samples == 20
        assert release.components.shape[1] == 4
        assert release.n_components == n_components
        assert not hasattr(release, "n_features")


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
            input_domain, input_metric, input_domain, input_metric,
            observe, lambda distance: distance,
        )
        return identity >> original(input_domain, input_metric)

    monkeypatch.setattr(eigen, "make_np_sscp", record)
    return observations


def _observe_private_mean(monkeypatch, *, replacement=None):
    """Keep the actual DP mean, optionally postprocess its result to a public point."""
    np = pytest.importorskip("numpy")
    from opendp._internal import _make_transformation, _new_pure_function
    from opendp.extras.sklearn import decomposition

    original = decomposition.make_private_np_mean
    observations = []

    def record(input_domain, input_metric, *args, **kwargs):
        measurement = original(input_domain, input_metric, *args, **kwargs)

        def observe_rows(data):
            observations.append({
                "domain": input_domain,
                "rows": np.array(data, copy=True),
                "map_at_two": measurement.map(2),
            })
            return data

        def observe_mean(mean):
            observations[-1]["mean"] = np.array(mean, copy=True)
            return mean if replacement is None else np.array(replacement, dtype=float)

        identity = _make_transformation(
            input_domain, input_metric, input_domain, input_metric,
            observe_rows, lambda distance: distance,
        )
        return identity >> measurement >> _new_pure_function(observe_mean)

    monkeypatch.setattr(decomposition, "make_private_np_mean", record)
    return observations


@pytest.mark.parametrize("center", [None, "origin"])
@pytest.mark.parametrize("bound, clip_norm, radius", [
    (1.0, None, 1.0),  # domain only
    (None, 0.6, 0.6),  # explicit clipping only
    (1.0, 0.6, 0.6),  # explicit clipping is tighter
    (0.4, 0.8, 0.4),  # original certification is tighter
])
def test_pca_effective_ball_and_same_rows_for_mean_and_eigens(
    monkeypatch, center, bound, clip_norm, radius,
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
        T=float, size=len(data), num_columns=2, nan=False,
        norm=bound, p=2 if bound is not None else None,
        origin=origin if bound is not None else None,
    )
    assert domain.member(data)
    means = _observe_private_mean(monkeypatch, replacement=origin)
    sscp = _capture_sscp_rows(monkeypatch)
    measurement = make_private_pca(
        domain, dp.symmetric_distance(), dp.max_divergence(), 2, 1.0,
        clip_norm=clip_norm, center=center,
    )
    release = measurement(data)
    expected = offsets / np.maximum(np.linalg.norm(offsets, axis=1, keepdims=True) / radius, 1)
    assert len(sscp) == 1
    centered_domain, centered_rows = sscp[0]
    assert centered_domain.descriptor.norm == radius
    assert centered_domain.descriptor.p == 2
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


def test_pca_sanitizes_nan_permitting_ball_without_explicit_clipping(monkeypatch):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import make_private_pca

    data = np.array([[np.nan, 10.0], [0.1, 0.2], [-0.3, 0.1]])
    domain = dp.numpy.array2_domain(T=float, size=3, num_columns=2, norm=0.5, p=2, nan=True)
    assert domain.member(data)
    sscp = _capture_sscp_rows(monkeypatch)
    measurement = make_private_pca(
        domain, dp.symmetric_distance(), dp.max_divergence(), 2, 1.0, center="origin",
    )
    measurement(data)
    centered_domain, rows = sscp[0]
    assert np.isfinite(rows).all()
    assert centered_domain.member(rows)
    assert np.max(np.linalg.norm(rows, axis=1)) <= 0.5
    assert np.isnan(data[0, 0])  # Sanitization does not mutate the private input.
    assert measurement.map(2) <= 1.0


@pytest.mark.parametrize("scalar_origin", [0.0, 2.0])
def test_pca_scalar_and_vector_origins_match_without_mean_or_redundant_clip(monkeypatch, scalar_origin):
    np = pytest.importorskip("numpy")
    from opendp._internal import _extrinsic_domain
    from opendp.extras.sklearn import decomposition

    def unexpected(*args, **kwargs):
        pytest.fail("finite certified origin-mode data needs neither a mean nor clipping")

    monkeypatch.setattr(decomposition, "make_private_np_mean", unexpected)
    monkeypatch.setattr(decomposition, "make_np_clamp", unexpected)
    origin = np.full(2, scalar_origin)
    offsets = np.tile([[0.7, 0.5], [0.7, 0.7]], (20, 1))
    data = origin + offsets
    vector_domain = dp.numpy.array2_domain(
        T=float, size=len(data), num_columns=2, nan=False, norm=1, p=2, origin=origin,
    )
    # array2_domain normally expands zero scalars itself. Exercise the scalar
    # descriptor directly, with the same certified ball and membership test.
    scalar_domain = _extrinsic_domain(
        "NPArray2Domain(scalar origin)", vector_domain.member,
        replace(vector_domain.descriptor, origin=np.array(scalar_origin)),
    )
    for domain in (scalar_domain, vector_domain):
        model = decomposition.PCA(center="origin")
        model.fit(
            data, domain=dp.sklearn.sklearn_fit_domain(domain),
            privacy_unit=dp.unit_of(contributions=2), privacy_loss=dp.loss_of(epsilon=1),
        )
        assert model.mean_.shape == (2,)
        np.testing.assert_array_equal(model.mean_, origin)
        assert not np.allclose(data.mean(axis=0), model.mean_)
        # Random bases need not match; their full-rank projected geometry must.
        scores = model.transform(data)
        np.testing.assert_allclose(scores @ scores.T, offsets @ offsets.T)


@pytest.mark.parametrize("origin", [[0.0], [0.0, 0.0, 0.0], [[0.0, 0.0]]])
def test_pca_rejects_incompatible_origin_shapes(origin):
    np = pytest.importorskip("numpy")
    from opendp._internal import _extrinsic_domain
    from opendp.extras.sklearn.decomposition import make_private_pca

    domain = dp.numpy.array2_domain(T=float, size=10, num_columns=2, norm=1, p=2, nan=False)
    domain = _extrinsic_domain(
        "NPArray2Domain(incompatible origin)", domain.member,
        replace(domain.descriptor, origin=np.array(origin)),
    )
    with pytest.raises(ValueError, match="domain origin must be scalar or have shape"):
        make_private_pca(domain, dp.symmetric_distance(), dp.max_divergence(), 2, 1.0)


@pytest.mark.parametrize("d_in", [1, 2])
def test_pca_bounded_default_executes_and_accounts_for_private_mean(monkeypatch, d_in):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import make_private_pca

    data = np.tile([[0.7, 0.5], [0.7, 0.7]], (20, 1))
    domain = dp.numpy.array2_domain(
        T=float, size=len(data), num_columns=2, nan=False, norm=1, p=2,
    )
    means = _observe_private_mean(monkeypatch)
    measurement = make_private_pca(
        domain, dp.symmetric_distance(), dp.max_divergence(), d_in, 1.0,
    )
    assert means == []  # Constructing a measurement does not evaluate private data.
    release = measurement(data)
    assert len(means) == 1
    np.testing.assert_array_equal(means[0]["rows"], data)
    raw = means[0]["mean"]
    np.testing.assert_allclose(release.mean, raw / max(1.0, np.linalg.norm(raw)), atol=1e-12)
    assert means[0]["map_at_two"] * d_in / 2 <= 1 / 3
    assert means[0]["map_at_two"] * d_in / 2 == pytest.approx(1 / 3)
    assert measurement.map(d_in) <= 1.0
    assert measurement.map(d_in) == pytest.approx(1.0)


def test_pca_projects_actual_private_mean_then_reclips_near_boundary(monkeypatch):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import make_private_pca

    origin = np.array([2.0, -1.0])
    public_point = origin + [3.0, 4.0]  # Deliberately outside the unit ball.
    projected = origin + [0.6, 0.8]
    offsets = np.tile([[-0.6, -0.79], [-0.8, -0.5], [0.7, 0.4], [0.2, 0.9]], (10, 1))
    data = origin + offsets
    domain = dp.numpy.array2_domain(
        T=float, size=len(data), num_columns=2, nan=False, norm=1, p=2, origin=origin,
    )
    means = _observe_private_mean(monkeypatch, replacement=public_point)
    sscp = _capture_sscp_rows(monkeypatch)
    release = make_private_pca(
        domain, dp.symmetric_distance(), dp.max_divergence(), 2, 1.0,
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
    assert np.max(np.linalg.norm(rows, axis=1)) <= 1 + 1e-12


@pytest.mark.parametrize("center, width", [(None, 4), ("origin", 4), (None, 1), ("origin", 1)])
@pytest.mark.parametrize("d_in", [1, 2])
def test_pca_fixed_budget_split_and_canonical_two_edit_calibration(monkeypatch, center, width, d_in):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn import decomposition

    budget = 0.9
    allocations = []
    original_eigen = decomposition.then_private_np_eigendecomposition

    def record_eigen(values, vectors, **kwargs):
        allocations.append((values, vectors))
        return original_eigen(values, vectors, **kwargs)

    monkeypatch.setattr(decomposition, "then_private_np_eigendecomposition", record_eigen)
    means = _observe_private_mean(monkeypatch, replacement=np.zeros(width))
    data = _legacy_data() if width == 4 else np.linspace(0.2, 0.8, 10)[:, None]
    domain = dp.numpy.array2_domain(
        T=float, size=len(data), num_columns=width, norm=1, p=2, nan=False,
    )
    measurement = decomposition.make_private_pca(
        domain, dp.symmetric_distance(), dp.max_divergence(), d_in, budget,
        center=center, num_components=2 if width == 4 else None,
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
    assert normalized_values == pytest.approx(eigen_budget / 2 if vectors else eigen_budget)
    if vectors:
        assert sum(vectors) * d_in / 2 == pytest.approx(eigen_budget / 2)
    assert measurement.check(d_in, budget)
    assert measurement.map(d_in) <= budget


@pytest.mark.parametrize("center, d_in, width, budget", [
    (None, 1, 17, 1e-6), ("origin", 1, 8, 0.1),
    (None, 2, 129, 3.7), ("origin", 2, 33, 1.0),
])
def test_pca_many_samples_rounding_stays_within_budget(monkeypatch, center, d_in, width, budget):
    from opendp.extras.sklearn.decomposition import make_private_pca

    domain = dp.numpy.array2_domain(
        T=float, size=10, num_columns=width, norm=1, p=2, nan=False,
    )
    measurement = make_private_pca(
        domain, dp.symmetric_distance(), dp.max_divergence(), d_in, budget, center=center,
    )
    # Private-mean mode constructs its eigen stage lazily. A public fixed mean
    # exercises that stage's rounding without sampling 128 Bingham directions.
    if center is None:
        np = pytest.importorskip("numpy")
        from opendp.extras.sklearn import decomposition

        original_eigen = decomposition.then_private_np_eigendecomposition
        eigen_maps = []

        def record_eigen(*args, **kwargs):
            stage = original_eigen(*args, **kwargs)
            # Retain stage construction/privacy accounting, but do not execute
            # the stochastic sampler: the rounding invariant is in its map.
            def make_stage(input_domain, input_metric):
                from opendp._internal import _make_measurement
                actual = (input_domain, input_metric) >> stage
                eigen_maps.append(actual.map(d_in))
                return _make_measurement(
                    input_domain, input_metric, actual.output_measure,
                    lambda data: (np.ones(width), np.eye(width)), actual.map,
                )
            from opendp.mod import _PartialConstructor
            return _PartialConstructor(make_stage)

        # Force evaluation of the lazy eigen constructor, not the real sampler.
        monkeypatch.setattr(decomposition, "then_private_np_eigendecomposition", record_eigen)
        measurement(np.zeros((10, width)))
        assert eigen_maps and eigen_maps[-1] <= budget * 2 / 3
    assert measurement.check(d_in, budget)
    assert measurement.map(d_in) <= budget


@pytest.mark.parametrize("d_in", [1, 2])
def test_pca_one_feature_origin_uses_no_private_mean_or_vector_samples(monkeypatch, d_in):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn import decomposition
    from opendp.extras.sklearn import _make_eigendecomposition as eigen

    def unexpected(*args, **kwargs):
        pytest.fail("one-feature origin PCA needs neither a private mean nor a vector sample")

    monkeypatch.setattr(decomposition, "make_private_np_mean", unexpected)
    monkeypatch.setattr(eigen, "then_private_eigenvectors", unexpected)
    domain = dp.numpy.array2_domain(T=float, size=10, num_columns=1, norm=1, p=2, nan=False)
    measurement = decomposition.make_private_pca(
        domain, dp.symmetric_distance(), dp.max_divergence(), d_in, 1.0, center="origin",
    )
    release = measurement(np.ones((10, 1)))
    np.testing.assert_array_equal(release.components, [[1.0]])
    assert measurement.map(d_in) == pytest.approx(1.0)
    model = decomposition.PCA(center="origin")
    assert model.fit(
        np.ones((10, 1)), domain=dp.sklearn.sklearn_fit_domain(domain),
        privacy_unit=dp.unit_of(contributions=d_in), privacy_loss=dp.loss_of(epsilon=1),
    ) is model
    assert model.components_.shape == (1, 1)


@pytest.mark.parametrize("center", [None, "origin"])
def test_pca_make_snapshots_center_clip_and_selection_without_mutating_state(monkeypatch, center):
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
    measurement = model.make(domain, dp.symmetric_distance(), dp.max_divergence(), 2, 1.0)
    assert measurement.input_domain == domain
    assert measurement.map(2) <= 1.0
    assert model.__dict__ == initial
    model.set_params(n_components=3, clip_norm=0.1, center="origin" if center is None else None)
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


@pytest.mark.parametrize("center", ["mean", "zero", False, 0, [0.0, 0.0]])
def test_pca_rejects_invalid_center(center):
    from opendp.extras.sklearn.decomposition import make_private_pca

    domain = dp.numpy.array2_domain(T=float, size=10, num_columns=2, norm=1, p=2, nan=False)
    with pytest.raises(ValueError, match="center must be None or 'origin'"):
        make_private_pca(
            domain, dp.symmetric_distance(), dp.max_divergence(), 2, 1.0, center=center,
        )


@pytest.mark.parametrize("clip_norm", [0, -1, float("inf"), float("nan")])
def test_pca_rejects_invalid_clip_norm(clip_norm):
    from opendp.extras.sklearn.decomposition import make_private_pca

    domain = dp.numpy.array2_domain(T=float, size=10, num_columns=2, norm=1, p=2, nan=False)
    with pytest.raises(ValueError, match="positive and finite"):
        make_private_pca(
            domain, dp.symmetric_distance(), dp.max_divergence(), 2, 1.0, clip_norm=clip_norm,
        )


def test_pca_nonzero_mean_regression_without_l1_truncation():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCA

    data = np.tile([[0.7, 0.5], [0.7, 0.7]], (200, 1))
    assert np.all(np.linalg.norm(data, ord=1, axis=1) > 1)
    assert np.all(np.linalg.norm(data, axis=1) < 1)
    domain = dp.sklearn.sklearn_fit_domain(dp.numpy.array2_domain(
        T=float, size=len(data), num_columns=2, norm=1, p=2, nan=False,
    ))
    model = PCA(n_components=1)
    model.fit(
        data, domain=domain,
        privacy_unit=dp.unit_of(contributions=2), privacy_loss=dp.loss_of(epsilon=10000),
    )
    np.testing.assert_allclose(model.mean_, [0.7, 0.6], atol=0.005)
    assert abs(model.components_[0, 1]) > 0.98
    assert abs(model.components_[0, 0]) < 0.2
    assert model.explained_variance_[0] == pytest.approx(0.01, abs=0.002)


def test_pca_whitening_uses_a_machine_epsilon_floor():
    np = pytest.importorskip("numpy")
    model = dp.sklearn.decomposition.PCA()
    model.whiten = True
    model.components_ = np.eye(2)
    model.mean_ = np.zeros(2)
    model.n_features_in_ = 2
    model.n_components_ = 2
    model.explained_variance_ = np.array([1.0, 1e-300])

    X = np.array([[0.5, -0.25]])
    transformed = model.transform(X)
    assert np.isfinite(transformed).all()
    # sklearn's inverse uses the raw variances, not the forward epsilon floor.
    scale = np.sqrt(model.explained_variance_)
    np.testing.assert_allclose(transformed, X / np.maximum(scale, np.finfo(float).eps))
    np.testing.assert_allclose(model.inverse_transform(transformed), transformed * scale)


def test_pca_fitted_methods_and_error_paths():
    np = pytest.importorskip("numpy")

    estimator = dp.sklearn.decomposition.PCA(n_components=2, whiten=True)
    public = np.zeros((3, 3))
    # Use sklearn's native unfitted-model behavior (NotFittedError is also an
    # AttributeError; some sklearn versions access fitted arrays directly).
    with pytest.raises(AttributeError):
        estimator.transform(public)
    with pytest.raises(AttributeError):
        estimator.inverse_transform(public[:, :2])
    with pytest.raises(AttributeError):
        estimator.get_covariance()
    with pytest.raises(AttributeError):
        estimator.get_precision()
    with pytest.raises(TypeError, match="required keyword-only"):
        estimator.fit_transform(public)

    with pytest.raises(TypeError, match="expected a PCARelease"):
        estimator._ingest_release(object())  # type: ignore[arg-type]
    estimator = _released_pca(whiten=True)
    with pytest.raises(ValueError, match="features"):
        estimator.transform(np.zeros((3, 2)))
    with pytest.raises(ValueError):
        estimator.inverse_transform(np.zeros((3, 3)))


def test_pca_release_requires_samples_and_derives_feature_count():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCA, PCARelease

    arrays = dict(mean=np.zeros(3), singular_values=np.ones(3), components=np.eye(3))
    with pytest.raises(TypeError, match="n_samples"):
        PCARelease(**arrays)
    model = PCA()
    model._ingest_release(PCARelease(**arrays, n_samples=10, n_components=2))
    assert model.n_features_in_ == 3
    assert model.n_components_ == 2


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
        estimator._ingest_release(replace(valid_release, n_components=estimator.n_components))
        assert estimator.n_components_ >= 1


@pytest.mark.parametrize("n_components,expected", [(1, 1), (0.8, 1), (None, 2)])
def test_pca_zero_released_variance(n_components, expected):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCARelease

    model = dp.sklearn.decomposition.PCA(n_components=n_components, whiten=True)
    model._ingest_release(PCARelease(
        mean=np.zeros(2), singular_values=np.zeros(2),
        components=np.eye(2), n_samples=10, n_components=n_components,
    ))
    assert model.n_components_ == expected
    np.testing.assert_array_equal(model.explained_variance_ratio_, np.zeros(expected))
    assert np.isfinite(model.transform(np.ones((1, 2)))).all()


@pytest.mark.parametrize("n_components, size, message", [
    (0, 10, "between 1"),
    (5, 10, "between 1"),
    (1.0, 10, r"in \(0, 1\)"),
    (True, 10, "None, an integer"),
    ("invalid", 10, "None, an integer"),
    ("mle", 3, "n_samples >= n_features"),
])
def test_pca_component_validation(n_components, size, message):
    pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCA, make_private_pca

    domain = dp.numpy.array2_domain(
        T=float, size=size, num_columns=4, norm=1, p=2, nan=False,
    )
    with pytest.raises(ValueError, match=message):
        make_private_pca(
            domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0,
            num_components=n_components,
        )
    with pytest.raises(ValueError, match=message):
        PCA(n_components=n_components).make(
            dp.sklearn.sklearn_fit_domain(domain), dp.symmetric_distance(),
            dp.max_divergence(), 1, 1.0,
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
    fitted = _context(data)[0].query().sklearn(model).release()
    restored = pickle.loads(pickle.dumps(fitted))
    np.testing.assert_allclose(restored.transform(data), fitted.transform(data))
    np.testing.assert_allclose(restored.get_covariance(), fitted.get_covariance())
    assert clone(fitted).get_params() == model.get_params()
    assert not hasattr(clone(fitted), "components_")
    assert fitted is model


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_pca_refitting_replaces_state_and_freezes_old_transform(route):
    np = pytest.importorskip("numpy")
    data = _legacy_data()
    context, model = _context(data, n_components=1, whiten=True, split=2)
    domain = context.accountant.input_domain

    def fit():
        if route == "context":
            return context.query().sklearn(model).release()
        return model.fit(data, domain=domain, privacy_unit=dp.unit_of(contributions=1),
                         privacy_loss=dp.loss_of(epsilon=1))

    assert fit() is model
    transform = model.make_transform(domain, dp.symmetric_distance())
    before = transform(dp.sklearn.SklearnFit(data)).X
    model.mean_[:] = 99
    model.components_[:] = 0
    model.explained_variance_[:] = 1
    model.set_params(n_components=2, whiten=False)
    assert fit() is model
    assert model.n_components_ == 2
    assert model.components_.shape == (2, 4)
    np.testing.assert_allclose(transform(dp.sklearn.SklearnFit(data)).X, before)
    if route == "context":
        assert context.current_privacy_loss() == [0.5, 0.5]


@pytest.mark.parametrize("n_components, expected", [(None, 4), (2, 2)])
def test_pca_query_snapshots_component_selection(n_components, expected):
    context, model = _context(_legacy_data(), n_components=n_components)
    query = context.query().sklearn(model)
    query.resolve()
    assert not hasattr(model, "components_")
    assert context.current_privacy_loss() == []
    model.set_params(n_components=3, clip_norm=2.0, center="origin")
    assert query.release() is model
    assert model.n_components_ == expected
    assert context.current_privacy_loss() == [1.0]
    state = model.components_.copy()
    with pytest.raises(dp.OpenDPException, match="out of queries"):
        query.release()
    np = pytest.importorskip("numpy")
    np.testing.assert_array_equal(model.components_, state)


def _released_pca(*, whiten=False, n_components=2, variances=(4.0, 2.0, 1.0)):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCA, PCARelease

    model = PCA(n_components=n_components, whiten=whiten, clip_norm=1.0)
    model._ingest_release(PCARelease(
        mean=np.array([0.5, -1.0, 2.0]), singular_values=np.sqrt(np.asarray(variances) * 9),
        components=np.eye(3), n_samples=10, n_components=n_components,
    ))
    return model


@pytest.fixture
def forbid_sklearn_fitting(monkeypatch):
    from sklearn.decomposition import PCA as SklearnPCA

    def nonprivate_fit(*args, **kwargs):
        pytest.fail("sklearn's non-private PCA fitting routine was called")

    for name in ("fit", "fit_transform", "_fit", "_fit_full", "_fit_truncated"):
        monkeypatch.setattr(SklearnPCA, name, nonprivate_fit)


@pytest.mark.parametrize("route, whiten", [("fit", False), ("fit_transform", True), ("context", False)])
def test_pca_fitting_never_calls_sklearn_training(forbid_sklearn_fitting, route, whiten):
    np = pytest.importorskip("numpy")
    data = _legacy_data()
    context, model = _context(data, n_components=2, whiten=whiten)
    if route == "context":
        assert context.query().sklearn(model).release() is model
        assert context.current_privacy_loss() == [1.0]
    else:
        kwargs = dict(domain=context.accountant.input_domain,
                      privacy_unit=dp.unit_of(contributions=2), privacy_loss=dp.loss_of(epsilon=1))
        if route == "fit":
            assert model.fit(data, **kwargs) is model
        else:
            result = model.fit_transform(data, **kwargs)
            assert result.shape == (len(data), 2)
            np.testing.assert_allclose(result, model.transform(data))
    assert model.components_.shape == (2, 4)


@pytest.mark.parametrize("n_components", [2, 0.8, "mle"])
def test_pca_clone_preserves_only_supported_parameters(n_components):
    from sklearn.base import clone
    from sklearn.decomposition import PCA as SklearnPCA
    from opendp.extras.sklearn.decomposition import PCA

    model = PCA(n_components=n_components, whiten=True, clip_norm=1.5, center="origin")
    assert isinstance(model, SklearnPCA)
    assert model.get_params() == {"n_components": n_components, "whiten": True, "clip_norm": 1.5, "center": "origin"}
    assert clone(model).get_params() == model.get_params()
    for name in ("transform", "inverse_transform", "get_covariance", "get_precision",
                 "score", "score_samples", "get_feature_names_out"):
        assert getattr(PCA, name) is getattr(SklearnPCA, name)


@pytest.mark.parametrize("whiten", [False, True])
def test_pca_inherited_inference_from_release_without_training(forbid_sklearn_fitting, whiten):
    np = pytest.importorskip("numpy")
    from sklearn.utils.validation import check_is_fitted

    model = _released_pca(whiten=whiten)
    attributes = ("mean_", "components_", "n_components_", "n_features_in_", "n_samples_",
                  "explained_variance_", "explained_variance_ratio_", "singular_values_", "noise_variance_")
    check_is_fitted(model, attributes=attributes)
    X = np.array([[1.0, 2.0, 3.0], [4.0, -1.0, 0.0]])
    expected = (X - model.mean_) @ model.components_.T
    if whiten:
        expected /= np.sqrt(model.explained_variance_)
    np.testing.assert_allclose(model.transform(X), expected)
    unscaled = expected * np.sqrt(model.explained_variance_) if whiten else expected
    np.testing.assert_allclose(model.inverse_transform(expected), unscaled @ model.components_ + model.mean_)

    covariance = model.get_covariance()
    precision = np.linalg.inv(covariance)
    centered = X - model.mean_
    expected_scores = -0.5 * (np.sum((centered @ precision) * centered, axis=1)
                              + 3 * np.log(2 * np.pi) + np.linalg.slogdet(covariance)[1])
    np.testing.assert_allclose(model.score_samples(X), expected_scores)
    np.testing.assert_allclose(model.score(X), expected_scores.mean())
    np.testing.assert_array_equal(model.get_feature_names_out(), ["pca0", "pca1"])
    np.testing.assert_array_equal(model.get_feature_names_out(["a", "b", "c"]), ["pca0", "pca1"])
    domain = dp.sklearn.sklearn_fit_domain(dp.numpy.array2_domain(T=float, num_columns=3))
    transform = model.make_transform(domain, dp.symmetric_distance())
    np.testing.assert_allclose(transform(dp.sklearn.SklearnFit(X)).X, expected)
    assert transform.map(2) == 2


@pytest.mark.parametrize("whiten", [False, True])
@pytest.mark.parametrize("n_components, variances", [(2, (4.0, 2.0, 1.0)),
                                                     (2, (4.0, 2.0, 0.0)), (3, (4.0, 2.0, 1.0))])
def test_pca_inherited_covariance_and_precision(forbid_sklearn_fitting, monkeypatch, whiten, n_components, variances):
    np = pytest.importorskip("numpy")
    linalg = pytest.importorskip("scipy.linalg")
    model = _released_pca(whiten=whiten, n_components=n_components, variances=variances)
    components = model.components_.copy()
    if whiten:
        components *= np.sqrt(model.explained_variance_)[:, None]
    covariance = ((components.T * np.maximum(model.explained_variance_ - model.noise_variance_, 0)) @ components
                  + model.noise_variance_ * np.eye(3))
    np.testing.assert_allclose(model.get_covariance(), covariance)
    if n_components < 3 and model.noise_variance_ == 0:
        # Zero residual variance makes a reduced-rank covariance singular.
        with pytest.raises(np.linalg.LinAlgError):
            model.get_precision()
        return

    inverse_shapes = []
    inverse = linalg.inv

    def record_inverse(matrix, *args, **kwargs):
        inverse_shapes.append(matrix.shape)
        return inverse(matrix, *args, **kwargs)

    monkeypatch.setattr(linalg, "inv", record_inverse)
    np.testing.assert_allclose(model.get_precision(), np.linalg.inv(covariance))
    if n_components < 3:
        # sklearn applies the matrix inversion lemma to the smaller matrix.
        assert inverse_shapes == [(n_components, n_components)]


@pytest.mark.parametrize("use_fit", [False, True])
@pytest.mark.parametrize("whiten", [False, True])
def test_pca_sklearn_fit_and_transform(use_fit, whiten):
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
        aligned, dp.unit_of(contributions=1), dp.loss_of(epsilon=1.0),
        domain=domain, split_evenly_over=1,
    )
    query = context.query()
    original_chain = query._chain
    model = dp.sklearn.decomposition.PCA(n_components=2, whiten=whiten, clip_norm=1.0)
    if use_fit:
        assert model.fit(
            aligned.X, aligned.y,
            domain=domain, privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(epsilon=1.0),
        ) is model
    else:
        fitted_query = query.sklearn(model)
        assert fitted_query.resolve().map(1) <= 1.0
        template = model
        model = fitted_query.release()
        assert model is template
    assert query._chain is original_chain
    assert model.n_features_in_ == n_features
    assert model.n_samples_ == size

    measurement = model.make(domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0)
    assert measurement.input_domain == domain

    transform = model.make_transform(domain, dp.symmetric_distance())
    result = transform(aligned)
    np.testing.assert_allclose(result.X, model.transform(data))
    assert result.X.shape == (size, 2)
    assert transform.output_domain.member(result)
    assert transform.map(1) == 1
    assert result.y is aligned.y
    np.testing.assert_array_equal(aligned.X, data)

    # Once constructed, the transformation depends on the released snapshot,
    # not on subsequent estimator parameter changes.
    model.set_params(whiten=not whiten)
    np.testing.assert_allclose(transform(aligned).X, result.X)


def test_pca_make_transform_validation_and_empty_data():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCARelease

    model = dp.sklearn.decomposition.PCA(n_components=1)
    domain = dp.sklearn.sklearn_fit_domain(dp.numpy.array2_domain(T=float, num_columns=2))
    metric = dp.symmetric_distance()
    with pytest.raises(ValueError, match="not fitted"):
        model.make_transform(domain, metric)
    model._ingest_release(PCARelease(
        mean=np.zeros(2), singular_values=np.array([2.0, 1.0]),
        components=np.eye(2), n_samples=10, n_components=1,
    ))
    transformation = model.make_transform(domain, metric)
    assert transformation.map(1) == 1
    assert transformation(dp.sklearn.SklearnFit(np.zeros((0, 2)))).X.shape == (0, 1)
    with pytest.raises(ValueError, match="symmetric_distance"):
        model.make_transform(domain, dp.change_one_distance())
    with pytest.raises(ValueError, match="number of features"):
        model.make_transform(
            dp.sklearn.sklearn_fit_domain(dp.numpy.array2_domain(T=float, num_columns=3)), metric,
        )


@pytest.mark.parametrize("supervised", [False, True])
def test_pca_sklearn_transform_composes_with_next_private_estimator(supervised):
    np = pytest.importorskip("numpy")
    data = _legacy_data()
    X_domain = dp.numpy.array2_domain(T=float, size=len(data), num_columns=data.shape[1])
    y_domain = dp.vector_domain(dp.atom_domain(T=int), size=len(data)) if supervised else None
    fit_domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain)
    inputs = dp.sklearn.SklearnFit(data.copy(), [0] * len(data) if supervised else None)
    context = dp.Context.compositor(
        inputs, dp.unit_of(contributions=1), dp.loss_of(epsilon=1.0),
        domain=fit_domain, split_evenly_over=2,
    )
    first = dp.sklearn.decomposition.PCA(n_components=2, clip_norm=1.0)
    first = context.query().sklearn(first).release()
    transform = first.make_transform(fit_domain, dp.symmetric_distance())
    second_query = context.query().new_with(chain=transform)
    second = dp.sklearn.decomposition.PCA(n_components=1, clip_norm=1.0)
    fitted = second_query.sklearn(second).release()
    assert fitted is second
    assert fitted.n_features_in_ == 2
    assert fitted.components_.shape == (1, 2)
    assert transform(inputs).y is inputs.y
    np.testing.assert_array_equal(inputs.X, data)


def test_pca_sklearn_transform_leaves_retained_complete_rows_unchanged():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCARelease

    model = dp.sklearn.decomposition.PCA(n_components=1)
    model._ingest_release(PCARelease(
        mean=np.array([0.5, 0.0]), singular_values=np.array([2.0, 1.0]),
        components=np.eye(2), n_samples=10,
    ))
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
