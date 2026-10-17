from __future__ import annotations

import math
import pickle

import pytest

import opendp.prelude as dp
from opendp._internal import _make_transformation
from opendp.extras.scipy import CSRArray2Domain, csr_array2_domain
from opendp.extras.sklearn.cluster import SphericalKMeans, make_private_spherical_kmeans
from opendp.extras.sklearn.cluster import _spherical_lloyd_binary as spherical

np = pytest.importorskip("numpy")
sparse = pytest.importorskip("scipy.sparse")
pytest.importorskip("sklearn")

dp.enable_features("contrib")

_PARAMS = dict(
    n_clusters=2,
    max_iter=4,
    max_features_per_center=3,
    max_features_per_record=3,
    random_state=0,
)
_METRIC = dp.symmetric_distance()


def _two_blob_data(reps=1):
    """Supported input: canonical boolean CSR, with two separated feature sets."""
    block = np.array(
        [
            [1, 1, 1, 0, 0, 0],
            [1, 1, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0],
            [0, 0, 0, 1, 1, 1],
            [0, 0, 0, 1, 1, 0],
            [0, 0, 0, 0, 1, 1],
        ],
        dtype=bool,
    )
    return sparse.csr_array(np.tile(block, (reps, 1)))


def _centers():
    return sparse.csr_array([[1, 1, 1, 0, 0, 0], [0, 0, 0, 1, 1, 1]], dtype=np.int32)


def _require_i64_dispatch():
    try:
        dp.l1_distance(T=dp.i64)
    except dp.OpenDPException as err:
        if "No match for concrete type i64" not in str(err):
            raise
        pytest.skip(
            "pure-DP spherical counts need i64 L1 dispatch: use a full-dispatch release binary"
        )


def _measure(kind):
    if kind == "epsilon":
        _require_i64_dispatch()
    return (
        dp.max_divergence() if kind == "epsilon" else dp.zero_concentrated_divergence()
    )


def _measurement(domain=None, *, kind="rho", d_in=1, budget=0.5, **params):
    return make_private_spherical_kmeans(
        domain if domain is not None else csr_array2_domain(6, max_rows=2**31 - 1, T=bool),
        _METRIC,
        _measure(kind),
        d_in,
        budget,
        **{**_PARAMS, **params},
    )


def _fit(est=None, x=None, *, kind="rho", **kwargs):
    kwargs.setdefault("domain", dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool)))
    return (est if est is not None else SphericalKMeans(**_PARAMS)).fit(
        x if x is not None else _two_blob_data(),
        privacy_unit=(_METRIC, 1),
        privacy_loss=(_measure(kind), 0.5),
        **kwargs,
    )


def _context(x, *, kind="rho", domain=None, split=1):
    return dp.Context.compositor(
        dp.sklearn.SklearnFit(x) if domain is None else x,
        (_METRIC, 1),
        (_measure(kind), 0.5),
        domain=domain if domain is not None else dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool)),
        split_evenly_over=split,
    )


@pytest.mark.parametrize("kind", ["epsilon", "rho"])
def test_constructor_calibrates_and_releases_integer_centers(kind):
    m = _measurement(kind=kind)
    assert m.map(1) <= 0.5
    assert m.map(1) == pytest.approx(0.5)
    # Adaptive composition guarantees only its configured input distance.
    with pytest.raises(dp.OpenDPException, match="d_in.*no greater"):
        m.map(2)
    release = m(_two_blob_data())
    assert isinstance(release, sparse.csr_array)
    assert release.shape == (2, 6)
    assert release.dtype == np.int32
    assert release.has_canonical_format
    assert (release.data > 0).all()
    assert (np.diff(release.indptr) <= 3).all()


@pytest.mark.parametrize("kind", ["epsilon", "rho"])
def test_group_privacy_calibrates_at_requested_distance(kind):
    m = _measurement(kind=kind, d_in=2)
    assert m.map(2) <= 0.5
    assert m.map(2) == pytest.approx(0.5)
    assert m.map(1) <= m.map(2)


@pytest.mark.parametrize(
    "name",
    ["n_clusters", "max_iter", "max_features_per_record", "max_features_per_center"],
)
def test_invalid_algorithm_parameters(name):
    with pytest.raises(ValueError, match="positive"):
        _measurement(**{name: 0})


def test_invalid_seed_metric_and_measure():
    with pytest.raises(ValueError, match="nonnegative"):
        _measurement(random_state=-1)
    with pytest.raises(ValueError, match="add/remove"):
        make_private_spherical_kmeans(
            csr_array2_domain(6, max_rows=2**31 - 1, T=bool), dp.change_one_distance(), _measure("rho"), 1, 0.5
        )
    with pytest.raises(ValueError, match="max_divergence.*zero_concentrated"):
        make_private_spherical_kmeans(
            csr_array2_domain(6, max_rows=2**31 - 1, T=bool), _METRIC, dp.fixed_smoothed_max_divergence(), 1, 0.5
        )
    with pytest.raises(TypeError, match="config"):
        _measurement(config=object())


@pytest.mark.parametrize(
    ("d_in", "error"), [(1.5, TypeError), (True, ValueError), (0, ValueError)]
)
def test_invalid_distance(d_in, error):
    with pytest.raises(error, match="integer|positive"):
        _measurement(d_in=d_in)


@pytest.mark.parametrize("budget", [float("nan"), float("inf"), 0.0, -1.0])
def test_invalid_budget(budget):
    with pytest.raises(ValueError, match="finite and positive"):
        _measurement(budget=budget)


def test_domain_exports_and_variable_row_counts():
    domain = csr_array2_domain(6, max_rows=None, T=bool)
    assert domain.cast(CSRArray2Domain) == CSRArray2Domain(6, None, "bool")
    assert not hasattr(domain.descriptor, "max_active")
    assert domain.member(_two_blob_data())
    assert domain.member(sparse.csr_array((0, 6), dtype=bool))
    assert not csr_array2_domain(6, max_rows=5, T=bool).member(_two_blob_data())


@pytest.mark.parametrize(
    "invalid",
    [
        lambda: _two_blob_data().toarray(),
        lambda: sparse.csr_matrix(_two_blob_data()),
        lambda: _two_blob_data().astype(np.int32),
        lambda: sparse.csr_array(
            (np.array([True, True]), [0, 0], [0, 2]), shape=(1, 6)
        ),
        lambda: sparse.csr_array(
            (np.array([True, True]), [1, 0], [0, 2]), shape=(1, 6)
        ),
        lambda: sparse.csr_array((np.array([False]), [0], [0, 1]), shape=(1, 6)),
        lambda: sparse.csr_array(np.ones(6, dtype=bool)),
    ],
)
def test_domain_rejects_noncanonical_or_nonboolean_input(invalid):
    x = invalid()
    assert not csr_array2_domain(6, max_rows=2**31 - 1, T=bool).member(x)


@pytest.mark.parametrize("max_rows", [None, 2**31])
def test_counts_allow_unbounded_or_large_row_bounds(max_rows):
    domain = csr_array2_domain(6, max_rows=max_rows, T=bool)
    transform = spherical.make_cluster_feature_sums(domain, _METRIC, centers=_centers(), max_active=3)
    assert transform.map(1) == pytest.approx(math.sqrt(3))
    assert _measurement(domain).map(1) <= 0.5


@pytest.mark.parametrize(("norm", "sensitivity"), [(1, 3), (2, math.sqrt(3))])
def test_exact_feature_sums_and_sensitivity(norm, sensitivity):
    if norm == 1:
        _require_i64_dispatch()
    t = spherical.make_cluster_feature_sums(
        csr_array2_domain(6, max_rows=2**31 - 1, T=bool),
        _METRIC,
        centers=_centers(),
        max_active=3,
        norm=norm,
    )
    assert t.map(0) == 0
    assert t.map(1) == pytest.approx(sensitivity)
    assert t.map(3) == pytest.approx(3 * sensitivity)
    x = _two_blob_data(reps=100)
    raw_counts = spherical._cluster_feature_sums(x, _centers(), 3)
    assert raw_counts.dtype == np.int32
    counts = np.asarray(t(x))  # OpenDP's integer vector carrier returns a list.
    np.testing.assert_array_equal(counts, raw_counts)
    np.testing.assert_array_equal(
        counts.reshape(2, 6), [[300, 200, 200, 0, 0, 0], [0, 0, 0, 200, 300, 200]]
    )
    np.testing.assert_array_equal(
        t(sparse.csr_array((0, 6), dtype=bool)), np.zeros(12, dtype=np.int32)
    )


def test_count_saturation_precedes_int32_cast(monkeypatch):
    maximum = np.iinfo(np.int32).max
    counts = np.array([maximum - 1, maximum, maximum + 1] + [0] * 9, dtype=np.int64)
    monkeypatch.setattr(np, "bincount", lambda *args, **kwargs: counts)
    result = spherical._cluster_feature_sums(_two_blob_data(), _centers(), 3)
    np.testing.assert_array_equal(result[:3], [maximum - 1, maximum, maximum])
    assert result.dtype == np.int32


def test_feature_count_constructor_snapshots_centers():
    centers = _centers()
    transform = spherical.make_cluster_feature_sums(
        csr_array2_domain(6, max_rows=None, T=bool), _METRIC, centers=centers, max_active=3,
    )
    before = transform(_two_blob_data())
    centers.data[:] = 0
    np.testing.assert_array_equal(transform(_two_blob_data()), before)
    assert transform.map(1) == pytest.approx(math.sqrt(3))


def test_clipping_is_binary_and_keeps_lowest_feature_ids():
    x = sparse.csr_array([[1, 0, 1, 1], [0, 1, 0, 0], [0, 0, 0, 0]], dtype=bool)
    clipped = spherical._clip_rows(x, 2)
    np.testing.assert_array_equal(
        clipped.toarray(), [[1, 0, 1, 0], [0, 1, 0, 0], [0, 0, 0, 0]]
    )
    assert clipped.dtype == bool and clipped.has_canonical_format
    assert spherical._clip_rows(x, 4) is x
    assert x.nnz == 4
    centers = sparse.csr_array([[1, 0, 0, 0]], dtype=np.int32)
    np.testing.assert_array_equal(
        spherical._cluster_feature_sums(x, centers, 2), [1, 1, 1, 0]
    )


def test_cosine_assignment_normalizes_centers_and_handles_zeros():
    x = sparse.csr_array([[1, 0], [0, 1], [0, 0]], dtype=bool)
    centers = sparse.csr_array([[100, 100], [1, 0]], dtype=np.int32)
    np.testing.assert_array_equal(
        spherical.nearest_center_labels(x, centers), [1, 0, 0]
    )
    distances = spherical._cosine_distances(x, centers)
    np.testing.assert_allclose(distances[0], [1 - 1 / math.sqrt(2), 0])
    np.testing.assert_array_equal(distances[2], [1, 1])
    np.testing.assert_array_equal(
        spherical._cosine_distances(x, sparse.csr_array((1, 2), dtype=np.int32)),
        np.ones((3, 1)),
    )


def test_projection_retains_integer_top_counts():
    out = spherical._project_centers(
        np.array([-1, 3, 9, 2, 4, 1], dtype=np.int32), 2, 3, 2
    )
    np.testing.assert_array_equal(out.toarray(), [[0, 3, 9], [2, 4, 0]])
    assert out.dtype == np.int32 and out.has_canonical_format
    with pytest.raises(TypeError, match="int32"):
        spherical._project_centers(np.zeros(6), 2, 3, 2)


@pytest.mark.parametrize("kind", ["epsilon", "rho"])
def test_standalone_fit_with_explicit_domain(kind):
    est = SphericalKMeans(**_PARAMS)
    assert (
        _fit(
            est,
            kind=kind,
        )
        is est
    )
    assert isinstance(est.cluster_centers_, sparse.csr_array)
    assert est.cluster_centers_.dtype == np.int32
    assert est.n_features_in_ == 6
    assert est.predict(_two_blob_data()).shape == (6,)
    assert est.transform(_two_blob_data()).shape == (6, 2)
    assert np.isfinite(est.score(_two_blob_data()))


@pytest.mark.parametrize("route", ["standalone", "context"])
@pytest.mark.parametrize("with_target", [False, True])
@pytest.mark.parametrize("empty", [False, True])
def test_fitting_infers_sparse_sklearn_domain(route, with_target, empty):
    X = sparse.csr_array((0, 6), dtype=bool) if empty else _two_blob_data()
    y = np.zeros(X.shape[0], dtype=np.int32) if with_target else None
    data = dp.sklearn.SklearnFit(X, y)
    model = SphericalKMeans(**_PARAMS)
    if route == "standalone":
        assert model.fit(
            X, y, privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(rho=0.5),
        ) is model
    else:
        context = dp.Context.compositor(
            data, privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(rho=0.5), split_evenly_over=1,
        )
        assert context.query().sklearn(model).release() is model
        assert context.current_privacy_loss() == [0.5]
    assert model.n_features_in_ == 6
    assert model.cluster_centers_.shape == (2, 6)
    domain = dp.domain_of(data, infer=True)
    assert domain.cast(dp.sklearn.SklearnFitDomain).X.cast(CSRArray2Domain).max_rows is None
    assert model.make_transform(domain, _METRIC).output_domain.member(
        dp.sklearn.SklearnFit(model.transform(X), y)
    )


def test_standalone_empty_input_releases_valid_centers():
    empty = sparse.csr_array((0, 6), dtype=bool)
    est = _fit(x=empty)
    assert est.cluster_centers_.dtype == np.int32
    assert est.cluster_centers_.shape == (2, 6)
    assert est.predict(empty).shape == (0,)
    assert est.transform(empty).shape == (0, 2)
    assert est.score(empty) == 0


def test_fit_requires_privacy_and_rejects_unknown_params():
    est = SphericalKMeans(**_PARAMS)
    with pytest.raises(TypeError, match="explicit privacy_unit and privacy_loss"):
        est.fit(_two_blob_data())
    with pytest.raises(TypeError, match="unexpected keyword argument 'unknown'"):
        _fit(est, unknown=True)


def test_make_is_calibrated_and_does_not_mutate():
    est = SphericalKMeans(**_PARAMS)
    state = est.__dict__.copy()
    args = (dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool)), _METRIC, _measure("rho"), 1, 0.5)
    assert est.make(*args).map(1) <= 0.5
    assert est.__dict__ == state


def test_clone_get_params_set_params_and_fitted_center_authority():
    from sklearn.base import clone

    est = _fit()
    assert est.get_params() == _PARAMS
    cloned = clone(est)
    assert cloned is not est and cloned.get_params() == _PARAMS
    assert not hasattr(cloned, "cluster_centers_")
    before = est.transform(_two_blob_data())
    assert est.set_params(n_clusters=3, max_iter=1) is est
    np.testing.assert_array_equal(est.transform(_two_blob_data()), before)
    assert (
        est.make_transform(
            dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool)), _METRIC
        ).output_domain.descriptor.X.descriptor.num_columns
        == 2
    )


@pytest.mark.parametrize("kind", ["epsilon", "rho"])
def test_query_is_lazy_refits_supplied_estimator_and_charges_exactly_once(kind, monkeypatch):
    calls = []
    original = spherical._cluster_feature_sums

    def count_calls(*args):
        calls.append(1)
        return original(*args)

    monkeypatch.setattr(spherical, "_cluster_feature_sums", count_calls)
    ctx = _context(_two_blob_data(), kind=kind)
    template = SphericalKMeans(**_PARAMS)
    template._ingest_release(_centers())
    state = template.cluster_centers_.copy()
    query = ctx.query()
    chain = query._chain
    fitted_query = query.sklearn(template)
    assert fitted_query.resolve().map(1) <= 0.5
    assert calls == [] and ctx.current_privacy_loss() == []
    np.testing.assert_array_equal(template.cluster_centers_.toarray(), state.toarray())
    template.set_params(n_clusters=3)
    fitted = fitted_query.release()
    assert fitted is template
    assert fitted.cluster_centers_.shape == (2, 6)
    assert template.n_clusters == 3
    assert query._chain is chain
    assert len(calls) == _PARAMS["max_iter"]
    assert ctx.current_privacy_loss() == [0.5]
    with pytest.raises(ValueError, match="exhausted"):
        ctx.query()
    released_centers = fitted.cluster_centers_.copy()
    with pytest.raises(dp.OpenDPException, match="out of queries"):
        fitted_query.release()
    np.testing.assert_array_equal(template.cluster_centers_.toarray(), released_centers.toarray())
    assert len(calls) == _PARAMS["max_iter"]


@pytest.mark.parametrize("kind", ["epsilon", "rho"])
def test_preceding_nonidentity_transformation_calibrates_distance(kind, monkeypatch):
    x = _two_blob_data()
    domain = dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool))
    duplicate = _make_transformation(
        domain,
        _METRIC,
        domain,
        _METRIC,
        lambda data: dp.sklearn.SklearnFit(sparse.vstack([data.X, data.X], format="csr")),
        lambda distance: distance * 2,
    )
    calls = []
    transformed_inputs = []
    original_sums = spherical._cluster_feature_sums

    def sums(data, *args):
        transformed_inputs.append(data)
        return original_sums(data, *args)

    monkeypatch.setattr(spherical, "_cluster_feature_sums", sums)
    original = SphericalKMeans.make

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        calls.append((input_domain, input_metric, d_in, d_out))
        return original(self, input_domain, input_metric, output_measure, d_in, d_out)

    monkeypatch.setattr(SphericalKMeans, "make", make)
    ctx = _context(x, kind=kind)
    query = ctx.query().new_with(chain=duplicate).sklearn(SphericalKMeans(**_PARAMS))
    assert query.resolve().map(1) <= 0.5
    assert query.release().cluster_centers_.shape == (2, 6)
    assert calls and all(call == (domain, _METRIC, 2, 0.5) for call in calls)
    assert len(transformed_inputs) == _PARAMS["max_iter"]
    for value in transformed_inputs:
        np.testing.assert_array_equal(
            value.toarray(), np.vstack([x.toarray(), x.toarray()])
        )
    assert ctx.current_privacy_loss() == [0.5]


def _fit_data():
    vector = dp.vector_domain(dp.atom_domain(T=int))
    domain = dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool), vector)
    data = dp.sklearn.SklearnFit(_two_blob_data(), list(range(6)))
    return domain, data


@pytest.mark.parametrize("route", ["query", "standalone"])
def test_fit_supports_private_targets(route):
    domain, data = _fit_data()
    template = SphericalKMeans(**_PARAMS)
    if route != "query":
        fitted = _fit(
            template, data.X, y=data.y, domain=domain,
        )
        assert fitted is template
    else:
        # Context inputs are always a complete, privately aligned carrier.
        ctx = _context(data, domain=domain)
        fitted = ctx.query().sklearn(template).release()
        assert fitted is template
    assert fitted.n_features_in_ == 6
    assert domain.member(data)
    assert "y" not in vars(fitted)


@pytest.mark.parametrize("standalone", [False, True])
def test_release_copy_detaches_opendp_metadata_and_pickle_inference(standalone):
    x = _two_blob_data()
    est = (
        _fit()
        if standalone
        else _context(x).query().sklearn(SphericalKMeans(**_PARAMS)).release()
    )
    assert "__opendp_dict__" not in vars(est.cluster_centers_)
    assert set(vars(est)) == set(_PARAMS) | {"cluster_centers_", "n_features_in_"}
    restored = pickle.loads(pickle.dumps(est))
    np.testing.assert_array_equal(restored.predict(x), est.predict(x))
    np.testing.assert_array_equal(restored.transform(x), est.transform(x))
    assert restored.score(x) == est.score(x)
    domain = dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool))
    assert restored.make_transform(domain, _METRIC).output_domain.member(dp.sklearn.SklearnFit(restored.transform(x)))


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_refitting_replaces_centers_and_keeps_old_transform_valid(route):
    domain = dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool))
    data = dp.sklearn.SklearnFit(_two_blob_data())
    model = SphericalKMeans(**_PARAMS)
    context = _context(data, domain=domain, split=2)

    def fit():
        return _fit(model) if route == "standalone" else context.query().sklearn(model).release()

    assert fit() is model
    old_centers = model.cluster_centers_
    transform = model.make_transform(domain, _METRIC)
    before = transform(data).X
    model.cluster_centers_.data[:] = 0
    model.set_params(n_clusters=3, max_iter=1)
    assert fit() is model
    assert model.cluster_centers_ is not old_centers
    assert model.cluster_centers_.shape == (3, 6)
    np.testing.assert_array_equal(transform(data).X, before)
    assert transform.output_domain.descriptor.X.descriptor.num_columns == 2
    if route == "context":
        assert context.current_privacy_loss() == [0.25, 0.25]


def test_inherited_fit_transform_uses_private_fit():
    model = SphericalKMeans(**_PARAMS)
    X = _two_blob_data()
    result = model.fit_transform(
        X, domain=dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool)),
        privacy_unit=(_METRIC, 1), privacy_loss=(_measure("rho"), 0.5),
    )
    np.testing.assert_array_equal(result, model.transform(X))
    assert result.shape == (6, 2)


def test_ingestion_copies_all_mutable_center_buffers_without_metadata():
    release = _measurement()(_two_blob_data())
    assert "__opendp_dict__" in vars(release)
    est = SphericalKMeans(**_PARAMS)
    est._ingest_release(release)
    assert est.cluster_centers_ is not release
    assert "__opendp_dict__" not in vars(est.cluster_centers_)
    for name in ("data", "indices", "indptr"):
        assert not np.shares_memory(
            getattr(est.cluster_centers_, name), getattr(release, name)
        )
    before = est.cluster_centers_.toarray()
    release.data[:] = 0
    np.testing.assert_array_equal(est.cluster_centers_.toarray(), before)


def test_rowwise_transform_predict_preserve_targets_and_empty_inputs():
    domain, data = _fit_data()
    value = data
    est = SphericalKMeans(**_PARAMS)
    est._ingest_release(_centers())
    for constructor, inference in [
        (est.make_transform, est.transform),
        (est.make_predict, est.predict),
    ]:
        t = constructor(domain, _METRIC)
        result = t(value)
        assert t.output_domain.member(result)
        assert t.map(0) == 0 and t.map(3) == 3
        np.testing.assert_array_equal(
            result.X, inference(data.X)
        )
        assert result.y is data.y
        retained = dp.sklearn.SklearnFit(data.X[:1], data.y[:1])
        out = t(retained)
        np.testing.assert_array_equal(
            result.X[:1], out.X
        )
        empty_x = sparse.csr_array((0, 6), dtype=bool)
        empty = dp.sklearn.SklearnFit(empty_x, [])
        out_empty = t(empty)
        assert t.output_domain.member(out_empty)
        assert len(out_empty.X) == 0
    snapshot = est.make_transform(domain, _METRIC)
    before = snapshot(value)
    est.cluster_centers_.data[:] = 0
    after = snapshot(value)
    np.testing.assert_array_equal(
        before.X, after.X
    )


def test_postprocessing_requires_fitted_matching_domain_metric_and_score_bound():
    est = SphericalKMeans(**_PARAMS)
    with pytest.raises(ValueError, match="not been fitted"):
        est.predict(_two_blob_data())
    est._ingest_release(_centers())
    for method in (est.make_transform, est.make_predict):
        with pytest.raises(ValueError, match="number of features"):
            method(dp.sklearn.sklearn_fit_domain(csr_array2_domain(5, max_rows=None, T=bool)), _METRIC)
        with pytest.raises(ValueError, match="add/remove"):
            method(dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool)), dp.change_one_distance())

@pytest.mark.parametrize("kind", ["epsilon", "rho"])
def test_recovers_well_separated_blobs_with_supported_input(kind):
    est = SphericalKMeans(**_PARAMS)
    est.fit(
        _two_blob_data(reps=200),
        domain=dp.sklearn.sklearn_fit_domain(csr_array2_domain(6, max_rows=None, T=bool)),
        privacy_unit=(_METRIC, 1),
        privacy_loss=(_measure(kind), 50.0),
    )
    labels = est.predict(_two_blob_data())
    assert len(set(labels[:3])) == len(set(labels[3:])) == 1
    assert labels[0] != labels[3]


@pytest.mark.parametrize("T", [np.int32, float])
def test_spherical_constructors_require_boolean_domains(T):
    domain = csr_array2_domain(6, max_rows=10, T=T)
    with pytest.raises(ValueError, match="T=bool"):
        _measurement(domain)
    with pytest.raises(ValueError, match="T=bool"):
        spherical.make_cluster_feature_sums(domain, _METRIC, centers=_centers(), max_active=3)
    est = SphericalKMeans(**_PARAMS)
    est._ingest_release(_centers())
    for method in (est.make_transform, est.make_predict):
        with pytest.raises(ValueError, match="T=bool"):
            method(dp.sklearn.sklearn_fit_domain(domain), _METRIC)


def test_cluster_exports_do_not_restore_obsolete_api():
    import opendp.extras.sklearn.cluster as cluster

    assert set(cluster.__all__) == {
        "SphericalKMeans",
        "make_private_spherical_kmeans",
        "then_private_spherical_kmeans",
    }
    for name in (
        "sparse_binary_domain",
        "SphericalKMeansConfig",
        "SphericalKMeansRelease",
        "make_private_cluster_sizes",
        "make_cluster_feature_sums",
    ):
        assert not hasattr(cluster, name)
