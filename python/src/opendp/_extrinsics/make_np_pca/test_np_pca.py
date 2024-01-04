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


def test_pca():
    num_columns = 4
    num_rows = 10_000
    space = (
        dp.np_array2_domain(num_columns=num_columns, size=num_rows, T=float),
        dp.symmetric_distance(),
    )
    m_pca = space >> dp.m.then_np_pca(1.0, norm=25.0)

    print(m_pca(sample_microdata(num_columns=num_columns, num_rows=num_rows)))


def test_pca_skl():
    num_columns = 4
    num_rows = 10_000

    model = dp.PCA(
        epsilon=1.0,
        row_norm=1.0,
        n_samples=num_rows,
        n_features=4,
    )

    model.fit(sample_microdata(num_columns=num_columns, num_rows=num_rows))
    print(model)

    print("singular values", model.singular_values_)
    print("components", model.components_)

    loadings = model.singular_values_ * model.components_
    print("loadings", loadings)
