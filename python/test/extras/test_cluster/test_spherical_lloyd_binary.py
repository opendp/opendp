from __future__ import annotations

import math

import pytest

import opendp.prelude as dp
from opendp.extras.sklearn import DPEstimator
from opendp.extras.sklearn.cluster import (
    SphericalKMeans,
    SphericalKMeansConfig,
    sparse_binary_domain,
    make_cluster_feature_sums,
    make_private_spherical_kmeans,
    nearest_center_labels,
)
from opendp.extras.sklearn.cluster._spherical_lloyd_binary import (
    _center_distances,
    _check_zcdp_budget,
    _clip_rows,
    _ensure_csr_binary,
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
        domain,
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        1,
        0.5,
        n_clusters=2,
        config=_TINY_CFG,
    )
    assert m.map(1) <= 0.5
    assert m.map(1) == pytest.approx(0.5, rel=1e-9)

    estimator = SphericalKMeans(n_clusters=2, config=_TINY_CFG)
    measurement = estimator.make(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(), 1, 0.5
    )
    assert measurement.map(1) <= 0.5
    # the release is just the centers matrix (no wrapper duplicating inputs)
    centers = m(x)
    assert sparse.issparse(centers)
    assert centers.shape == (2, 6)


def test_reads_n_features_from_domain():
    domain = sparse_binary_domain(6)
    m = make_private_spherical_kmeans(
        domain,
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        1,
        0.5,
        n_clusters=2,
        config=_TINY_CFG,
    )
    assert m(_two_blob_data()).shape[1] == 6


def test_group_privacy_scales_with_d_in():
    domain = sparse_binary_domain(6)
    m1 = make_private_spherical_kmeans(
        domain,
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        1,
        0.5,
        n_clusters=2,
        config=_TINY_CFG,
    )
    m2 = make_private_spherical_kmeans(
        domain,
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        2,
        0.5,
        n_clusters=2,
        config=_TINY_CFG,
    )
    assert m1.map(1) <= 0.5
    assert m2.map(2) <= 0.5
    assert m2.map(1) <= m2.map(2)


def test_requires_symmetric_distance():
    domain = sparse_binary_domain(6)
    with pytest.raises(ValueError, match="add/remove"):
        make_private_spherical_kmeans(
            domain,
            dp.l1_distance(T=int),
            dp.zero_concentrated_divergence(),
            1,
            0.5,
            n_clusters=2,
            config=_TINY_CFG,
        )


def test_requires_zcdp():
    domain = sparse_binary_domain(6)
    with pytest.raises(ValueError, match="zero_concentrated"):
        make_private_spherical_kmeans(
            domain,
            dp.symmetric_distance(),
            dp.max_divergence(),
            1,
            0.5,
            n_clusters=2,
            config=_TINY_CFG,
        )


@pytest.mark.parametrize(
    ("d_in", "error"),
    [(1.5, TypeError), (True, ValueError), (0, ValueError)],
)
def test_rejects_invalid_privacy_distance(d_in, error):
    with pytest.raises(error, match="positive integer"):
        _check_zcdp_budget(
            dp.zero_concentrated_divergence(), d_in, 0.5
        )


@pytest.mark.parametrize("d_out", [float("nan"), float("inf"), 0.0])
def test_rejects_invalid_privacy_budget(d_out):
    with pytest.raises(ValueError, match="finite and positive"):
        _check_zcdp_budget(
            dp.zero_concentrated_divergence(), 1, d_out
        )


def test_rejects_unknown_distance():
    domain = sparse_binary_domain(6)
    config = SphericalKMeansConfig(distance="typo")  # type: ignore[arg-type]
    with pytest.raises(ValueError, match="distance must be"):
        make_private_spherical_kmeans(
            domain,
            dp.symmetric_distance(),
            dp.zero_concentrated_divergence(),
            1,
            0.5,
            n_clusters=2,
            config=config,
        )


def test_helpers_reject_unknown_distance():
    domain = sparse_binary_domain(3)
    x = sparse.csr_matrix([[1, 0, 1]], dtype=np.float32)
    centers = sparse.csr_matrix([[1, 0, 1]], dtype=np.float32)

    with pytest.raises(ValueError, match="distance must be"):
        _center_distances(x, centers, "typo")  # type: ignore[arg-type]
    with pytest.raises(ValueError, match="distance must be"):
        nearest_center_labels(x, centers, distance="typo")  # type: ignore[arg-type]
    with pytest.raises(ValueError, match="distance must be"):
        make_cluster_feature_sums(
            domain,
            dp.symmetric_distance(),
            centers=centers,
            max_active=1,
            distance="typo",  # type: ignore[arg-type]
        )


def test_cluster_feature_sums_stability_and_output_shape():
    domain = sparse_binary_domain(6)
    centers = sparse.csr_matrix(
        np.array([[1, 1, 1, 0, 0, 0], [0, 0, 0, 1, 1, 1]], dtype=np.float32)
    )
    t = make_cluster_feature_sums(
        domain,
        dp.symmetric_distance(),
        centers=centers,
        max_active=4,
        distance="cosine",
    )
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


def test_dense_binary_conversion_clipping_and_jaccard_assignment():
    dense = _ensure_csr_binary(np.array([1, 0, 1]), n_features=3)
    assert _ensure_csr_binary(np.array([1, 0])).shape == (2, 1)
    clipped = _clip_rows(dense, 1)
    assert clipped.nnz == 1
    assert nearest_center_labels(
        dense, sparse.csr_matrix([[1, 0, 1]], dtype=np.float32), distance="jaccard"
    ).tolist() == [0]


def test_binary_conversion_accepts_dense_and_sparse_binary_data():
    dense = np.array([[1, 0, 1], [0, 1, 0]], dtype=np.float32)
    sparse_data = sparse.csr_matrix(dense)
    assert np.array_equal(
        _ensure_csr_binary(dense).toarray(), _ensure_csr_binary(sparse_data).toarray()
    )
    domain = sparse_binary_domain(3)
    assert domain.member(dense)
    assert domain.member(sparse_data)
    assert dp.domain_of(sparse_data, infer=True) == domain


@pytest.mark.parametrize("value", [2.0, -1.0, 0.5, float("nan"), float("inf")])
def test_binary_conversion_rejects_invalid_dense_and_sparse_data(value):
    dense = np.array([[value, 0.0, 1.0]])
    sparse_data = sparse.csr_matrix(dense)
    for data in (dense, sparse_data):
        with pytest.raises(ValueError, match="finite|0/1"):
            _ensure_csr_binary(data)
        assert not sparse_binary_domain(3).member(data)


def test_binary_conversion_rejects_duplicate_sparse_values():
    duplicate = sparse.csr_matrix(
        (np.array([1.0, 1.0], dtype=np.float32), np.array([0, 0]), np.array([0, 2])),
        shape=(1, 3),
    )
    with pytest.raises(ValueError, match="0/1"):
        _ensure_csr_binary(duplicate)


def test_sparse_explicit_zeros_remain_inactive():
    x = sparse.csr_matrix(
        (np.array([0.0]), np.array([4]), np.array([0, 1])), shape=(1, 6)
    )
    domain = sparse_binary_domain(6)
    centers = sparse.csr_matrix([[1, 0, 0, 0, 0, 0]], dtype=np.float32)
    t = make_cluster_feature_sums(
        domain,
        dp.symmetric_distance(),
        centers=centers,
        max_active=1,
        distance="cosine",
    )
    assert domain.member(x)
    assert np.asarray(t(x)).sum() == 0


def test_domain_inference_from_csr():
    # dp.domain_of infers a sparse binary domain from a scipy sparse matrix,
    # reading n_features from its column count.
    x = _two_blob_data()
    inferred = dp.domain_of(x, infer=True)
    assert inferred == sparse_binary_domain(x.shape[1])
    assert inferred.descriptor.n_features == 6


def test_sparse_domain_rejects_nonbinary_values():
    domain = sparse_binary_domain(3)
    for value in (2.0, -1.0, 0.5):
        weighted = sparse.csr_matrix([[value, 0.0, 0.0]])
        assert not domain.member(weighted)
        with pytest.raises(ValueError, match="nonbinary"):
            dp.domain_of(weighted, infer=True)


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
    from sklearn.base import BaseEstimator, clone

    assert issubclass(SphericalKMeans, DPEstimator)
    assert issubclass(SphericalKMeans, BaseEstimator)
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG)
    assert est.n_clusters == 2
    assert not hasattr(est, "cluster_centers_")
    cloned = clone(est)
    assert cloned.n_clusters == est.n_clusters
    assert cloned.config == est.config


def test_sklearn_estimator_is_abstract():
    with pytest.raises(TypeError, match="abstract"):
        DPEstimator()


def test_estimator_fit_requires_context_query():
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG)
    with pytest.raises(TypeError, match="X to be a Query from an OpenDP Context"):
        est.fit(_two_blob_data())


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
    # predict/transform/score are ordinary sklearn methods over caller-held data.
    assert np.asarray(est.predict(x)).shape == (x.shape[0],)
    assert est.transform(x).shape == (x.shape[0], 2)
    assert est.make_transform()(x).shape == est.transform(x).shape
    assert np.array_equal(est.make_predict()(x), est.predict(x))
    assert np.isfinite(est.score(x))
    # no labels for the fitted (private) data are stored
    assert est.__dict__.get("labels_") is None


def test_fitted_configuration_is_used_after_set_params():
    x = _two_blob_data()
    ctx = _context(x, rho=0.5, split=2)
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG, random_state=1)
    est.fit(ctx.query())

    public = _two_blob_data(reps=1)
    before = est.transform(public)
    est.set_params(
        n_clusters=3,
        config=SphericalKMeansConfig(
            iterations=1,
            center_active=1,
            max_active=1,
            init_active=1,
            distance="hamming",
        ),
    )
    assert est.n_clusters == 3
    assert est.n_clusters_ == 2
    assert est.distance_ == _TINY_CFG.distance
    assert np.allclose(est.transform(public), before)
    assert est.make_transform().output_domain.descriptor.num_columns == 2


def test_estimator_fit_accepts_ignored_y():
    x = _two_blob_data()
    ctx = _context(x, rho=0.5, split=1)
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG, random_state=1)
    assert est.fit(ctx.query(), y=object()) is est


def test_estimator_fit_rejects_unsupported_metadata():
    x = _two_blob_data()
    ctx = _context(x, rho=0.5, split=1)
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG)
    with pytest.raises(TypeError, match="Unexpected fit parameters: sample_weight"):
        est.fit(ctx.query(), sample_weight=object())


def test_hamming_score_uses_feature_width_stability_bound():
    est = SphericalKMeans(
        n_clusters=1,
        config=SphericalKMeansConfig(distance="hamming"),
    )
    est.cluster_centers_ = sparse.csr_matrix([[1, 1, 1, 0, 0, 0]], dtype=np.float32)
    est.n_features_in_ = 6
    est.n_clusters_ = 1
    est.distance_ = "hamming"
    empty = sparse.csr_matrix((0, 6), dtype=np.float32)
    opposite = sparse.csr_matrix([[0, 0, 0, 1, 1, 1]], dtype=np.float32)
    assert est.make_score().map(1) == 6
    assert abs(est.score(opposite) - est.score(empty)) == 6


def test_sklearn_rejects_non_estimator():
    x = _two_blob_data()
    ctx = _context(x, rho=0.5, split=1)
    with pytest.raises(ValueError, match="DPEstimator"):
        ctx.query().sklearn(object())


def test_cluster_sizes_and_silhouette_methods():
    # sizes/silhouette are explicit private releases on their own budget.
    x = _two_blob_data(reps=100)  # 600 rows, ~300 per blob
    ctx = _context(x, rho=1.5, split=3)
    est = SphericalKMeans(n_clusters=2, config=_TINY_CFG, random_state=0)
    est.fit(ctx.query())

    sizes = np.asarray(est.cluster_sizes(ctx.query()))
    assert sizes.shape == (2,)
    assert abs(int(sizes.sum()) - 600) < 60  # noisy counts sum to ~n

    sil = est.release_silhouette(ctx.query())
    assert 0.0 <= float(sil) <= 1.0  # clamped to the valid silhouette range
    assert float(sil) > 0.7  # well-separated blobs score high


def test_silhouette_requires_two_clusters():
    x = _two_blob_data(reps=10)
    ctx = _context(x, rho=1.0, split=2)
    est = SphericalKMeans(
        n_clusters=1,
        config=SphericalKMeansConfig(
            iterations=2, center_active=3, max_active=3, init_active=3
        ),
        random_state=0,
    )
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
