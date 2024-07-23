import pytest
import opendp.prelude as dp
from opendp._lib import import_optional_dependency
from ..helpers import optional_dependency

dp.enable_features("honest-but-curious", "contrib", "floating-point")


def sample_microdata(*, num_columns=None, num_rows=None, cov=None):
    np = import_optional_dependency('numpy')

    cov = cov or sample_covariance(num_columns)
    microdata = np.random.multivariate_normal(
        np.zeros(cov.shape[0]), cov, size=num_rows or 100_000
    )
    microdata -= microdata.mean(axis=0)
    return microdata


def sample_covariance(num_features):
    np = import_optional_dependency('numpy')

    A = np.random.uniform(0, num_features, size=(num_features, num_features))
    return A.T @ A


def test_pca():
    from opendp.extras.numpy.make_pca import then_private_pca

    num_columns = 4
    num_rows = 10_000
    with optional_dependency('numpy'):
        space = (
            dp.numpy.np_array2_domain(norm=1, p=2, origin=0, num_columns=num_columns, size=num_rows, T=float),
            dp.symmetric_distance(),
        )
    with optional_dependency('scipy.linalg'):
        m_pca = space >> then_private_pca(unit_epsilon=1.0)

    with optional_dependency('randomgen'):
        print(m_pca(sample_microdata(num_columns=num_columns, num_rows=num_rows)))
    assert m_pca.check(2, 1.0)


def test_pca_skl():
    num_columns = 4
    num_rows = 10_000
    with optional_dependency('numpy'):
        data = sample_microdata(num_columns=num_columns, num_rows=num_rows)

    with optional_dependency('sklearn'):
        model = dp.sklearn.PCA(
            epsilon=1.0,
            row_norm=1.0,
            n_samples=num_rows,
            n_features=4,
        )

    with optional_dependency('randomgen'):
        model.fit(data)
    print(model)

    print("singular values", model.singular_values_)
    print("components", model.components_)

    loadings = model.singular_values_ * model.components_
    print("loadings", loadings)

    model = dp.sklearn.PCA(
        epsilon=1.0, row_norm=1.0, n_samples=num_rows, n_features=4, n_components="mle"
    )

    model.fit(data)

    model = dp.sklearn.PCA(
        epsilon=1.0, row_norm=1.0, n_samples=num_rows, n_features=4, n_components=0.4
    )
    model.fit(data)

    model = dp.sklearn.PCA(
        epsilon=1.0, row_norm=1.0, n_samples=num_rows, n_features=4, n_components=0.4
    )
    meas = model.measurement()
    meas(data)
    print(model.components_)


def flip_row_signs(a, b):
    np = pytest.importorskip('numpy')
    signs = np.equal(np.sign(a[:, 0]), np.sign(b[:, 0])) * 2 - 1
    return a, b * signs[:, None]


def flaky_assert_pca_compare_sklearn():
    num_columns = 4
    num_rows = 1_000_000
    with optional_dependency('numpy'):
        data = sample_microdata(num_columns=num_columns, num_rows=num_rows)

    with optional_dependency("sklearn"):
        model_odp = dp.sklearn.PCA(
            epsilon=1_000_000.0,
            row_norm=64.0,
            n_samples=num_rows,
            n_features=4,
        )
    model_odp.fit(data)

    sklearn = pytest.importorskip('sklearn')
    model_skl = sklearn.decomposition.PCA()
    model_skl.fit(data)

    print(model_odp)
    print(model_skl)

    print("odp singular values", model_odp.singular_values_)
    print("skl singular values", model_skl.singular_values_)
    np = pytest.importorskip('numpy')
    assert np.allclose(
        model_odp.singular_values_, model_skl.singular_values_, atol=1e-1
    )

    odp_comp, skl_comp = flip_row_signs(model_odp.components_, model_skl.components_)

    print("odp components\n", odp_comp)
    print("skl components\n", skl_comp)
    print("diff\n", odp_comp - skl_comp)
    assert np.allclose(odp_comp, skl_comp, atol=1e-3)

    print(model_skl.explained_variance_)
    print(model_odp.explained_variance_)


def test_pca_compare_sklearn():
    for _ in range(5):
        try:
            with optional_dependency('randomgen'):
                flaky_assert_pca_compare_sklearn()
            break
        except AssertionError:
            pass
    else:
        assert False, "PCA failed too many times"
