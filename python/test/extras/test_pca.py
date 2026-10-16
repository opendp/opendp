import pytest
import opendp.prelude as dp
from opendp._lib import import_optional_dependency
from ..helpers import optional_dependency


def sample_microdata(*, num_columns=None, num_rows=None, cov=None):
    np = import_optional_dependency("numpy")
    cov = cov if cov is not None else sample_covariance(num_columns)
    microdata = np.random.multivariate_normal(
        np.zeros(cov.shape[0]), cov, size=num_rows or 100_000
    )
    microdata -= microdata.mean(axis=0)
    return microdata


def sample_covariance(num_features):
    np = import_optional_dependency("numpy")
    A = np.random.uniform(0, num_features, size=(num_features, num_features))
    return A.T @ A


def _context(data, *, n_components=None, whiten=False):
    domain = dp.numpy.array2_domain(
        norm=None,
        size=len(data),
        num_columns=data.shape[1],
        nan=False,
        T=float,
    )
    context = dp.Context.compositor(
        data=data,
        domain=domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    model = dp.sklearn.decomposition.PCA(
        n_components=n_components,
        whiten=whiten,
        row_norm=1.0,
    )
    return context, model


def test_pca_measurement_is_calibrated():
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
            output_measure=dp.max_divergence(), d_in=1, d_out=1.0
        )
    assert m_pca.check(1, 1.0)
    assert m_pca.map(1) <= 1.0

    with optional_dependency("scipy.linalg"):
        direct_pca = make_private_pca(
            space[0], space[1], dp.max_divergence(), d_in=1, d_out=1.0
        )
    assert direct_pca.map(1) <= 1.0
    with pytest.raises(ValueError, match="PCA currently supports d_in=1 only"):
        make_private_pca(
            space[0], space[1], dp.max_divergence(), d_in=2, d_out=1.0
        )

    estimator = dp.sklearn.decomposition.PCA()
    measurement = estimator.make(
        space[0], space[1], dp.max_divergence(), 1, 1.0
    )
    assert measurement.map(1) <= 1.0


def test_pca_center_uses_zero_origin_for_output_domain():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import then_center

    origin = np.array([2.0, -1.0])
    input_domain = dp.numpy.array2_domain(
        norm=1,
        p=2,
        origin=origin,
        size=2,
        num_columns=2,
        nan=False,
        T=float,
    )
    transformation = (
        input_domain,
        dp.symmetric_distance(),
    ) >> then_center()
    data = np.array([[2.5, -1.0], [2.0, -0.5]])

    assert np.array_equal(transformation(data), data - origin)
    assert np.array_equal(
        transformation.output_domain.descriptor.origin, np.zeros(2)
    )
    assert transformation.output_domain.member(transformation(data))


def _standalone_pca_query():
    from opendp.context import Query

    domain = dp.numpy.array2_domain(
        size=10,
        num_columns=4,
        nan=False,
        T=float,
    )
    return Query(
        chain=(domain, dp.symmetric_distance()),
        output_measure=dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )


def test_pca_requires_known_domain_shape():
    pytest.importorskip("numpy")
    pytest.importorskip("scipy.linalg")
    domain = dp.numpy.array2_domain(num_columns=4, nan=False, T=float)
    with pytest.raises(ValueError, match="known input-domain size"):
        dp.sklearn.decomposition.PCA().make(
            domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
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


def test_pca_context_rejects_invalid_row_norm_before_release(monkeypatch):
    pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    from opendp.context import Query

    released = []
    monkeypatch.setattr(Query, "release", lambda self: released.append(True))
    model = dp.sklearn.decomposition.PCA(n_components=2, row_norm=0.0)
    with pytest.raises(ValueError, match="positive"):
        _standalone_pca_query().sklearn(model)
    assert released == []


def test_pca_make_clips_X_internally():
    np = pytest.importorskip("numpy")
    domain = dp.numpy.array2_domain(size=10, num_columns=4, nan=False, T=float)
    model = dp.sklearn.decomposition.PCA(n_components=2, row_norm=1.0)
    measurement = model.make(domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0)
    assert measurement.input_domain == domain
    assert measurement.map(1) <= 1.0
    data = np.full((10, 4), 100.0)
    release = measurement(data)
    assert release.components.shape == (2, 4)
    assert release.n_samples == 10
    np.testing.assert_array_equal(data, np.full((10, 4), 100.0))
    assert not hasattr(model, "components_")


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
            domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
        )
        assert estimator.__dict__ == initial_state
        release = measurement(data)
        assert estimator.__dict__ == initial_state
        assert release.components.shape == (expected_rows, 4)
        assert release.n_samples == 20
        assert release.n_features == 4


def test_pca_dependency_preflight_happens_before_release(monkeypatch):
    pytest.importorskip("numpy")
    from opendp.context import Query
    from opendp.extras.sklearn import decomposition

    domain = dp.numpy.array2_domain(
        size=2,
        num_columns=2,
        nan=False,
        T=float,
    )
    query = Query(
        chain=(domain, dp.symmetric_distance()),
        output_measure=dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    released: list[bool] = []

    import_dependency = decomposition.import_optional_dependency

    def fail_sklearn(name, *args, **kwargs):
        if name == "sklearn":
            raise ImportError("sklearn is unavailable")
        return import_dependency(name, *args, **kwargs)

    monkeypatch.setattr(decomposition, "import_optional_dependency", fail_sklearn)
    with pytest.raises(ImportError, match="sklearn is unavailable"):
        query.sklearn(dp.sklearn.decomposition.PCA())
    assert released == []


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
    assert np.allclose(model.inverse_transform(transformed), X)


def test_pca_fitted_methods_and_error_paths():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCARelease

    estimator = dp.sklearn.decomposition.PCA(n_components=2, whiten=True)
    public = np.zeros((3, 3))
    with pytest.raises(ValueError, match="not fitted"):
        estimator.transform(public)
    with pytest.raises(ValueError, match="not fitted"):
        estimator.inverse_transform(public[:, :2])
    with pytest.raises(ValueError, match="not fitted"):
        estimator.get_covariance()
    with pytest.raises(ValueError, match="not fitted"):
        estimator.get_precision()
    with pytest.raises(NotImplementedError, match="fit_transform would release"):
        estimator.fit_transform(public)

    with pytest.raises(TypeError, match="expected a PCARelease"):
        estimator._ingest_release(object())  # type: ignore[arg-type]
    estimator._ingest_release(
        PCARelease(
            mean=np.zeros(3),
            singular_values=np.ones(3),
            components=np.eye(3),
            n_samples=10,
            n_features=3,
        )
    )
    assert estimator.components_.shape == (2, 3)
    assert estimator.transform(public).shape == (3, 2)
    assert estimator.inverse_transform(np.zeros((3, 2))).shape == (3, 3)
    assert estimator.get_covariance().shape == (3, 3)
    assert estimator.get_precision().shape == (3, 3)
    with pytest.raises(ValueError, match="shape"):
        estimator.transform(np.zeros((3, 2)))
    with pytest.raises(ValueError, match="shape"):
        estimator.inverse_transform(np.zeros((3, 3)))


def test_pca_ingest_release_error_paths():
    np = pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    from opendp.extras.sklearn.decomposition import PCA, PCARelease

    def release(*, n_samples, n_features=3):
        return PCARelease(
            mean=np.zeros(3),
            singular_values=np.ones(3),
            components=np.eye(3),
            n_samples=n_samples,
            n_features=n_features,
        )

    with pytest.raises(ValueError, match="at least two samples"):
        PCA()._ingest_release(release(n_samples=1))

    with pytest.raises(ValueError, match="n_samples >= n_features"):
        PCA(n_components="mle")._ingest_release(release(n_samples=2))

    with pytest.raises(ValueError, match=r"in \(0, 1\)"):
        PCA(n_components=1.0)._ingest_release(release(n_samples=10))

    with pytest.raises(ValueError, match="between 1"):
        PCA(n_components=0)._ingest_release(release(n_samples=10))

    with pytest.raises(ValueError, match="None, an integer"):
        PCA(n_components="invalid")._ingest_release(release(n_samples=10))

    short_release = PCARelease(
        mean=np.zeros(3),
        singular_values=np.ones(1),
        components=np.ones((1, 3)),
        n_samples=10,
        n_features=3,
    )
    with pytest.raises(ValueError, match="incompatible"):
        PCA()._ingest_release(short_release)

    valid_release = PCARelease(
        mean=np.zeros(3),
        singular_values=np.array([3.0, 2.0, 1.0]),
        components=np.eye(3),
        n_samples=10,
        n_features=3,
    )
    for estimator in (PCA(n_components="mle"), PCA(n_components=0.8)):
        estimator._ingest_release(valid_release)
        assert estimator.n_components_ >= 1


@pytest.mark.parametrize("n_components,expected", [(1, 1), (0.8, 1), (None, 2)])
def test_pca_zero_released_variance(n_components, expected):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCARelease

    model = dp.sklearn.decomposition.PCA(n_components=n_components, whiten=True)
    model._ingest_release(PCARelease(
        mean=np.zeros(2), singular_values=np.zeros(2),
        components=np.eye(2), n_samples=10, n_features=2,
    ))
    assert model.n_components_ == expected
    np.testing.assert_array_equal(model.explained_variance_ratio_, np.zeros(expected))
    assert np.isfinite(model.transform(np.ones((1, 2)))).all()


def test_pca_component_validation():
    from opendp.extras.sklearn.decomposition import PCA

    with pytest.raises(ValueError, match="between 1"):
        PCA._validate_n_components(0, 10, 4)
    with pytest.raises(ValueError, match=r"in \(0, 1\)"):
        PCA._validate_n_components(1.0, 10, 4)
    with pytest.raises(ValueError, match="None, an integer"):
        PCA._validate_n_components(True, 10, 4)
    with pytest.raises(ValueError, match="None, an integer"):
        PCA._validate_n_components("invalid", 10, 4)
    with pytest.raises(ValueError, match="n_samples >= n_features"):
        PCA._validate_n_components("mle", 3, 4)


def _legacy_data():
    np = import_optional_dependency("numpy")
    rng = np.random.default_rng(0)
    data = rng.normal(size=(10, 4))
    data /= np.maximum(np.linalg.norm(data, axis=1, keepdims=True), 1.0)
    return data


@pytest.mark.parametrize("explicit_domain", [False, True])
def test_pca_standalone_array_fit(explicit_domain):
    data = _legacy_data()
    model = dp.sklearn.decomposition.PCA(n_components=2, row_norm=1.0)
    domain = dp.numpy.array2_domain(T=float, size=len(data), num_columns=data.shape[1]) if explicit_domain else None
    assert model.fit(
        data, domain=domain, privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    ) is model
    assert model.components_.shape == (2, data.shape[1])
    assert model.n_samples_ == len(data)


def test_pca_constructor_has_only_algorithm_parameters():
    from opendp.extras.sklearn.decomposition import PCA

    assert set(PCA().get_params()) == {"n_components", "row_norm", "whiten"}
    for parameter in ("epsilon", "n_samples", "n_features", "n_changes"):
        with pytest.raises(TypeError, match="unexpected keyword"):
            PCA(**{parameter: 1})


def test_pca_standalone_shape_validation_before_measurement(monkeypatch):
    np = pytest.importorskip("numpy")
    model = dp.sklearn.decomposition.PCA(row_norm=1.0)
    monkeypatch.setattr(model, "make", lambda *args: pytest.fail("invalid input must not be measured"))
    domain = dp.numpy.array2_domain(T=float, size=10, num_columns=4)
    with pytest.warns(UserWarning, match="rows"), pytest.raises(ValueError, match="belong"):
        model.fit(np.zeros((9, 4)), domain=domain, privacy_unit=dp.unit_of(contributions=1),
                  privacy_loss=dp.loss_of(epsilon=1.0))


def test_pca_fit_rejects_query_and_missing_privacy():
    from opendp.extras.sklearn.decomposition import PCA

    with pytest.raises(TypeError, match="not a Query"):
        PCA(row_norm=1.0).fit(_standalone_pca_query())
    with pytest.raises(TypeError, match="explicit privacy_unit and privacy_loss"):
        PCA(row_norm=1.0).fit(_legacy_data())


def test_pca_fitted_model_pickle_preserves_inference():
    import pickle
    import numpy as np
    from sklearn.base import clone

    model = dp.sklearn.decomposition.PCA(n_components=2, row_norm=1.0)
    data = _legacy_data()
    fitted = _context(data)[0].query().sklearn(model).release()
    restored = pickle.loads(pickle.dumps(fitted))
    np.testing.assert_allclose(restored.transform(data), fitted.transform(data))
    np.testing.assert_allclose(restored.get_covariance(), fitted.get_covariance())
    assert clone(fitted).get_params() == model.get_params()
    assert not hasattr(clone(fitted), "components_")
    assert not hasattr(model, "components_")


def test_pca_context_fit_and_methods():
    np = pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    data = sample_microdata(num_columns=4, num_rows=100)
    context, model = _context(data, n_components=2, whiten=True)

    assert not hasattr(model, "components_")
    template = model
    model = context.query().sklearn(template).release()
    assert model is not template
    assert not hasattr(template, "components_")
    assert model.components_.shape == (2, 4)
    assert model.n_features_in_ == 4

    public = np.asarray(data[:3])
    transformed = model.transform(public)
    assert transformed.shape == (3, 2)
    assert model.inverse_transform(transformed).shape == public.shape
    with pytest.raises(NotImplementedError, match="fit_transform would release"):
        model.fit_transform(public)


def test_pca_parameter_introspection_and_component_modes():
    pytest.importorskip("sklearn")
    from sklearn.base import clone

    model = dp.sklearn.decomposition.PCA(
        n_components=0.8, whiten=True
    )
    assert clone(model).get_params() == model.get_params()
    assert model.set_params(n_components="mle") is model
    assert model.get_params()["n_components"] == "mle"


def test_pca_fit_and_query_sklearn_release_are_equivalent():
    pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    data = sample_microdata(num_columns=3, num_rows=30)
    _, fit_model = _context(data)
    release_context, release_model = _context(data)

    assert fit_model.fit(
        data, privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(epsilon=1.0),
    ) is fit_model

    release_query = release_context.query()
    initial_state = release_model.__dict__.copy()
    fitted_query = release_query.sklearn(release_model)
    assert release_model.__dict__ == initial_state
    fitted_model = fitted_query.release()
    assert fitted_model is not release_model
    assert not hasattr(release_model, "components_")
    assert fit_model.mean_.shape == fitted_model.mean_.shape == (3,)
    assert fit_model.components_.shape == fitted_model.components_.shape


def test_pca_standalone_accepts_private_unused_targets():
    model = dp.sklearn.decomposition.PCA(n_components=2, row_norm=1.0)
    data = _legacy_data()
    assert model.fit(
        data, [0] * len(data), privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    ) is model
    assert model.components_.shape == (2, data.shape[1])


def test_pca_covariance_matches_sklearn_ppca():
    np = pytest.importorskip("numpy")
    model = dp.sklearn.decomposition.PCA()
    model.components_ = np.eye(2)
    model.explained_variance_ = np.array([4.0, 2.0])
    model.noise_variance_ = 1.0

    covariance = model.get_covariance()
    assert np.array_equal(covariance, np.diag([4.0, 2.0]))
    assert np.array_equal(model.get_precision(), np.diag([1 / 4, 1 / 2]))


def test_pca_covariance_and_precision_require_fit():
    pytest.importorskip("numpy")
    model = dp.sklearn.decomposition.PCA()
    with pytest.raises(ValueError, match="not fitted"):
        model.get_covariance()
    with pytest.raises(ValueError, match="not fitted"):
        model.get_precision()


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
    weight_domain = dp.vector_domain(dp.atom_domain(T=float), size=size)
    domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain, weight_domain, y_domain)
    aligned = dp.sklearn.SklearnFit(data.copy(), list(range(size)), [0.5] * size, list(range(size)))
    context = dp.Context.compositor(
        aligned, dp.unit_of(contributions=1), dp.loss_of(epsilon=1.0),
        domain=domain, split_evenly_over=1,
    )
    query = context.query()
    original_chain = query._chain
    model = dp.sklearn.decomposition.PCA(n_components=2, whiten=whiten, row_norm=1.0)
    if use_fit:
        assert model.fit(
            aligned.X, aligned.y, sample_weight=aligned.sample_weight, groups=aligned.groups,
            domain=domain, privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(epsilon=1.0),
        ) is model
    else:
        fitted_query = query.sklearn(model)
        assert fitted_query.resolve().map(1) <= 1.0
        template = model
        model = fitted_query.release()
        assert model is not template
        assert not hasattr(template, "components_")
    assert query._chain is original_chain
    assert model.n_features_in_ == n_features

    # The sklearn constructor also retains support for natural X-only inputs.
    measurement = model.make(X_domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0)
    assert measurement.input_domain == X_domain

    transform = model.make_transform(domain)
    result = transform(aligned)
    np.testing.assert_allclose(result.X, model.transform(data))
    assert result.X.shape == (size, 2)
    assert transform.output_domain.member(result)
    assert transform.map(1) == 1
    for name in ("y", "sample_weight", "groups"):
        assert getattr(result, name) is getattr(aligned, name)
    np.testing.assert_array_equal(aligned.X, data)

    # Once constructed, the transformation depends on the released snapshot,
    # not on subsequent estimator parameter changes.
    model.set_params(whiten=not whiten)
    np.testing.assert_allclose(transform(aligned).X, result.X)


def test_pca_make_transform_validation_and_empty_data():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCARelease

    model = dp.sklearn.decomposition.PCA(n_components=1)
    with pytest.raises(ValueError, match="not fitted"):
        model.make_transform()
    model._ingest_release(PCARelease(
        mean=np.zeros(2), singular_values=np.array([2.0, 1.0]),
        components=np.eye(2), n_samples=10, n_features=2,
    ))
    transformation = model.make_transform()
    assert transformation.map(1) == 1
    assert transformation(np.zeros((0, 2))).shape == (0, 1)
    with pytest.raises(ValueError, match="symmetric_distance"):
        model.make_transform(input_metric=dp.change_one_distance())
    with pytest.raises(ValueError, match="number of features"):
        model.make_transform(dp.numpy.array2_domain(T=float, num_columns=3))


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
    first = dp.sklearn.decomposition.PCA(n_components=2, row_norm=1.0)
    first = context.query().sklearn(first).release()
    transform = first.make_transform(fit_domain)
    second_query = context.query().new_with(chain=transform)
    second = dp.sklearn.decomposition.PCA(n_components=1, row_norm=1.0)
    fitted = second_query.sklearn(second).release()
    assert not hasattr(second, "components_")
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
        components=np.eye(2), n_samples=10, n_features=2,
    ))
    X_domain = dp.numpy.array2_domain(T=float, num_columns=2)
    y_domain = dp.vector_domain(dp.atom_domain(T=int))
    domain = dp.sklearn.sklearn_fit_domain(X_domain, y_domain, groups=y_domain)
    transform = model.make_transform(domain)
    before = dp.sklearn.SklearnFit(np.array([[1.0, 2.0]]), [0], groups=[10])
    after = dp.sklearn.SklearnFit(np.array([[1.0, 2.0], [100.0, 200.0]]), [0, 1], groups=[10, 11])
    out_before, out_after = transform(before), transform(after)
    np.testing.assert_array_equal(out_before.X, out_after.X[:1])
    assert out_before.y == out_after.y[:1]
    assert out_before.groups == out_after.groups[:1]
    assert transform.map(1) == 1
