from __future__ import annotations

import math

import pytest

import opendp.prelude as dp
from opendp.extras.sklearn._estimator import _DPEstimator
from opendp.extras.sklearn.cluster import (
    SphericalKMeans,
    make_private_spherical_kmeans,
    sparse_binary_domain,
)
from opendp.extras.sklearn.cluster._spherical_lloyd_binary import (
    _check_zcdp_budget,
    _clip_rows,
    _ensure_csr_binary,
    make_cluster_feature_sums,
    make_private_cluster_sizes,
)
from opendp.mod import UnknownTypeException

np = pytest.importorskip("numpy")
sparse = pytest.importorskip("scipy.sparse")

dp.enable_features("contrib")

_TINY_PARAMS = dict(
    max_iter=4,
    max_features_per_center=3,
    max_features_per_record=3,
    random_state=0,
)


def _two_blob_data(reps=1):
    """Two well-separated feature blobs: features {0,1,2} vs {3,4,5}."""
    block = np.array(
        [
            [1, 1, 1, 0, 0, 0],
            [1, 1, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0],
            [0, 0, 0, 1, 1, 1],
            [0, 0, 0, 1, 1, 0],
            [0, 0, 0, 0, 1, 1],
        ],
        dtype=np.float32,
    )
    return sparse.csr_matrix(np.tile(block, (reps, 1)))


def _measurement(domain, *, d_in=1, rho=0.5, **params):
    return make_private_spherical_kmeans(
        domain,
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        d_in,
        rho,
        **({"n_clusters": 2, **_TINY_PARAMS, **params}),
    )


# --------------------------------------------------------------------------
# framework constructors + calibration
# --------------------------------------------------------------------------
def test_constructor_calibrates_and_releases_only_centers():
    x = _two_blob_data()
    m = _measurement(sparse_binary_domain(6))
    assert m.map(1) <= 0.5
    assert m.map(1) == pytest.approx(0.5, rel=1e-9)
    release = m(x)
    assert sparse.issparse(release)
    assert release.shape == (2, 6)


def test_framework_parameters_are_explicit_and_initialization_uses_center_bound():
    domain = sparse_binary_domain(6)
    m = _measurement(
        domain, max_iter=2, max_features_per_record=4, max_features_per_center=1
    )
    assert m(_two_blob_data()).shape == (2, 6)
    with pytest.raises(TypeError, match="config"):
        _measurement(domain, config=object())


@pytest.mark.parametrize(
    ("params", "error"),
    [
        ({"n_clusters": 0}, ValueError),
        ({"max_iter": 0}, ValueError),
        ({"max_features_per_record": 0}, ValueError),
        ({"max_features_per_center": 0}, ValueError),
        ({"random_state": -1}, ValueError),
    ],
)
def test_framework_validates_algorithm_parameters(params, error):
    domain = sparse_binary_domain(6)
    defaults = dict(
        n_clusters=2,
        max_iter=2,
        max_features_per_record=3,
        max_features_per_center=3,
        random_state=1,
    )
    defaults.update(params)
    with pytest.raises(error, match="positive integer|nonnegative integer"):
        make_private_spherical_kmeans(
            domain,
            dp.symmetric_distance(),
            dp.zero_concentrated_divergence(),
            1,
            0.5,
            **defaults,
        )


def test_group_privacy_scales_with_d_in():
    domain = sparse_binary_domain(6)
    m1 = _measurement(domain, d_in=1)
    m2 = _measurement(domain, d_in=2)
    assert m1.map(1) <= 0.5
    assert m2.map(2) <= 0.5
    assert m2.map(1) <= m2.map(2)


def test_requires_symmetric_distance_and_zcdp():
    domain = sparse_binary_domain(6)
    with pytest.raises(ValueError, match="add/remove"):
        make_private_spherical_kmeans(
            domain, dp.l1_distance(T=int), dp.zero_concentrated_divergence(), 1, 0.5
        )
    with pytest.raises(ValueError, match="zero_concentrated"):
        make_private_spherical_kmeans(
            domain, dp.symmetric_distance(), dp.max_divergence(), 1, 0.5
        )


@pytest.mark.parametrize(
    ("d_in", "error"), [(1.5, TypeError), (True, ValueError), (0, ValueError)]
)
def test_rejects_invalid_privacy_distance(d_in, error):
    with pytest.raises(error, match="positive integer"):
        _check_zcdp_budget(dp.zero_concentrated_divergence(), d_in, 0.5)


@pytest.mark.parametrize("d_out", [float("nan"), float("inf"), 0.0])
def test_rejects_invalid_privacy_budget(d_out):
    with pytest.raises(ValueError, match="finite and positive"):
        _check_zcdp_budget(dp.zero_concentrated_divergence(), 1, d_out)


def test_cluster_feature_sums_stability_and_output_shape():
    domain = sparse_binary_domain(6)
    centers = sparse.csr_matrix(
        np.array([[1, 1, 1, 0, 0, 0], [0, 0, 0, 1, 1, 1]], dtype=np.float32)
    )
    t = make_cluster_feature_sums(
        domain, dp.symmetric_distance(), centers=centers, max_active=4
    )
    assert t.map(1) == pytest.approx(math.sqrt(4))
    assert t.map(3) == pytest.approx(3 * math.sqrt(4))
    out = np.asarray(t(_two_blob_data()))
    assert out.shape == (12,)
    assert np.issubdtype(out.dtype, np.integer)


# --------------------------------------------------------------------------
# sparse binary domain
# --------------------------------------------------------------------------
def test_domain_has_no_max_features_per_record():
    domain = sparse_binary_domain(6)
    assert not hasattr(domain.descriptor, "max_active")
    assert domain.descriptor.n_features == 6
    assert domain.descriptor.max_rows == 2**31 - 1


def test_binary_conversion_and_clipping():
    dense = _ensure_csr_binary(np.array([1, 0, 1]), n_features=3)
    assert _ensure_csr_binary(np.array([1, 0])).shape == (2, 1)
    assert _clip_rows(dense, 1).nnz == 1
    sparse_data = sparse.csr_matrix(np.array([[1, 0, 1], [0, 1, 0]], dtype=np.float32))
    assert np.array_equal(
        _ensure_csr_binary(sparse_data).toarray(), sparse_data.toarray()
    )


@pytest.mark.parametrize("value", [2.0, -1.0, 0.5, float("nan"), float("inf")])
def test_binary_conversion_rejects_invalid_data(value):
    for data in (np.array([[value, 0.0, 1.0]]), sparse.csr_matrix([[value, 0.0, 1.0]])):
        with pytest.raises(ValueError, match="finite|0/1"):
            _ensure_csr_binary(data)
        assert not sparse_binary_domain(3).member(data)


def test_sparse_domain_rejects_duplicate_or_weighted_values():
    duplicate = sparse.csr_matrix(
        (np.array([1.0, 1.0]), np.array([0, 0]), np.array([0, 2])), shape=(1, 3)
    )
    with pytest.raises(ValueError, match="0/1"):
        _ensure_csr_binary(duplicate)
    assert not sparse_binary_domain(3).member(sparse.csr_matrix([[2.0, 0.0, 0.0]]))


# --------------------------------------------------------------------------
# sklearn estimator + Context bridge
# --------------------------------------------------------------------------
def test_estimator_has_flat_sklearn_parameters():
    from sklearn.base import BaseEstimator, clone

    assert issubclass(SphericalKMeans, _DPEstimator)
    assert issubclass(SphericalKMeans, BaseEstimator)
    est = SphericalKMeans(2, **_TINY_PARAMS)
    assert est.get_params(deep=False) == {"n_clusters": 2, **_TINY_PARAMS}
    cloned = clone(est)
    assert cloned.get_params(deep=False) == est.get_params(deep=False)
    est.set_params(max_iter=2, max_features_per_record=2)
    assert est.max_iter == 2
    assert est.max_features_per_record == 2
    assert not hasattr(est, "config")


def test_removed_public_api_is_not_exported():
    import opendp.extras.sklearn.cluster as cluster

    assert set(cluster.__all__) == {
        "SphericalKMeans",
        "sparse_binary_domain",
        "make_private_spherical_kmeans",
        "then_private_spherical_kmeans",
    }
    for name in (
        "SphericalKMeansConfig",
        "SphericalKMeansRelease",
        "nearest_center_labels",
        "make_cluster_feature_sums",
        "then_cluster_feature_sums",
    ):
        assert not hasattr(cluster, name)


def test_sklearn_estimator_is_abstract():
    with pytest.raises(TypeError, match="abstract"):
        _DPEstimator()


def _context(x, rho=0.5, split=None):
    return dp.Context.compositor(
        data=x,
        privacy_unit=(dp.symmetric_distance(), 1),
        privacy_loss=(dp.zero_concentrated_divergence(), rho),
        domain=sparse_binary_domain(x.shape[1]),
        split_evenly_over=split,
    )


def test_context_requires_explicit_sparse_binary_domain():
    x = _two_blob_data()
    with pytest.raises(UnknownTypeException):
        dp.Context.compositor(
            data=x,
            privacy_unit=(dp.symmetric_distance(), 1),
            privacy_loss=(dp.zero_concentrated_divergence(), 0.5),
            split_evenly_over=1,
        )


def test_estimator_make_does_not_mutate_state():
    est = SphericalKMeans(2, **_TINY_PARAMS)
    state = est.__dict__.copy()
    measurement = est.make(
        sparse_binary_domain(6),
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        1,
        0.5,
    )
    assert isinstance(measurement, dp.Measurement)
    assert est.__dict__ == state


def test_context_fit_release_and_postprocessing():
    x = _two_blob_data()
    ctx = _context(x, rho=0.5, split=2)
    est = SphericalKMeans(2, **_TINY_PARAMS)
    assert est.fit(ctx.query(), y=object()) is est
    assert sparse.issparse(est.cluster_centers_)
    assert est.cluster_centers_.shape == (2, 6)
    assert est.n_features_in_ == 6
    assert not hasattr(est, "config_")
    assert not hasattr(est, "n_clusters_")
    assert not hasattr(est, "n_iter_")
    assert np.asarray(est.predict(x)).shape == (x.shape[0],)
    assert est.transform(x).shape == (x.shape[0], 2)
    assert est.make_transform()(x).shape == est.transform(x).shape
    assert np.array_equal(est.make_predict()(x), est.predict(x))
    assert np.isfinite(est.score(x))


def test_fitted_centers_remain_authoritative_after_set_params():
    x = _two_blob_data()
    est = SphericalKMeans(2, **_TINY_PARAMS)
    est.fit(_context(x, split=1).query())
    before = est.transform(x)
    est.set_params(n_clusters=3, max_iter=1, max_features_per_center=1)
    assert est.n_clusters == 3
    assert est.transform(x).shape == (x.shape[0], 2)
    assert np.allclose(est.transform(x), before)
    assert est.make_transform().output_domain.descriptor.num_columns == 2


def test_fit_requires_query_and_rejects_unknown_metadata():
    est = SphericalKMeans(2, **_TINY_PARAMS)
    with pytest.raises(TypeError, match="X to be a Query"):
        est.fit(_two_blob_data())
    with pytest.raises(TypeError, match="Unexpected fit parameters: sample_weight"):
        est.fit(_context(_two_blob_data(), split=1).query(), sample_weight=object())


def test_cluster_sizes_are_a_lower_level_measurement_and_context_wrapper():
    x = _two_blob_data(reps=100)
    ctx = _context(x, rho=1.0, split=2)
    est = SphericalKMeans(2, **_TINY_PARAMS)
    est.fit(ctx.query())
    sizes = np.asarray(est.release_cluster_sizes(ctx.query()))
    assert sizes.shape == (2,)
    assert abs(int(sizes.sum()) - 600) < 60
    assert not hasattr(est, "cluster_sizes")
    assert not hasattr(est, "silhouette")
    assert not hasattr(est, "release_silhouette")

    measurement = make_private_cluster_sizes(
        sparse_binary_domain(6),
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        1,
        0.5,
        centers=est.cluster_centers_,
    )
    assert measurement.map(1) <= 0.5
    assert measurement.map(2) <= 2.0
    assert np.asarray(measurement(x)).shape == (2,)
    with pytest.raises(ValueError, match="columns"):
        make_private_cluster_sizes(
            sparse_binary_domain(5),
            dp.symmetric_distance(),
            dp.zero_concentrated_divergence(),
            1,
            0.5,
            centers=est.cluster_centers_,
        )


def test_recovers_two_blobs_with_enough_budget():
    x = _two_blob_data(reps=200)
    est = SphericalKMeans(2, **_TINY_PARAMS)
    est.fit(_context(x, rho=50.0, split=1).query())
    labels = np.asarray(est.predict(_two_blob_data()))
    assert len(set(labels[:3])) == 1
    assert len(set(labels[3:])) == 1
    assert labels[0] != labels[3]
