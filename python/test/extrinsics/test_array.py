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


def test_np_array2():
    print(dp.np_array2_domain(T=float))
    print(dp.np_array2_domain(T=float).descriptor)


def test_clamp():
    space = dp.np_array2_domain(T=float), dp.symmetric_distance()
    trans = space >> dp.t.then_np_clamp(norm=1.0, ord=2)
    assert trans.output_domain.member(trans(sample_microdata(num_columns=4)))


def test_sum():
    space = dp.np_array2_domain(T=float), dp.symmetric_distance()
    trans = space >> dp.t.then_np_clamp(norm=1.0, ord=2) >> dp.t.then_np_sum()

    arg = sample_microdata(num_columns=4)
    assert trans.output_domain.member(trans(arg))


def test_sum():
    space = dp.np_array2_domain(T=float), dp.symmetric_distance()
    trans = space >> dp.t.then_np_clamp(norm=1.0, ord=2) >> dp.t.then_np_sum()

    arg = sample_microdata(num_columns=4)
    assert trans.output_domain.member(trans(arg))


def test_cov():
    space = dp.np_array2_domain(T=float, num_columns=4), dp.symmetric_distance()
    trans = space >> dp.t.then_np_clamp(norm=1.0, ord=2) >> dp.t.then_np_cov()

    arg = sample_microdata(num_columns=4)
    print("clamped cov", trans(arg))
    # assert trans.output_domain.member(trans(arg))


# test_cov()


def test_eigenvector():
    num_columns = 4
    space = (
        dp.np_array2_domain(num_columns=num_columns, T=float),
        dp.symmetric_distance(),
    )
    meas = (
        space
        >> dp.t.then_np_clamp(norm=4.0, ord=2)
        >> dp.t.then_np_cov()
        >> dp.m.then_private_eigenvector(1.0)
    )
    print(meas(sample_microdata(num_columns=num_columns)))


def test_eigenvectors():
    num_columns = 4
    space = (
        dp.np_array2_domain(num_columns=num_columns, T=float),
        dp.symmetric_distance(),
    )
    meas = (
        space
        >> dp.t.then_np_clamp(norm=4.0, ord=2)
        >> dp.t.then_np_cov()
        >> dp.m.then_private_eigenvectors(1.0)
    )
    print(meas(sample_microdata(num_columns=num_columns)))


def test_pca():
    num_columns = 4
    num_rows = 10_000
    space = (
        dp.np_array2_domain(num_columns=num_columns, size=num_rows, T=float),
        dp.symmetric_distance(),
    )
    m_pca = space >> dp.m.then_pca(1.0, norm=25.)

    print(m_pca(sample_microdata(num_columns=num_columns, num_rows=num_rows)))


def test_pca_skl():
    num_columns = 4
    num_rows = 10_000
    
    model = dp.PCA(
        epsilon=1.,
        row_norm=1.,
        n_samples=num_rows,
        n_features=4,
    )

    model.fit(sample_microdata(num_columns=num_columns, num_rows=num_rows))
    print(model)

    print("singular values", model.singular_values_)
    print("components", model.components_)
    
    loadings = model.singular_values_ * model.components_
    print("loadings", loadings)