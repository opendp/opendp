"""Normalization, safe integer sums, privacy accounting and estimator integration."""

from fractions import Fraction
from functools import partial
import inspect
import math

import pytest

np = pytest.importorskip("numpy")
sp = pytest.importorskip("scipy.sparse")
pytest.importorskip("sklearn")

import opendp.prelude as dp
from opendp.extras.scipy import csr_array2_domain
from opendp.extras.sklearn.cluster import SphericalKMeans, make_private_spherical_kmeans
from opendp.extras.sklearn.cluster import _make_private_spherical_kmeans as skm
from opendp.extras.sklearn.cluster import _spherical_kmeans as estimator


dp.enable_features("contrib")
METRIC = dp.symmetric_distance()
PARAMS = dict(n_clusters=2, max_iter=3, random_state=0)


def data():
    return sp.csr_array(
        [[1, 1, 0, 0], [1, 0, 0, 0], [0, 0, 1, 1], [0, 0, 0, 1], [0, 0, 0, 0]],
        dtype=bool,
    )


def centers():
    return sp.csr_array([[1, 1, 0, 0], [0, 0, 1, 1]], dtype=np.int32)


def mechanism(kind="rho", **kwargs):
    measure = (
        dp.max_divergence() if kind == "epsilon" else dp.zero_concentrated_divergence()
    )
    return make_private_spherical_kmeans(
        csr_array2_domain(4, T=bool),
        METRIC,
        measure,
        3,
        0.5,
        T=dp.i32,
        **(PARAMS | kwargs),
    )


@pytest.fixture
def i32_estimator(monkeypatch):
    # Exercise estimator plumbing with the debug build's supported native dtype.
    monkeypatch.setattr(
        estimator,
        "make_private_spherical_kmeans",
        partial(make_private_spherical_kmeans, T=dp.i32),
    )


@pytest.mark.parametrize("dtype,q", [(np.int32, 2**10), (np.int64, 2**24)])
@pytest.mark.parametrize("cap", [None, Fraction(1, 2)])
def test_normalized_weights_and_hand_computed_sums(dtype, q, cap):
    x = data()
    weights = skm._row_weights(x, q, cap, np.dtype(dtype))
    for r, w in zip(map(int, np.diff(x.indptr)), map(int, weights)):
        expected = math.isqrt(q * q // r) if r else 0
        if cap and r:
            expected = min(expected, q * cap.numerator // (cap.denominator * r))
            assert r * w <= q * cap
        assert w == expected and r * w * w <= q * q
    a, b = map(int, weights[:2])
    labels = skm._labels(x, skm._unit_centers(centers()))
    sums = skm._aggregate(x, labels, weights, 2).reshape(2, 4)
    np.testing.assert_array_equal(sums, [[a + b, a, 0, 0], [0, 0, a, a + b]])
    np.testing.assert_array_equal(
        skm._aggregate(x[::-1], labels[::-1], weights[::-1], 2).reshape(2, 4), sums
    )


@pytest.mark.parametrize("dtype", [np.int32, np.int64])
def test_aggregation_fast_path_and_saturation_before_overflow(dtype):
    x = sp.csr_array([[1, 1], [1, 0], [1, 1]], dtype=bool)
    limit = int(np.iinfo(dtype).max)
    for values in [[3, 4, 5], [limit // 2 + 1] * 3, [limit, 1, 0]]:
        expected = [
            min(limit, sum(values[i] for i in range(3) if x[i, j])) for j in range(2)
        ]
        actual = skm._aggregate(x, np.zeros(3, int), np.array(values, dtype=dtype), 1)
        np.testing.assert_array_equal(actual, expected)
        assert actual.dtype == dtype
    empty = sp.csr_array((0, 2), dtype=bool)
    np.testing.assert_array_equal(
        skm._aggregate(empty, np.array([], int), np.array([], dtype), 1), [0, 0]
    )
    np.testing.assert_array_equal(
        skm._aggregate(x, np.zeros(3, int), np.zeros(3, dtype), 1), [0, 0]
    )


@pytest.mark.parametrize("norm", [1, 2])
@pytest.mark.parametrize("cap", [None, Fraction(1, 2)])
def test_integer_sensitivity_and_upward_rounding(norm, cap):
    q, d = 1024, 17
    t = skm._make_sums(
        csr_array2_domain(d, T=bool),
        METRIC,
        sp.csr_array(np.ones((2, d))),
        q,
        cap,
        norm,
        np.dtype(np.int32),
    )
    bound = q if norm == 2 else math.isqrt(q * q * d)
    if cap:
        bound = min(bound, int(q * cap))
    for r in range(1, d + 1):
        values = np.asarray(t(sp.csr_array(np.arange(d)[None, :] < r)))
        actual = (
            sum(map(int, values)) if norm == 1 else sum(int(v) ** 2 for v in values)
        )
        assert actual <= (bound if norm == 1 else bound**2)
    assert t.map(3) >= 3 * bound
    assert skm._upper_float(2**60 + 1) >= 2**60 + 1
    assert skm._upper_float(2**60) == 2**60


def test_initialization_cosine_and_sparse_projection():
    public = centers()
    snapshot = skm._initialize(2, 4, public, 0)
    public.data[:] = 0
    np.testing.assert_array_equal(snapshot.toarray(), centers().toarray())
    partition = skm._initialize(3, 32, "random", 0)
    np.testing.assert_array_equal(
        partition.toarray(), skm._initialize(3, 32, "random", 0).toarray()
    )
    assert partition.nnz == 32
    c = sp.csr_array([[1e308, 1e308], [1e-300, 0], [0, 0]])
    x = sp.csr_array([[1, 0], [0, 0]], dtype=bool)
    np.testing.assert_array_equal(skm._labels(x, skm._unit_centers(c)), [1, 0])
    np.testing.assert_allclose(
        skm._distances(x, c), [[1 - 1 / math.sqrt(2), 0, 1], [1, 1, 1]]
    )
    out = skm._project([-1, 9, 9, 9], 1, 4, 2, np.dtype(np.int32))
    np.testing.assert_array_equal(out.toarray(), [[0, 9, 9, 0]])
    maximum = int(np.iinfo(np.int64).max)
    duplicate = sp.coo_array(([maximum, maximum], ([0, 0], [0, 0])), shape=(1, 1))
    assert skm._initialize(1, 1, duplicate, None)[0, 0] > maximum


@pytest.mark.parametrize("kind", ["epsilon", "rho"])
@pytest.mark.parametrize("iterations", [1, 3])
def test_i32_native_noise_group_distance_and_adaptive_iterations(
    kind, iterations, monkeypatch
):
    monkeypatch.setattr("opendp.mod.GLOBAL_FEATURES", {"contrib"})
    calls = []
    original = skm._aggregate

    def aggregate(x, labels, weights, k):
        calls.append(weights.dtype)
        return original(x, labels, weights, k)

    monkeypatch.setattr(skm, "_aggregate", aggregate)
    m = mechanism(
        kind, max_iter=iterations, max_l1_contribution=1.5, max_features_per_center=2
    )
    assert m.map(3) <= 0.5 and m.map(3) == pytest.approx(0.5)
    with pytest.raises(dp.OpenDPException, match="d_in.*no greater"):
        m.map(4)
    release = m(data())
    assert calls == [np.dtype(np.int32)] * iterations
    assert release.dtype == np.int32 and (np.diff(release.indptr) <= 2).all()
    assert mechanism(kind)(sp.csr_array((0, 4), dtype=bool)).shape == (2, 4)
    assert (
        inspect.signature(make_private_spherical_kmeans).parameters["T"].default
        == dp.i64
    )
    assert "T" not in SphericalKMeans().get_params()


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_fitting_and_cloning(route, i32_estimator):
    from sklearn.base import clone

    model = SphericalKMeans(**PARAMS)
    assert clone(model).get_params() == model.get_params()
    if route == "standalone":
        fitted = model.fit(
            data(),
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(rho=0.5),
        )
        assert fitted is model
    else:
        ctx = dp.Context.compositor(
            dp.sklearn.SklearnFit(data()),
            dp.unit_of(contributions=1),
            dp.loss_of(rho=0.5),
            split_evenly_over=1,
        )
        fitted = ctx.query().sklearn(model).release()
        assert fitted is not model and not hasattr(model, "cluster_centers_")
        assert ctx.current_privacy_loss() == [0.5]
    assert "__opendp_dict__" not in vars(fitted.cluster_centers_)
    np.testing.assert_array_equal(
        fitted.predict(data()), fitted.transform(data()).argmin(axis=1)
    )
    assert math.isfinite(fitted.score(data()))


def test_configuration_release_and_postprocessor_isolation(i32_estimator):
    public = centers()
    model = SphericalKMeans(**PARAMS, init=public)
    fit_domain = dp.sklearn.sklearn_fit_domain(
        csr_array2_domain(4, T=bool), dp.vector_domain(dp.atom_domain(T=int))
    )
    carrier = dp.sklearn.SklearnFit(data(), np.arange(5, dtype=np.int32))
    m = model.make(fit_domain, METRIC, dp.zero_concentrated_divergence(), 1, 0.5)
    public.data[:] = 0
    model.set_params(n_clusters=3, max_iter=1)
    release = m(carrier)
    model._ingest_release(release)
    assert model.cluster_centers_.shape == (2, 4)
    for name in ("data", "indices", "indptr"):
        assert not np.shares_memory(
            getattr(release, name), getattr(model.cluster_centers_, name)
        )
    for make, infer in [
        (model.make_transform, model.transform),
        (model.make_predict, model.predict),
    ]:
        t = make(fit_domain, METRIC)
        expected = infer(data())
        result = t(carrier)
        assert (
            t.map(3) == 3 and t.output_domain.member(result) and result.y is carrier.y
        )
        np.testing.assert_array_equal(result.X, expected)
        saved = model.cluster_centers_.copy()
        model.cluster_centers_.data[:] = 0
        np.testing.assert_array_equal(t(carrier).X, expected)
        model._ingest_release(saved)
    release.data[:] = 0
    np.testing.assert_array_equal(model.cluster_centers_.toarray(), saved.toarray())


@pytest.mark.parametrize(
    "invalid",
    [
        lambda: data().toarray(),
        lambda: data().astype(np.int32),
        lambda: data().tocsc(),
        lambda: sp.csr_matrix(data()),
        lambda: sp.csr_array((1, 5), dtype=bool),
        lambda: sp.csr_array([True, False]),
        lambda: sp.csr_array(([True, True], [0, 0], [0, 2]), shape=(1, 4)),
        lambda: sp.csr_array(([False], [0], [0, 1]), shape=(1, 4)),
    ],
)
def test_direct_and_framework_inference_reject_invalid_inputs(invalid):
    model = SphericalKMeans(**PARAMS)
    model._ingest_release(centers())
    x = invalid()
    for name in ["predict", "transform", "score"]:
        with pytest.raises(ValueError, match="canonical boolean CSR"):
            getattr(model, name)(x)
    fit_domain = dp.sklearn.sklearn_fit_domain(csr_array2_domain(4, T=bool))
    for make in [model.make_predict, model.make_transform]:
        with pytest.raises(dp.OpenDPException, match="canonical boolean CSR"):
            make(fit_domain, METRIC)(dp.sklearn.SklearnFit(x))


@pytest.mark.parametrize(
    "params",
    [
        dict(n_clusters=0),
        dict(max_iter=True),
        dict(max_l1_contribution=True),
        dict(max_l1_contribution=float("nan")),
        dict(max_l1_contribution=-1),
        dict(init="private"),
        dict(init=np.ones((1, 4))),
        dict(init=np.full((2, 4), -1)),
        dict(init=np.full((2, 4), np.inf)),
        dict(init=np.ones((2, 4), dtype=complex)),
        dict(max_l1_contribution=1e-12),
    ],
)
def test_invalid_configuration(params):
    with pytest.raises(ValueError):
        mechanism(**params)


def test_invalid_spaces_precision_and_unfitted_state():
    with pytest.raises(ValueError, match="T must"):
        make_private_spherical_kmeans(
            csr_array2_domain(4, T=bool), METRIC, dp.max_divergence(), 1, 1.0, T=dp.f64
        )
    with pytest.raises(ValueError, match="quantize"):
        make_private_spherical_kmeans(
            csr_array2_domain(1024**2 + 1, T=bool),
            METRIC,
            dp.max_divergence(),
            1,
            1.0,
            T=dp.i32,
        )
    for metric, measure in [
        (dp.change_one_distance(), dp.max_divergence()),
        (METRIC, dp.fixed_smoothed_max_divergence()),
    ]:
        with pytest.raises(ValueError):
            make_private_spherical_kmeans(
                csr_array2_domain(4, T=bool), metric, measure, 1, 1.0, T=dp.i32
            )
    with pytest.raises(ValueError, match="T=bool"):
        make_private_spherical_kmeans(
            csr_array2_domain(4, T=float), METRIC, dp.max_divergence(), 1, 1.0, T=dp.i32
        )
    for budget in [0.0, math.nextafter(0.0, 1.0)]:
        with pytest.raises(ValueError):
            make_private_spherical_kmeans(
                csr_array2_domain(4, T=bool),
                METRIC,
                dp.max_divergence(),
                1,
                budget,
                T=dp.i32,
            )
    model = SphericalKMeans(**PARAMS)
    with pytest.raises(ValueError, match="not been fitted"):
        model.predict(data())
    model._ingest_release(centers())
    with pytest.raises(ValueError, match="number of features"):
        model.make_transform(
            dp.sklearn.sklearn_fit_domain(csr_array2_domain(5, T=bool)), METRIC
        )


@pytest.mark.parametrize("kind", ["epsilon", "rho"])
def test_private_updates_on_separable_records(kind):
    model = SphericalKMeans(**PARAMS)
    model._ingest_release(
        mechanism(kind, init=centers())(sp.vstack([data()] * 1000, format="csr"))
    )
    np.testing.assert_array_equal(model.predict(data()), [0, 0, 1, 1, 0])
