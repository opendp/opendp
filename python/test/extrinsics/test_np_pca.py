import sys
import pytest
import opendp.prelude as dp

dp.enable_features("honest-but-curious", "contrib", "floating-point")


def sample_microdata(*, num_columns=None, num_rows=None, cov=None):
    import numpy as np

    cov = cov or sample_covariance(num_columns)
    microdata = np.random.multivariate_normal(
        np.zeros(cov.shape[0]), cov, size=num_rows or 100_000
    )
    microdata -= microdata.mean(axis=0)
    return microdata


def sample_covariance(num_features):
    import numpy as np

    A = np.random.uniform(0, num_features, size=(num_features, num_features))
    return A.T @ A


@pytest.mark.skipif("scipy" not in sys.modules, reason="Scipy needed")
def test_pca():
    from opendp._extrinsics.make_np_pca import then_private_np_pca

    num_columns = 4
    num_rows = 10_000
    space = (
        dp.np_array2_domain(norm=1, p=2, origin=0, num_columns=num_columns, size=num_rows, T=float),
        dp.symmetric_distance(),
    )
    m_pca = space >> then_private_np_pca(unit_epsilon=1.0)

    print(m_pca(sample_microdata(num_columns=num_columns, num_rows=num_rows)))
    assert m_pca.check(2, 1.0)


@pytest.mark.skipif("sklearn" not in sys.modules, reason="Scikit-Learn needed")
def test_pca_skl():
    num_columns = 4
    num_rows = 10_000
    data = sample_microdata(num_columns=num_columns, num_rows=num_rows)

    model = dp.sklearn.PCA(
        epsilon=1.0,
        row_norm=1.0,
        n_samples=num_rows,
        n_features=4,
    )

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
    import numpy as np

    signs = np.equal(np.sign(a[:, 0]), np.sign(b[:, 0])) * 2 - 1
    return a, b * signs[:, None]


def flaky_test_pca_compare_sklearn():
    import numpy as np
    from sklearn.decomposition import PCA  # type: ignore[import]

    num_columns = 4
    num_rows = 1_000_000
    data = sample_microdata(num_columns=num_columns, num_rows=num_rows)

    model_odp = dp.sklearn.PCA(
        epsilon=1_000_000.0,
        row_norm=64.0,
        n_samples=num_rows,
        n_features=4,
    )
    model_odp.fit(data)

    model_skl = PCA()
    model_skl.fit(data)

    print(model_odp)
    print(model_skl)

    print("odp singular values", model_odp.singular_values_)
    print("skl singular values", model_skl.singular_values_)
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


@pytest.mark.skipif("sklearn" not in sys.modules, reason="Scikit-Learn needed")
def test_pca_compare_sklearn():
    for _ in range(5):
        try:
            flaky_test_pca_compare_sklearn()
            break
        except AssertionError:
            pass
    else:
        assert False, "PCA failed too many times"
