import pytest

pytest.importorskip("sklearn")
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
    assert measurement.map(1) == 0
    assert measurement.check(2, 1.0)
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
        dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(num_columns=1, T=float),
            dp.vector_domain(dp.atom_domain(T=float)),
        ),
        dp.symmetric_distance(),
        x_cuts,
        runs=2,
    )
    assert unsized.map(1) == 4
    assert unsized.map(2) == 8

    sized = make_pairwise_predict(
        dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(size=5, num_columns=1, T=float),
            dp.vector_domain(dp.atom_domain(T=float)),
        ),
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
    target_dtype = "int32" if dtype == "int32" else "float64"
    trans = make_pairwise_predict(
        dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(num_columns=1, T=np.dtype(dtype).type),
            dp.vector_domain(dp.atom_domain(T=np.dtype(target_dtype).type)),
        ),
        dp.symmetric_distance(),
        np.array([-1.0, 1.0], dtype=cuts_dtype),
    )
    data = dp.sklearn.SklearnFit(
        np.array([[0], [1], [2], [3]], dtype=dtype),
        np.array([0, 1, 2, 3], dtype=target_dtype),
    )
    assert trans.input_domain.member(data)

    output = trans(data)
    assert output.dtype == np.float64
    assert trans.output_domain.member(output)


@pytest.mark.parametrize("T", ["bool", "String"])
def test_pairwise_predict_rejects_nonnumeric(T):
    np = pytest.importorskip("numpy")
    with pytest.raises(ValueError, match="numeric"):
        make_pairwise_predict(
            dp.sklearn.sklearn_fit_domain(
                dp.numpy.array2_domain(num_columns=1, T=T),
                dp.vector_domain(dp.atom_domain(T=T)),
            ),
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


def test_pairwise_prediction_is_permutation_invariant_for_odd_rows(monkeypatch):
    from collections import Counter
    from itertools import permutations
    import numpy as np
    from opendp.extras.sklearn.linear_model._make_private_theil_sen import pairwise_predict

    X, y = np.array([[0.0], [1.0], [3.0]]), np.array([0.0, 1.0, 9.0])
    distributions = []
    for row_order in permutations(range(3)):
        data = dp.sklearn.SklearnFit(X[list(row_order)], y[list(row_order)])
        outcomes = []
        for shuffle_order in permutations(range(3)):
            def shuffle(indices):
                assert len(indices) == 3  # omit only after randomizing all rows
                indices[:] = shuffle_order
            monkeypatch.setattr(np.random, "shuffle", shuffle)
            outcomes.append(tuple(pairwise_predict(data, np.array([0.0, 1.0])).ravel()))
        distributions.append(Counter(outcomes))
    assert all(distribution == distributions[0] for distribution in distributions)
    assert len(distributions[0]) == 3


@pytest.mark.parametrize("runs", [1, 3])
def test_pairwise_variable_size_stability_allows_replacing_a_pair(runs, monkeypatch):
    from collections import Counter
    import numpy as np
    from opendp.extras.sklearn.linear_model._make_private_theil_sen import make_pairwise_predict

    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=1),
        dp.vector_domain(dp.atom_domain(T=float)),
    )
    transform = make_pairwise_predict(
        domain, dp.symmetric_distance(), np.array([0.0, 1.0]), runs=runs,
    )
    assert transform.map(1) == 2 * runs
    assert transform.map(2) == 4 * runs
    monkeypatch.setattr(np.random, "shuffle", lambda indices: indices.__setitem__(slice(None), indices[::-1]))
    before = dp.sklearn.SklearnFit(np.array([[0.0], [1.0]]), [0.0, 1.0])
    after = dp.sklearn.SklearnFit(np.array([[0.0], [1.0], [3.0]]), [0.0, 1.0, 9.0])
    old_rows = Counter(map(tuple, transform(before)))
    new_rows = Counter(map(tuple, transform(after)))
    distance = sum((old_rows - new_rows).values()) + sum((new_rows - old_rows).values())
    assert distance == 2 * runs == transform.map(1)


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
    assert estimator is template
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
    assert fitted is release_estimator
    assert fit_estimator.coef_.shape == fitted.coef_.shape == (1,)
    assert isinstance(fitted.intercept_, float)


def test_pca_transform_retains_private_targets_for_theil_sen():
    from opendp.extras.sklearn.decomposition import PCA

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
    pca = PCA(n_components=1, clip_norm=1.0)
    pca = context.query().sklearn(pca).release()
    transform = pca.make_transform(domain, dp.symmetric_distance())
    assert transform(inputs).y is inputs.y
    estimator = dp.sklearn.linear_model.TheilSenRegressor(
        x_bounds=((-30.0, 30.0),), y_bounds=(-10.0, 60.0),
    )
    next_query = context.query().new_with(chain=transform)
    fitted = next_query.sklearn(estimator).release()
    assert fitted is estimator
    assert fitted.coef_.shape == (1,)
    np.testing.assert_array_equal(inputs.X, X)


@pytest.mark.parametrize("domain_kind", ["infer", "fit"])
def test_theil_sen_standalone_and_serialization(domain_kind):
    import pickle
    import numpy as np
    from sklearn.base import clone

    data, fit_domain = _training_data()
    domain = fit_domain if domain_kind == "fit" else None
    estimator = _estimator()
    assert estimator.fit(
        data.X, data.y, domain=domain, privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    ) is estimator
    restored = pickle.loads(pickle.dumps(estimator))
    np.testing.assert_allclose(restored.predict(data.X), estimator.predict(data.X))
    assert clone(estimator).get_params() == estimator.get_params()
    assert not hasattr(clone(estimator), "coef_")


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


def test_theil_sen_fit_requires_privacy_and_rejects_unsupported_parameters(monkeypatch):
    pytest.importorskip("sklearn")
    from opendp.context import Query

    _, domain = _training_data(size=10)
    query = dp.Query(
        (domain, dp.symmetric_distance()), dp.max_divergence(), d_in=1, d_out=1.0
    )
    estimator = _estimator()
    released = []
    monkeypatch.setattr(Query, "release", lambda self: released.append(True))

    with pytest.raises(TypeError, match="explicit privacy_unit and privacy_loss"):
        estimator.fit(query, [1.0] * 10)
    with pytest.raises(TypeError, match="unexpected keyword argument 'sample_weight'"):
        estimator.fit(  # type: ignore[call-arg]
            [[1.0]] * 10, [1.0] * 10, sample_weight=[1.0] * 10,
            privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(epsilon=1),
        )
    assert released == []


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_theil_sen_refitting_replaces_learned_state(route):
    import numpy as np

    data, domain = _training_data()
    model = _estimator()
    context = dp.Context.compositor(
        data, dp.unit_of(contributions=1), dp.loss_of(epsilon=2),
        domain=domain, split_evenly_over=2,
    )

    def fit():
        if route == "context":
            return context.query().sklearn(model).release()
        return model.fit(data.X, data.y, domain=domain,
                         privacy_unit=dp.unit_of(contributions=1), privacy_loss=dp.loss_of(epsilon=1))

    assert fit() is model
    old_coef = model.coef_
    model.coef_[:] = 1e6
    model.intercept_ = -1e6
    assert fit() is model
    assert model.coef_ is not old_coef
    assert np.isfinite(model.predict(data.X)).all()
    assert abs(model.coef_[0]) <= 5.0 + 1e-10
    assert abs(model.intercept_) < 100
    if route == "context":
        assert context.current_privacy_loss() == [1.0, 1.0]


def test_theil_sen_query_snapshots_mutable_configuration_and_is_lazy(monkeypatch):
    import numpy as np
    from opendp.extras.sklearn.linear_model import _make_private_theil_sen as framework

    data, domain = _training_data()
    x_bounds, y_bounds, fractions = np.array([[-3.0, 25.0]]), [-10.0, 60.0], [0.25, 0.75]
    model = dp.sklearn.linear_model.TheilSenRegressor(
        x_bounds=x_bounds, y_bounds=y_bounds, fraction_bounds=fractions,
    )
    model._ingest_release((2.0, 1.0))
    old_coef = model.coef_
    context = dp.Context.compositor(
        data, dp.unit_of(contributions=1), dp.loss_of(epsilon=1),
        domain=domain, split_evenly_over=1,
    )
    cuts = []
    original = framework.pairwise_predict

    def predict(data, x_cuts):
        cuts.append(x_cuts.copy())
        return original(data, x_cuts)

    monkeypatch.setattr(framework, "pairwise_predict", predict)
    query = context.query().sklearn(model)
    query.resolve()
    assert cuts == [] and context.current_privacy_loss() == []
    assert model.coef_ is old_coef and model.intercept_ == 1.0
    x_bounds[:] = [[1000.0, 2000.0]]
    y_bounds[:] = [10000.0, 20000.0]
    fractions[:] = [0.1, 0.9]
    model.set_params(runs=3, candidates_count=2)
    assert query.release() is model
    assert len(cuts) == 1
    np.testing.assert_array_equal(cuts[0], [4.0, 18.0])
    predictions = model.predict([[4.0], [18.0]])
    assert np.all(predictions >= -10.0 - 1e-10) and np.all(predictions <= 60.0 + 1e-10)
    assert model.coef_ is not old_coef
    assert context.current_privacy_loss() == [1.0]
    state = model.coef_.copy(), model.intercept_
    with pytest.raises(dp.OpenDPException, match="out of queries"):
        query.release()
    np.testing.assert_array_equal(model.coef_, state[0])
    assert model.intercept_ == state[1] and len(cuts) == 1


def test_pairwise_constructor_snapshots_mutable_cuts(monkeypatch):
    import numpy as np
    from opendp.extras.sklearn.linear_model._make_private_theil_sen import make_pairwise_predict

    data, domain = _training_data()
    monkeypatch.setattr(np.random, "shuffle", lambda indices: None)
    cuts = np.array([4.0, 18.0])
    transform = make_pairwise_predict(domain, dp.symmetric_distance(), cuts)
    before = transform(data)
    cuts[:] = [1000.0, 2000.0]
    np.testing.assert_allclose(transform(data), before)
    assert transform.map(1) == 0
    assert transform.map(2) == 2


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
