"""Exact arithmetic, privacy accounting, and sklearn integration of spherical Lloyd."""

from fractions import Fraction
import math
import pickle
from typing import Any

import pytest

np = pytest.importorskip("numpy")
sp = pytest.importorskip("scipy.sparse")
pytest.importorskip("sklearn")

import opendp.prelude as dp
from opendp.extras.scipy import csr_array2_domain
from opendp.extras.sklearn.cluster import SphericalKMeans, make_private_spherical_kmeans
from opendp.extras.sklearn.cluster import _make_private_spherical_kmeans as skm


dp.enable_features("contrib")
METRIC = dp.symmetric_distance()
PARAMS: dict[str, Any] = dict(
    n_clusters=2,
    max_iter=3,
    max_features_per_center=None,
    max_l1_contribution=None,
    init="random",
    random_state=0,
)


def data():
    return sp.csr_array(
        [[1, 1, 0, 0], [1, 0, 0, 0], [0, 0, 1, 1], [0, 0, 0, 1], [0, 0, 0, 0]],
        dtype=bool,
    )


def domain(d=4):
    return csr_array2_domain(d, max_rows=None, T=bool)


def centers():
    return sp.csr_array([[1, 1, 0, 0], [0, 0, 1, 1]], dtype=np.int64)


def measure(kind):
    return (
        dp.max_divergence() if kind == "epsilon" else dp.zero_concentrated_divergence()
    )


def mechanism(kind="rho", distance=1, budget=1.0, **kwargs):
    return make_private_spherical_kmeans(
        domain(), METRIC, measure(kind), distance, budget, **(PARAMS | kwargs)
    )


@pytest.mark.parametrize("cap", [None, Fraction(1, 2), Fraction(3, 2)])
def test_weights_are_exact_conservative_and_treat_all_features_equally(cap):
    q = skm._quantization(4, cap)
    x = data()
    weights = skm._row_weights(x, q, cap)
    # Verify floor(Q/sqrt(r)) with integer inequalities, not floating tolerances.
    for r, w in zip(map(int, np.diff(x.indptr)), map(int, weights)):
        if r == 0:
            assert w == 0
            continue
        expected = math.isqrt(q * q // r)
        if cap is not None:
            expected = min(expected, q * cap.numerator // (cap.denominator * r))
            assert r * w * cap.denominator <= q * cap.numerator
        assert w == expected and r * w * w <= q * q
    labels = skm._labels(x, skm._unit_centers(centers()))
    actual = skm._aggregate(x, labels, weights, 2).reshape(2, 4)
    a, b = map(int, weights[:2])
    np.testing.assert_array_equal(actual, [[a + b, a, 0, 0], [0, 0, a, a + b]])
    # Column permutation, including moving informative features to high IDs,
    # cannot change how much each row contributes.
    permuted = x[:, ::-1].tocsr()
    permuted.sort_indices()
    np.testing.assert_array_equal(skm._row_weights(permuted, q, cap), weights)
    np.testing.assert_array_equal(
        skm._aggregate(permuted, labels, weights, 2).reshape(2, 4), actual[:, ::-1]
    )


@pytest.mark.parametrize(
    "d,cap", [(10**12, None), (10**9, Fraction(1, 10)), (2**62, None)]
)
def test_large_public_universe_has_nonzero_accurate_weights_without_allocating_it(
    d, cap
):
    q = skm._quantization(d, cap)
    w = (
        min(math.isqrt(q * q // d), q * cap.numerator // (cap.denominator * d))
        if cap
        else math.isqrt(q * q // d)
    )
    assert 2**24 <= w <= skm._MAX and q <= skm._MAX
    # Actual sparse input in a trillion-dimensional universe is also supported.
    x = sp.csr_array((np.ones(2, bool), [0, d - 1], [0, 2]), shape=(1, d))
    assert skm._row_weights(x, q, cap)[0] > 0


def test_impossible_precision_is_rejected_before_data_access():
    with pytest.raises(ValueError, match="fixed-point precision"):
        skm._quantization(10**12, Fraction(1, 10**12))


def test_sparse_aggregation_fast_path_and_saturation_before_overflow():
    x = sp.csr_array([[1, 1], [1, 0], [1, 1]], dtype=bool)
    labels = np.array([0, 0, 0])
    for weights in [
        np.array([3, 4, 5], np.int64),
        np.array([skm._MAX // 2 + 1] * 3, np.int64),
        np.array([skm._MAX, 1, 0], np.int64),
    ]:
        expected = [
            min(skm._MAX, sum(int(weights[i]) for i in range(3) if x[i, j]))
            for j in range(2)
        ]
        np.testing.assert_array_equal(skm._aggregate(x, labels, weights, 1), expected)
    empty = sp.csr_array((0, 2), dtype=bool)
    np.testing.assert_array_equal(
        skm._aggregate(empty, labels[:0], np.array([], np.int64), 2), np.zeros(4)
    )
    zero = sp.csr_array((3, 2), dtype=bool)
    np.testing.assert_array_equal(
        skm._aggregate(zero, labels, np.zeros(3, np.int64), 1), [0, 0]
    )


@pytest.mark.parametrize("cap", [None, Fraction(1, 2), Fraction(3, 2)])
@pytest.mark.parametrize("norm", [1, 2])
def test_fixed_point_sensitivity_and_saturation_are_nonexpansive(cap, norm):
    d = 17
    q = skm._quantization(d, cap)
    transform = skm._make_sums(
        domain(d), METRIC, sp.csr_array(np.ones((2, d))), q, cap, norm
    )
    exact_bound = q if norm == 2 else math.isqrt(q * q * d)
    if cap:
        exact_bound = min(exact_bound, q * cap.numerator // cap.denominator)
    for r in range(1, d + 1):
        x = sp.csr_array(np.arange(d)[None, :] < r)
        vector = np.asarray(transform(x), dtype=np.int64)
        l1 = sum(map(int, vector))
        l2_squared = sum(int(v) ** 2 for v in vector)
        bound = transform.map(1)
        assert l1 <= bound if norm == 1 else l2_squared <= int(bound) ** 2
        assert l2_squared <= q * q
        if cap:
            assert l1 * cap.denominator <= q * cap.numerator
        assert transform.map(3) >= 3 * exact_bound
    a = [skm._MAX - 2, skm._MAX, 3]
    b = [skm._MAX + 10, skm._MAX - 1, 8]
    before = [abs(x - y) for x, y in zip(a, b)]
    after = [abs(min(x, skm._MAX) - min(y, skm._MAX)) for x, y in zip(a, b)]
    assert sum(after) <= sum(before) and sum(v * v for v in after) <= sum(
        v * v for v in before
    )


def test_integer_bounds_are_rounded_up_before_entering_float_metrics():
    value = 2**60 + 1
    assert skm._upper_float(value) >= value
    assert skm._upper_float(2**60) == 2**60


def test_cosine_assignment_extreme_public_directions_and_deterministic_projection():
    x = sp.csr_array([[1, 0], [0, 1], [0, 0]], dtype=bool)
    c = sp.csr_array([[1e308, 1e308], [1e-300, 0], [0, 0]])
    np.testing.assert_array_equal(skm._labels(x, skm._unit_centers(c)), [1, 0, 0])
    distances = skm._distances(x, c)
    np.testing.assert_allclose(distances[0], [1 - 1 / math.sqrt(2), 0, 1])
    np.testing.assert_array_equal(distances[2], [1, 1, 1])
    projected = skm._project([-1, 9, 9, 9, 0, 0, 0, 0], 2, 4, 2)
    np.testing.assert_array_equal(projected.toarray(), [[0, 9, 9, 0], [0, 0, 0, 0]])
    assert projected.dtype == np.int64 and projected.has_canonical_format
    np.testing.assert_array_equal(
        skm._project([-1, 9, 9, 9], 1, 4, 4).toarray(), [[0, 9, 9, 9]]
    )


def test_initialization_covers_all_feature_ids_and_snapshots_public_directions():
    c = skm._initialize(3, 10000, "random", 42)
    assert c.nnz == 10000
    np.testing.assert_array_equal(np.asarray(c.sum(axis=0)).ravel(), np.ones(10000))
    np.testing.assert_array_equal(
        c.toarray(), skm._initialize(3, 10000, "random", 42).toarray()
    )
    public = centers()
    snapshot = skm._initialize(2, 4, public, 0)
    public.data[:] = 99
    np.testing.assert_array_equal(snapshot.toarray(), centers().toarray())
    np.testing.assert_array_equal(
        skm._initialize(2, 4, centers().toarray(), None).toarray(), centers().toarray()
    )
    # Converting duplicate integer coordinates must not wrap before float conversion.
    coo = sp.coo_array((np.array([skm._MAX, skm._MAX]), ([0, 0], [0, 0])), shape=(1, 1))
    assert skm._initialize(1, 1, coo, None)[0, 0] > skm._MAX
    with (
        pytest.raises(ValueError, match="overflowing duplicate"),
        np.errstate(over="ignore"),
    ):
        skm._initialize(
            1,
            1,
            sp.coo_array(([1e308, 1e308], ([0, 0], [0, 0])), shape=(1, 1)),
            None,
        )


@pytest.mark.parametrize("kind", ["epsilon", "rho"])
@pytest.mark.parametrize("distance", [1, 3])
@pytest.mark.parametrize("iterations", [1, 3])
def test_native_int64_noise_and_adaptive_group_accounting(
    kind, distance, iterations, monkeypatch
):
    monkeypatch.setattr("opendp.mod.GLOBAL_FEATURES", {"contrib"})
    calls = []
    original = skm._aggregate

    def aggregate(*args):
        calls.append(1)
        return original(*args)

    monkeypatch.setattr(skm, "_aggregate", aggregate)
    m = mechanism(
        kind,
        distance,
        max_iter=iterations,
        max_l1_contribution=1.5,
        max_features_per_center=2,
    )
    assert m.map(distance) <= 1.0 and m.map(distance) == pytest.approx(1.0)
    with pytest.raises(dp.OpenDPException, match="d_in.*no greater"):
        m.map(distance + 1)
    release = m(data())
    assert len(calls) == iterations
    assert release.dtype == np.int64 and release.shape == (2, 4)
    assert (release.data > 0).all() and (np.diff(release.indptr) <= 2).all()


@pytest.mark.parametrize("route", ["standalone", "context"])
@pytest.mark.parametrize("empty", [False, True])
def test_fitting_clone_aligned_targets_detached_buffers_and_minimal_state(route, empty):
    from sklearn.base import clone

    x = sp.csr_array((0, 4), dtype=bool) if empty else data()
    y = np.arange(x.shape[0], dtype=np.int32)
    model = SphericalKMeans(**PARAMS)
    assert clone(model).get_params() == PARAMS
    if route == "standalone":
        fitted = model.fit(
            x,
            y,
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(rho=1.0),
        )
        assert fitted is model
    else:
        ctx = dp.Context.compositor(
            dp.sklearn.SklearnFit(x, y),
            dp.unit_of(contributions=1),
            dp.loss_of(rho=1.0),
            split_evenly_over=1,
        )
        query = ctx.query().sklearn(model)
        # Inspect the estimator before generic FFI result metadata is attached,
        # as in the shared estimator tests (the metadata is not learned state).
        fitted = pickle.loads(
            query.new_with(chain=query.resolve() >> pickle.dumps).release()
        )
        assert fitted is not model and vars(model) == PARAMS
        assert ctx.current_privacy_loss() == [1.0]
    assert set(vars(fitted)) == set(PARAMS) | {"cluster_centers_", "n_features_in_"}
    assert "__opendp_dict__" not in vars(fitted.cluster_centers_)
    assert fitted.transform(x).shape == (x.shape[0], 2)
    np.testing.assert_array_equal(fitted.predict(x), fitted.transform(x).argmin(axis=1))
    assert math.isfinite(fitted.score(x))
    restored = pickle.loads(pickle.dumps(fitted))
    np.testing.assert_array_equal(restored.predict(x), fitted.predict(x))
    if empty:
        assert fitted.score(x) == 0


def test_configuration_and_postprocessors_snapshot_without_changing_base_fit_transform(
    monkeypatch,
):
    assignments = []
    original = skm._aggregate

    def aggregate(x, labels, *args):
        assignments.append(labels.copy())
        return original(x, labels, *args)

    monkeypatch.setattr(skm, "_aggregate", aggregate)
    public = centers()
    est = SphericalKMeans(**(PARAMS | {"init": public}))
    fit_domain = dp.sklearn.sklearn_fit_domain(
        domain(), dp.vector_domain(dp.atom_domain(T=dp.i32))
    )
    args = (fit_domain, METRIC, measure("rho"), 1, 1.0)
    m = est.make(*args)
    old_params = est.get_params()
    assert est.init is public and not hasattr(est, "cluster_centers_")
    public.data[:] = 0
    est.set_params(n_clusters=3, max_iter=1)
    release = m(dp.sklearn.SklearnFit(data(), np.arange(5, dtype=np.int32)))
    est._ingest_release(release)
    assert est.cluster_centers_.shape == (2, 4)  # params were snapshotted
    np.testing.assert_array_equal(assignments[0], [0, 0, 1, 1, 0])
    for name in ("data", "indices", "indptr"):
        assert not np.shares_memory(
            getattr(release, name), getattr(est.cluster_centers_, name)
        )
    carrier = dp.sklearn.SklearnFit(data(), np.arange(5, dtype=np.int32))
    for make, infer in [
        (est.make_transform, est.transform),
        (est.make_predict, est.predict),
    ]:
        t = make(fit_domain, METRIC)
        result = t(carrier)
        assert t.map(3) == 3 and t.output_domain.member(result)
        assert result.y is carrier.y
        np.testing.assert_array_equal(result.X, infer(carrier.X))
        snapshot = result.X.copy()
        saved = est.cluster_centers_.copy()
        est.cluster_centers_.data[:] = 0
        np.testing.assert_array_equal(t(carrier).X, snapshot)
        est._ingest_release(saved)
    release.data[:] = 0
    assert est.get_params()["n_clusters"] == 3 and old_params["n_clusters"] == 2


@pytest.mark.parametrize(
    "params,message",
    [
        ({"n_clusters": 0}, "positive integer"),
        ({"max_iter": True}, "positive integer"),
        ({"max_features_per_center": 0}, "positive integer"),
        ({"max_l1_contribution": float("nan")}, "finite and positive"),
        ({"max_l1_contribution": -1}, "finite and positive"),
        ({"max_l1_contribution": True}, "finite and positive"),
        ({"init": "private"}, "init must"),
        ({"init": np.ones((1, 4))}, "shape"),
        ({"init": [[-1, 0, 0, 0], [0, 0, 1, 0]]}, "nonnegative"),
        ({"init": np.full((2, 4), np.inf)}, "finite"),
    ],
)
def test_invalid_public_configuration(params, message):
    with pytest.raises(ValueError, match=message):
        mechanism(**params)


def test_invalid_spaces_and_unfitted_postprocessors():
    with pytest.raises(ValueError, match="add/remove"):
        make_private_spherical_kmeans(
            domain(), dp.change_one_distance(), measure("rho"), 1, 1.0
        )
    with pytest.raises(ValueError, match="T=bool"):
        make_private_spherical_kmeans(
            csr_array2_domain(4, None, T=float), METRIC, measure("rho"), 1, 1.0
        )
    with pytest.raises(ValueError, match="zero_concentrated"):
        make_private_spherical_kmeans(
            domain(), METRIC, dp.fixed_smoothed_max_divergence(), 1, 1.0
        )
    for budget in [0.0, -1.0, float("inf"), True]:
        with pytest.raises(ValueError, match="d_out must"):
            mechanism(budget=budget)
    with pytest.raises(ValueError, match="positive iteration budgets"):
        mechanism(budget=math.nextafter(0.0, 1.0), max_iter=5)
    model = SphericalKMeans(**PARAMS)
    with pytest.raises(ValueError, match="not been fitted"):
        model.predict(data())
    model._ingest_release(centers())
    with pytest.raises(ValueError, match="number of features"):
        model.make_transform(dp.sklearn.sklearn_fit_domain(domain(5)), METRIC)


@pytest.mark.parametrize("kind", ["epsilon", "rho"])
def test_separated_public_directions_have_useful_private_updates(kind):
    x = sp.vstack([data()] * 1000, format="csr")
    model = SphericalKMeans(**(PARAMS | {"init": centers()}))
    model._ingest_release(mechanism(kind, init=centers())(x))
    np.testing.assert_array_equal(model.predict(data()), [0, 0, 1, 1, 0])


def test_public_seed_does_not_repeat_dp_noise(monkeypatch):
    releases = []
    original = skm._project

    def project(noisy, *args):
        releases.append(np.asarray(noisy).copy())
        return original(noisy, *args)

    monkeypatch.setattr(skm, "_project", project)
    m = mechanism(random_state=7, max_iter=1)
    m(data())
    m(data())
    assert not np.array_equal(releases[0], releases[1])


def test_exports_and_removed_record_truncation():
    import opendp.extras.sklearn.cluster as cluster

    assert set(cluster.__all__) == {
        "SphericalKMeans",
        "make_private_spherical_kmeans",
        "then_private_spherical_kmeans",
    }
    assert "max_features_per_record" not in SphericalKMeans().get_params()
    assert set(cluster.__all__) <= set(dir(cluster))
    # Then-constructor remains a framework-level entrypoint.
    assert (domain(), METRIC) >> cluster.then_private_spherical_kmeans(
        measure("rho"), 1, 1.0, **PARAMS
    )
