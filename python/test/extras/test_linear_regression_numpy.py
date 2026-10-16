import pytest

from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_pairwise_predict,
)
import opendp.prelude as dp


def _training_data(size=20):
    np = pytest.importorskip("numpy")
    X = np.arange(size, dtype=float).reshape(-1, 1)
    y = 2.0 * X[:, 0] + 1.0
    domain = dp.sklearn.sklearn_fit_domain(
        X=dp.numpy.array2_domain(num_columns=1, size=size, T=float, nan=False),
        y=dp.vector_domain(dp.atom_domain(T=float, nan=False), size=size),
    )
    return dp.sklearn.SklearnFit(X=X, y=y), domain


def _estimator():
    return dp.sklearn.linear_model.TheilSenRegressor(
        x_bounds=((-3.0, 25.0),), y_bounds=(-10.0, 60.0)
    )


def test_private_theil_sen_measurement_consumes_aligned_data():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.linear_model import make_private_theil_sen

    data, domain = _training_data()
    measurement = make_private_theil_sen(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        d_in=1,
        d_out=1.0,
        x_bounds=((-3.0, 3.0),),
        y_bounds=(-10.0, 60.0),
    )
    assert measurement.map(1) <= 1.0
    assert isinstance(
        measurement.input_domain.cast(dp.sklearn.SklearnFitDomain), dp.sklearn.SklearnFitDomain
    )
    slope, intercept = measurement(data)
    assert np.asarray(slope).shape == ()
    assert np.asarray(intercept).shape == ()


def test_pairwise_predict_stability():
    np = pytest.importorskip("numpy")
    x_cuts = np.array([-1.0, 1.0])

    unsized = make_pairwise_predict(
        dp.numpy.array2_domain(num_columns=2, T=float),
        dp.symmetric_distance(),
        x_cuts,
        runs=2,
    )
    assert unsized.map(1) == 4
    assert unsized.map(2) == 8

    sized = make_pairwise_predict(
        dp.numpy.array2_domain(size=5, num_columns=2, T=float),
        dp.symmetric_distance(),
        x_cuts,
        runs=2,
    )
    assert sized.map(1) == 0
    assert sized.map(2) == 4
    assert sized.map(3) == 4
    assert sized.map(4) == 8
    assert sized.map(10) == 20


@pytest.mark.parametrize("dtype", ["int32", "float32", "float64"])
@pytest.mark.parametrize("cuts_dtype", ["float32", "float64"])
def test_pairwise_predict_output_domain(dtype, cuts_dtype):
    np = pytest.importorskip("numpy")
    trans = make_pairwise_predict(
        dp.numpy.array2_domain(num_columns=2, T=np.dtype(dtype).type),
        dp.symmetric_distance(),
        np.array([-1.0, 1.0], dtype=cuts_dtype),
    )
    data = np.array([[0, 0], [1, 1], [2, 2], [3, 3]], dtype=dtype)
    assert trans.input_domain.member(data)

    output = trans(data)
    assert output.dtype == np.float64
    assert trans.output_domain.member(output)


@pytest.mark.parametrize("T", ["bool", "String"])
def test_pairwise_predict_rejects_nonnumeric(T):
    np = pytest.importorskip("numpy")
    with pytest.raises(ValueError, match="numeric"):
        make_pairwise_predict(
            dp.numpy.array2_domain(num_columns=2, T=T),
            dp.symmetric_distance(),
            np.array([-1.0, 1.0]),
        )


def test_pairwise_prediction_keeps_features_and_targets_aligned(monkeypatch):
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
        pairwise_predict,
    )

    data = dp.sklearn.SklearnFit(
        X=np.array([[0.0], [1.0], [3.0], [6.0]]),
        y=np.array([0.0, 1.0, 9.0, 36.0]),
    )

    def reverse(values):
        values[:] = values[::-1]

    monkeypatch.setattr(np.random, "shuffle", reverse)
    assert np.allclose(
        pairwise_predict(data, np.array([0.0, 1.0])),
        np.array([[-6.0, 1.0], [0.0, 3.0]]),
    )


def test_theil_sen_estimator_context_fit_and_methods():
    pytest.importorskip("numpy")
    sklearn = pytest.importorskip("sklearn")
    assert sklearn.base.is_regressor(_estimator())

    data, domain = _training_data()
    context = dp.Context.compositor(
        data=data,
        domain=domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    estimator = _estimator()

    assert not hasattr(estimator, "coef_")
    template = estimator
    estimator = context.query().sklearn(template).release()
    assert estimator is not template
    assert not hasattr(template, "coef_")
    assert estimator.coef_.shape == (1,)
    assert isinstance(estimator.intercept_, float)
    assert estimator.n_features_in_ == 1
    assert estimator.predict([[1.0], [2.0]]).shape == (2,)
    assert estimator.score([[1.0], [2.0]], [3.0, 5.0]) <= 1.0


def test_theil_sen_fit_and_query_sklearn_release_are_equivalent():
    pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    data, domain = _training_data()
    release_context = dp.Context.compositor(
        data=data,
        domain=domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    fit_estimator, release_estimator = _estimator(), _estimator()

    assert fit_estimator.fit(
        data.X, data.y, domain=domain, privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    ) is fit_estimator

    initial_state = release_estimator.__dict__.copy()
    fitted_query = release_context.query().sklearn(release_estimator)
    assert release_estimator.__dict__ == initial_state
    fitted = fitted_query.release()
    assert fitted is not release_estimator
    assert not hasattr(release_estimator, "coef_")
    assert fit_estimator.coef_.shape == fitted.coef_.shape == (1,)
    assert isinstance(fitted.intercept_, float)


def test_pca_transform_retains_private_targets_for_theil_sen():
    np = pytest.importorskip("numpy")
    data, original_domain = _training_data()
    X = np.column_stack([data.X[:, 0], np.ones(len(data.X))])
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, size=len(X), num_columns=2),
        original_domain.cast(dp.sklearn.SklearnFitDomain).y,
    )
    inputs = dp.sklearn.SklearnFit(X.copy(), data.y)
    context = dp.Context.compositor(
        inputs, dp.unit_of(contributions=1), dp.loss_of(epsilon=2.0),
        domain=domain, split_evenly_over=2,
    )
    pca = dp.sklearn.decomposition.PCA(n_components=1, row_norm=1.0)
    pca = context.query().sklearn(pca).release()
    transform = pca.make_transform(domain)
    assert transform(inputs).y is inputs.y
    estimator = dp.sklearn.linear_model.TheilSenRegressor(
        x_bounds=((-30.0, 30.0),), y_bounds=(-10.0, 60.0),
    )
    next_query = context.query().new_with(chain=transform)
    fitted = next_query.sklearn(estimator).release()
    assert fitted is not estimator
    assert fitted.coef_.shape == (1,)
    np.testing.assert_array_equal(inputs.X, X)


@pytest.mark.parametrize("domain_kind", ["infer", "X", "fit"])
def test_theil_sen_standalone_and_serialization(domain_kind):
    import pickle
    import numpy as np
    from sklearn.base import clone

    data, fit_domain = _training_data()
    domain = {"infer": None, "X": fit_domain.descriptor.X, "fit": fit_domain}[domain_kind]
    estimator = _estimator()
    assert estimator.fit(
        data.X, data.y, domain=domain, privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    ) is estimator
    restored = pickle.loads(pickle.dumps(estimator))
    np.testing.assert_allclose(restored.predict(data.X), estimator.predict(data.X))
    assert clone(estimator).get_params() == estimator.get_params()
    assert not hasattr(clone(estimator), "coef_")


def test_theil_sen_sklearn_fit_constructs_private_targets():
    import numpy as np

    data, _ = _training_data()
    rows = np.column_stack([data.X[:, 0], data.y])
    domain = dp.numpy.array2_domain(T=float, num_columns=2, size=len(rows), nan=False)
    context = dp.Context.compositor(
        rows, dp.unit_of(contributions=1), dp.loss_of(epsilon=1),
        domain=domain, split_evenly_over=1,
    )
    template = _estimator()
    query = context.query().sklearn_fit(X=[0], y=1).sklearn(template)
    assert context.current_privacy_loss() == []
    assert not hasattr(template, "coef_")
    fitted = query.release()
    assert fitted.coef_.shape == (1,)
    assert not hasattr(template, "coef_")
    assert context.current_privacy_loss() == [1.0]


def test_theil_sen_framework_make_is_non_mutating():
    np = pytest.importorskip("numpy")
    data, domain = _training_data()
    estimator = _estimator()
    initial_state = estimator.__dict__.copy()

    measurement = estimator.make(
        domain, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
    )
    assert measurement.map(1) <= 1.0
    assert isinstance(
        measurement.input_domain.cast(dp.sklearn.SklearnFitDomain), dp.sklearn.SklearnFitDomain
    )
    assert estimator.__dict__ == initial_state
    slope, intercept = measurement(data)
    assert estimator.__dict__ == initial_state
    assert np.asarray(slope).shape == ()
    assert np.asarray(intercept).shape == ()


def test_theil_sen_fitted_methods_and_error_paths():
    np = pytest.importorskip("numpy")
    estimator = _estimator()
    with pytest.raises(ValueError, match="not fitted"):
        estimator.predict([[1.0]])

    estimator._ingest_release((2.0, 1.0))
    assert np.array_equal(estimator.coef_, np.array([2.0]))
    assert estimator.intercept_ == 1.0
    assert estimator.n_features_in_ == 1
    assert np.array_equal(estimator.predict([1.0, 2.0]), np.array([3.0, 5.0]))
    assert estimator.score([[1.0], [2.0]], [3.0, 5.0]) == 1.0
    with pytest.raises(NotImplementedError, match="sample_weight"):
        estimator.score([[1.0]], [3.0], sample_weight=[1.0])
    with pytest.raises(ValueError, match="shape"):
        estimator.predict([[1.0, 2.0]])
    with pytest.raises(ValueError, match="one-dimensional"):
        estimator.score([[1.0]], [[3.0]])


def test_theil_sen_constructor_and_aligned_domain_validation():
    np = pytest.importorskip("numpy")
    pytest.importorskip("sklearn")
    from sklearn.base import clone

    bounds = [(-3.0, 3.0)]
    estimator = dp.sklearn.linear_model.TheilSenRegressor(
        x_bounds=bounds,
        y_bounds=(-10.0, 10.0),
    )
    assert clone(estimator).x_bounds is not bounds
    assert estimator.get_params()["x_bounds"] == bounds
    assert estimator.set_params(runs=2) is estimator

    with pytest.raises(ValueError, match="SklearnFit input domain"):
        estimator.make(
            dp.numpy.array2_domain(num_columns=2, size=10, T=float),
            dp.symmetric_distance(),
            dp.max_divergence(),
            1,
            1.0,
        )

    X_domain = dp.numpy.array2_domain(num_columns=1, size=10, T=float)
    without_y = dp.sklearn.sklearn_fit_domain(X=X_domain)
    with pytest.raises(ValueError, match="with y"):
        estimator.make(
            without_y, dp.symmetric_distance(), dp.max_divergence(), 1, 1.0
        )

    mismatched = dp.sklearn.sklearn_fit_domain(
        X=X_domain, y=dp.vector_domain(dp.atom_domain(T=float), size=9),
    )
    assert not mismatched.member(dp.sklearn.SklearnFit(np.zeros((10, 1)), np.zeros(9)))


def test_theil_sen_scale_is_not_bounded_by_response_range():
    pytest.importorskip("numpy")
    from opendp.extras.sklearn.linear_model import make_private_theil_sen

    _, domain = _training_data()
    measurement = make_private_theil_sen(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        d_in=1,
        d_out=1e-6,
        x_bounds=((-3.0, 3.0),),
        y_bounds=(-1e-3, 1e-3),
    )
    assert measurement.map(1) <= 1e-6


def test_theil_sen_rejects_separate_targets_and_weights_before_release(monkeypatch):
    pytest.importorskip("sklearn")
    from opendp.context import Query

    _, domain = _training_data(size=10)
    query = dp.Query(
        (domain, dp.symmetric_distance()), dp.max_divergence(), d_in=1, d_out=1.0
    )
    estimator = _estimator()
    released = []
    monkeypatch.setattr(Query, "release", lambda self: released.append(True))

    with pytest.raises(TypeError, match="not a Query"):
        estimator.fit(query, [1.0] * 10)
    with pytest.raises(TypeError, match="not a Query"):
        estimator.fit(query, sample_weight=[1.0] * 10)
    assert released == []


def test_theil_sen_fit_rejects_aligned_input_without_target_before_release(monkeypatch):
    pytest.importorskip("sklearn")
    from opendp.context import Query

    X_domain = dp.numpy.array2_domain(num_columns=1, size=10, T=float)
    query = dp.Query(
        (dp.sklearn.sklearn_fit_domain(X=X_domain), dp.symmetric_distance()),
        dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    released = []
    monkeypatch.setattr(Query, "release", lambda self: released.append(True))

    with pytest.raises(ValueError, match="with y"):
        query.sklearn(_estimator())
    assert released == []
