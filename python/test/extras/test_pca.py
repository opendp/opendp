import warnings

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
        dp.sklearn.decomposition.PCA(n_components=2).fit(_standalone_pca_query())
    assert released == []


def test_pca_context_rejects_row_norm_before_release(monkeypatch):
    pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    from opendp.context import Query

    released = []
    monkeypatch.setattr(Query, "release", lambda self: released.append(True))
    model = dp.sklearn.decomposition.PCA(n_components=2, row_norm=1.0)
    with pytest.raises(TypeError, match="np_clip"):
        model.fit(_standalone_pca_query())
    assert released == []


def test_pca_make_rejects_row_norm_in_context_mode():
    pytest.importorskip("numpy")
    domain = dp.numpy.array2_domain(
        norm=1.0,
        p=2,
        size=10,
        num_columns=4,
        nan=False,
        T=float,
    )
    with pytest.raises(TypeError, match="np_clip"):
        dp.sklearn.decomposition.PCA(n_components=2, row_norm=1.0).make(
            domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
        )


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
        measurement = estimator.make(
            domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
        )
        release = measurement(data)
        assert release.components.shape == (expected_rows, 4)


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
    released = []

    class FakeQuery:
        def release(self):
            released.append(True)
            return object()

    monkeypatch.setattr(Query, "_sklearn", lambda self, estimator: FakeQuery())
    import_dependency = decomposition.import_optional_dependency

    def fail_sklearn(name, *args, **kwargs):
        if name == "sklearn":
            raise ImportError("sklearn is unavailable")
        return import_dependency(name, *args, **kwargs)

    monkeypatch.setattr(decomposition, "import_optional_dependency", fail_sklearn)
    with pytest.raises(ImportError, match="sklearn is unavailable"):
        dp.sklearn.decomposition.PCA().fit(query)
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

    estimator._fit_n_samples = 10
    estimator._fit_n_features = 3
    with pytest.raises(TypeError, match="expected a PCARelease"):
        estimator._ingest_release(object())  # type: ignore[arg-type]
    estimator._ingest_release(
        PCARelease(
            mean=np.zeros(3),
            singular_values=np.ones(3),
            components=np.eye(3),
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

    release = PCARelease(
        mean=np.zeros(3),
        singular_values=np.ones(3),
        components=np.eye(3),
    )

    estimator = PCA()
    estimator._fit_n_samples = 1
    estimator._fit_n_features = 3
    with pytest.raises(ValueError, match="at least two samples"):
        estimator._ingest_release(release)

    estimator = PCA(n_components="mle")
    estimator._fit_n_samples = 2
    estimator._fit_n_features = 3
    with pytest.raises(ValueError, match="n_samples >= n_features"):
        estimator._ingest_release(release)

    estimator = PCA(n_components=1.0)
    estimator._fit_n_samples = 10
    estimator._fit_n_features = 3
    with pytest.raises(ValueError, match=r"in \(0, 1\)"):
        estimator._ingest_release(release)

    estimator = PCA(n_components=0)
    estimator._fit_n_samples = 10
    estimator._fit_n_features = 3
    with pytest.raises(ValueError, match="between 1"):
        estimator._ingest_release(release)

    estimator = PCA(n_components="invalid")
    estimator._fit_n_samples = 10
    estimator._fit_n_features = 3
    with pytest.raises(ValueError, match="None, an integer"):
        estimator._ingest_release(release)

    estimator = PCA()
    estimator._fit_n_samples = 10
    estimator._fit_n_features = 3
    short_release = PCARelease(
        mean=np.zeros(3),
        singular_values=np.ones(1),
        components=np.ones((1, 3)),
    )
    with pytest.raises(ValueError, match="incompatible"):
        estimator._ingest_release(short_release)

    valid_release = PCARelease(
        mean=np.zeros(3),
        singular_values=np.array([3.0, 2.0, 1.0]),
        components=np.eye(3),
    )
    for estimator in (PCA(n_components="mle"), PCA(n_components=0.8)):
        estimator._fit_n_samples = 10
        estimator._fit_n_features = 3
        estimator._ingest_release(valid_release)
        assert estimator.n_components_ >= 1


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


def test_pca_legacy_array_fit():
    pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    pytest.importorskip("scipy.linalg")
    data = _legacy_data()
    model = dp.sklearn.decomposition.PCA(
        epsilon=1.0,
        row_norm=1.0,
        n_samples=len(data),
        n_features=data.shape[1],
        n_components=2,
    )

    with pytest.warns(FutureWarning):
        assert model.fit(data) is model
    assert model.components_.shape == (2, data.shape[1])
    with pytest.warns(FutureWarning), pytest.raises(ValueError, match="already been fitted"):
        model.fit(data)


def test_pca_legacy_measurement():
    pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    pytest.importorskip("scipy.linalg")
    data = _legacy_data()
    model = dp.sklearn.decomposition.PCA(
        epsilon=1.0,
        row_norm=1.0,
        n_samples=len(data),
        n_features=data.shape[1],
        n_components=2,
    )

    with pytest.warns(FutureWarning):
        measurement = model.measurement()
    assert measurement(data) is model
    assert model.components_.shape == (2, data.shape[1])


def test_pca_legacy_mode_tracks_set_params():
    pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    pytest.importorskip("scipy.linalg")
    data = _legacy_data()
    model = dp.sklearn.decomposition.PCA(n_components=2, row_norm=1.0)
    model.set_params(
        epsilon=1.0,
        n_samples=len(data),
        n_features=data.shape[1],
    )

    with pytest.warns(FutureWarning):
        assert model.fit(data) is model


def test_pca_incomplete_legacy_mode_after_set_params_fails_before_measurement(
    monkeypatch,
):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.decomposition import PCA

    model = PCA(
        epsilon=1.0,
        n_samples=10,
        n_features=4,
        row_norm=1.0,
    )
    model.set_params(epsilon=None)
    monkeypatch.setattr(
        model,
        "_make_legacy_measurement",
        lambda: pytest.fail("measurement should not be constructed"),
    )

    with pytest.raises(
        TypeError,
        match="requires epsilon, n_samples, and n_features together",
    ):
        model.fit(np.zeros((10, 4)))


def test_pca_legacy_constructor_validation():
    from opendp.extras.sklearn.decomposition import PCA

    for parameter in ("epsilon", "n_samples", "n_features"):
        with pytest.raises(TypeError, match="requires epsilon"):
            PCA(**{parameter: 1})
    with pytest.raises(ValueError, match="epsilon must be positive"):
        PCA(epsilon=0, n_samples=10, n_features=4)
    with pytest.raises(ValueError, match="n_samples must be greater"):
        PCA(epsilon=1, n_samples=1, n_features=4)
    with pytest.raises(ValueError, match="n_features must be at least"):
        PCA(epsilon=1, n_samples=10, n_features=0)
    with pytest.raises(ValueError, match="n_changes must be at least"):
        PCA(epsilon=1, n_samples=10, n_features=4, n_changes=0)


def test_pca_legacy_shape_validation_before_measurement(monkeypatch):
    np = pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    model = dp.sklearn.decomposition.PCA(
        epsilon=1.0,
        row_norm=1.0,
        n_samples=10,
        n_features=4,
    )
    monkeypatch.setattr(
        model,
        "_make_legacy_measurement",
        lambda: pytest.fail("measurement should not be constructed"),
    )
    with pytest.warns(FutureWarning), pytest.raises(ValueError, match="shape"):
        model.fit(np.zeros((9, 4)))


def test_pca_legacy_and_context_api_are_not_mixed():
    np = pytest.importorskip("numpy")
    from opendp.context import Query
    from opendp.extras.sklearn.decomposition import PCA

    domain = dp.numpy.array2_domain(size=10, num_columns=4, T=float, nan=False)
    query = Query(
        chain=(domain, dp.symmetric_distance()),
        output_measure=dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    model = PCA(epsilon=1.0, n_samples=10, n_features=4, row_norm=1.0)
    with pytest.raises(TypeError, match="Do not provide epsilon"):
        model.fit(query)

    with pytest.raises(TypeError, match="expects an OpenDP Context query"):
        PCA(row_norm=1.0).fit(np.zeros((10, 4)))


def test_pca_sklearn_introspection_in_legacy_mode():
    pytest.importorskip("sklearn")
    from sklearn.base import clone

    model = dp.sklearn.decomposition.PCA(
        epsilon=1.0,
        n_samples=100,
        n_features=4,
        n_changes=2,
        n_components=2,
        row_norm=1.0,
    )
    cloned = clone(model)
    assert cloned.get_params() == model.get_params()


def test_pca_context_fit_and_methods():
    np = pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    data = sample_microdata(num_columns=4, num_rows=100)
    context, model = _context(data, n_components=2, whiten=True)

    assert not hasattr(model, "components_")
    with warnings.catch_warnings():
        warnings.simplefilter("error", FutureWarning)
        assert model.fit(context.query().np_clip(p=2, norm=1.0)) is model
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


def test_pca_context_fit_with_query_clipping():
    pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    data = sample_microdata(num_columns=3, num_rows=30)
    context, model = _context(data)
    model.fit(context.query().np_clip(p=2, norm=2.0))
    assert model.mean_.shape == (3,)


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
