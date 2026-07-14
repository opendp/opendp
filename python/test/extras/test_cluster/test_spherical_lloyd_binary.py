from __future__ import annotations

import math

import pytest

import opendp.prelude as dp
from opendp.extras.sklearn import SklearnEstimator
from opendp.extras.sklearn.cluster import (
    SphericalKMeans,
    SphericalKMeansConfig,
    sparse_binary_domain,
    make_cluster_feature_sums,
    make_private_spherical_kmeans,
    nearest_center_labels,
)

np = pytest.importorskip("numpy")
sparse = pytest.importorskip("scipy.sparse")

# The mechanism must work with only "contrib" -- it must NOT require honest-but-curious.
dp.enable_features("contrib")

_TINY_CFG = SphericalKMeansConfig(
    iterations=4, center_active=3, max_active=3, init_active=3
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


# --------------------------------------------------------------------------
# constructor convention + calibration
# --------------------------------------------------------------------------
def test_constructor_calibrates_map_below_d_out():
    x = _two_blob_data()
    domain = sparse_binary_domain(6)
    m = make_private_spherical_kmeans(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(),
        1, 0.5, n_clusters=2, config=_TINY_CFG)
    assert m.map(1) <= 0.5
    assert m.map(1) == pytest.approx(0.5, rel=1e-9)
    # the release is just the centers matrix (no wrapper duplicating inputs)
    centers = m(x)
    assert sparse.issparse(centers)
    assert centers.shape == (2, 6)


def test_reads_n_features_from_domain():
    domain = sparse_binary_domain(6)
    m = make_private_spherical_kmeans(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(),
        1, 0.5, n_clusters=2, config=_TINY_CFG)
    assert m(_two_blob_data()).shape[1] == 6


def test_group_privacy_scales_with_d_in():
    domain = sparse_binary_domain(6)
    m1 = make_private_spherical_kmeans(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(),
        1, 0.5, n_clusters=2, config=_TINY_CFG)
    m2 = make_private_spherical_kmeans(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(),
        2, 0.5, n_clusters=2, config=_TINY_CFG)
    assert m1.map(1) <= 0.5
    assert m2.map(2) <= 0.5
    assert m2.map(1) <= m2.map(2)


def test_requires_symmetric_distance():
    domain = sparse_binary_domain(6)
    with pytest.raises(ValueError, match="add/remove"):
        make_private_spherical_kmeans(
            domain, dp.l1_distance(T=int), dp.zero_concentrated_divergence(),
            1, 0.5, n_clusters=2, config=_TINY_CFG)


def test_requires_zcdp():
    domain = sparse_binary_domain(6)
    with pytest.raises(ValueError, match="zero_concentrated"):
        make_private_spherical_kmeans(
            domain, dp.symmetric_distance(), dp.max_divergence(),
            1, 0.5, n_clusters=2, config=_TINY_CFG)


def test_rejects_unknown_distance():
    domain = sparse_binary_domain(6)
    config = SphericalKMeansConfig(distance="typo")  # type: ignore[arg-type]
    with pytest.raises(ValueError, match="distance must be"):
        make_private_spherical_kmeans(
            domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(),
            1, 0.5, n_clusters=2, config=config)


def test_cluster_feature_sums_stability_and_output_shape():
    domain = sparse_binary_domain(6)
    centers = sparse.csr_matrix(
        np.array([[1, 1, 1, 0, 0, 0], [0, 0, 0, 1, 1, 1]], dtype=np.float32))
    t = make_cluster_feature_sums(
        domain, dp.symmetric_distance(), centers=centers, max_active=4, distance="cosine")
    assert t.map(1) == pytest.approx(math.sqrt(4))
    assert t.map(3) == pytest.approx(3 * math.sqrt(4))
    out = np.asarray(t(_two_blob_data()))
    assert out.shape == (2 * 6,)
    assert np.issubdtype(out.dtype, np.integer)
    # rows 0-2 assign to center 0 (features 0,1,2); their sums land in the first block
    assert out[:6].sum() > 0 and out[6:].sum() > 0


def test_domain_has_no_max_active():
    domain = sparse_binary_domain(6)
    assert not hasattr(domain.descriptor, "max_active")
    assert domain.descriptor.n_features == 6
    assert domain.descriptor.max_rows == 2**31 - 1


def test_sparse_explicit_zeros_remain_inactive():
    x = sparse.csr_matrix(
        (np.array([0.0]), np.array([4]), np.array([0, 1])), shape=(1, 6))
    domain = sparse_binary_domain(6)
    centers = sparse.csr_matrix([[1, 0, 0, 0, 0, 0]], dtype=np.float32)
    t = make_cluster_feature_sums(
        domain, dp.symmetric_distance(), centers=centers,
        max_active=1, distance="cosine")
    assert domain.member(x)
    assert np.asarray(t(x)).sum() == 0


def test_domain_inference_from_csr():
    # dp.domain_of infers a sparse binary domain from a scipy sparse matrix,
    # reading n_features from its column count.
    x = _two_blob_data()
    inferred = dp.domain_of(x, infer=True)
    assert inferred == sparse_binary_domain(x.shape[1])
    assert inferred.descriptor.n_features == 6


def test_context_infers_domain_from_csr():
    # Context.compositor with domain omitted now works for a CSR input.
    x = _two_blob_data()
    ctx = dp.Context.compositor(
        data=x,
        privacy_unit=(dp.symmetric_distance(), 1),
        privacy_loss=(dp.zero_concentrated_divergence(), 0.5),
        split_evenly_over=1,
    )
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG, random_state=1)
    centers = ctx.query().sklearn(est).release()
    assert centers.shape == (2, 6)


# --------------------------------------------------------------------------
# estimator + Context bridge
# --------------------------------------------------------------------------
def test_estimator_is_sklearn_estimator():
    assert issubclass(SphericalKMeans, SklearnEstimator)
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG)
    assert est.n_clusters == 2
    assert est.cluster_centers_ is None


def _context(x, rho=0.5, split=None):
    return dp.Context.compositor(
        data=x,
        privacy_unit=(dp.symmetric_distance(), 1),
        privacy_loss=(dp.zero_concentrated_divergence(), rho),
        domain=sparse_binary_domain(x.shape[1]),
        split_evenly_over=split,
    )


def test_context_query_sklearn_release():
    x = _two_blob_data()
    ctx = _context(x, rho=0.5, split=2)
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG, random_state=1)
    centers = ctx.query().sklearn(est).release()
    assert sparse.issparse(centers) and centers.shape == (2, 6)


def test_context_query_with_rho_kwarg():
    x = _two_blob_data()
    ctx = _context(x, rho=0.5)  # filter; allocate per query
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG, random_state=1)
    centers = ctx.query(rho=0.3).sklearn(est).release()
    assert centers.shape == (2, 6)


def test_estimator_fit_query_and_predict():
    x = _two_blob_data()
    ctx = _context(x, rho=0.5, split=2)
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG, random_state=1)
    fitted = est.fit(ctx.query())
    assert fitted is est
    assert est.cluster_centers_.shape == (2, 6)
    assert est.n_features_in_ == 6
    assert est.n_iter_ == 4
    # predict/transform/score are postprocessing on caller-held data; each is a
    # property returning a Transformation, so calling it applies the transformation.
    assert np.asarray(est.predict(x)).shape == (x.shape[0],)
    assert est.transform(x).shape == (x.shape[0], 2)
    assert np.isfinite(est.score(x))
    # no labels for the fitted (private) data are stored
    assert est.__dict__.get("labels_") is None


def test_hamming_score_uses_feature_width_stability_bound():
    est = SphericalKMeans(
        n_clusters=1,
        config=SphericalKMeansConfig(distance="hamming"),
    )
    est.cluster_centers_ = sparse.csr_matrix(
        [[1, 1, 1, 0, 0, 0]], dtype=np.float32)
    est.n_features_in_ = 6
    empty = sparse.csr_matrix((0, 6), dtype=np.float32)
    opposite = sparse.csr_matrix([[0, 0, 0, 1, 1, 1]], dtype=np.float32)
    assert est.score.map(1) == 6
    assert abs(est.score(opposite) - est.score(empty)) == 6


def test_sklearn_rejects_non_estimator():
    x = _two_blob_data()
    ctx = _context(x, rho=0.5, split=1)
    with pytest.raises(ValueError, match="SklearnEstimator"):
        ctx.query().sklearn(object())


def test_cluster_sizes_and_silhouette_methods():
    # sizes/silhouette are methods on the fitted model, released on their own budget,
    # composed over est.predict / est.transform.
    x = _two_blob_data(reps=100)  # 600 rows, ~300 per blob
    ctx = _context(x, rho=1.5, split=3)
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG, random_state=0)
    est.fit(ctx.query())

    sizes = np.asarray(est.cluster_sizes(ctx.query()))
    assert sizes.shape == (2,)
    assert abs(int(sizes.sum()) - 600) < 60  # noisy counts sum to ~n

    sil = est.silhouette(ctx.query())
    assert 0.0 <= float(sil) <= 1.0          # clamped to the valid silhouette range
    assert float(sil) > 0.7                  # well-separated blobs score high


def test_silhouette_requires_two_clusters():
    x = _two_blob_data(reps=10)
    ctx = _context(x, rho=1.0, split=2)
    est = SphericalKMeans(n_clusters=1, config=SphericalKMeansConfig(
        iterations=2, center_active=3, max_active=3, init_active=3), random_state=0)
    est.fit(ctx.query())
    with pytest.raises(ValueError, match="at least 2 clusters"):
        est.silhouette(ctx.query())


def test_recovers_two_blobs_with_enough_budget():
    x = _two_blob_data(reps=200)  # 1200 rows
    ctx = _context(x, rho=50.0, split=1)
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG, random_state=0)
    est.fit(ctx.query())
    labels = np.asarray(est.predict(_two_blob_data(reps=1)))
    assert len(set(labels[:3])) == 1
    assert len(set(labels[3:])) == 1
    assert labels[0] != labels[3]
