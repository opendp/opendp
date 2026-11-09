from types import SimpleNamespace
from unittest.mock import Mock

import pytest

pytest.importorskip("sklearn")
import numpy as np
import opendp.prelude as dp
from opendp.extras.sklearn.linear_model import _make_private_theil_sen as framework
from opendp.extras.sklearn.linear_model import TheilSenRegressor


def _training_data(size=20, sized=True):
    X = np.arange(size, dtype=float).reshape(-1, 1)
    y = 2.0 * X[:, 0] + 1.0
    domain = dp.sklearn.sklearn_fit_domain(
        X=dp.numpy.array2_domain(num_columns=1, size=size if sized else None, T=float),
        y=dp.vector_domain(dp.atom_domain(T=float), size=size if sized else None),
    )
    return dp.sklearn.SklearnFit(X=X, y=y), domain


def _estimator(**params):
    return TheilSenRegressor(
        **(dict(slope_bounds=(-5.0, 5.0), intercept_bounds=(-10.0, 10.0)) | params)
    )


def _make(domain, d_in=2, d_out=1.0, measure=None, **params):
    return _estimator(**params).make(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence() if measure is None else measure,
        d_in,
        d_out,
    )


@pytest.fixture
def pairing_rng(monkeypatch):
    rng = SimpleNamespace(shuffle=lambda _: None)
    monkeypatch.setattr(framework, "get_np_csprng", lambda: rng)
    return rng


@pytest.mark.parametrize(
    "sized, d_in, measure",
    [
        (False, 1, dp.max_divergence()),
        (True, 1, dp.max_divergence()),
        (True, 2, dp.zero_concentrated_divergence()),
    ],
)
def test_private_theil_sen_measurement_consumes_aligned_data(sized, d_in, measure):
    data, domain = _training_data(sized=sized)
    # Include the true coefficients in the candidate grids. Check recovery, not an
    # exact noisy draw; very large budgets make failures overwhelmingly unlikely.
    measurement = _make(
        domain,
        d_in,
        1000.0,
        measure,
        slope_bounds=(-7.0, 2.0),
        intercept_bounds=(-8.0, 1.0),
    )
    assert measurement.input_domain == domain
    assert measurement.check(d_in, 1000.0)
    if sized:
        assert measurement.check(2, 1000.0)  # meaningful even when d_in=1 is vacuous
    np.testing.assert_allclose(measurement(data), [2.0, 1.0], atol=0.2, rtol=0)


def test_pairwise_slopes_stability():
    for sized in (False, True):
        _, domain = _training_data(5, sized)
        transform = framework.make_pairwise_slopes(domain, dp.symmetric_distance())
        for distance in [0, 1, 2, 3, 4, 10]:
            expected = 2 * (distance // 2) if sized else 2 * distance
            assert transform.map(distance) == expected


@pytest.mark.parametrize("dtype", ["int32", "float32", "float64"])
def test_pairwise_slopes_output_domain(dtype):
    target_dtype = "int32" if dtype == "int32" else "float64"
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(num_columns=1, T=np.dtype(dtype).type),
        dp.vector_domain(dp.atom_domain(T=np.dtype(target_dtype).type)),
    )
    transform = framework.make_pairwise_slopes(domain, dp.symmetric_distance())
    data = dp.sklearn.SklearnFit(
        np.arange(4, dtype=dtype).reshape(-1, 1),
        np.arange(4, dtype=target_dtype),
    )
    assert transform.input_domain.member(data)
    output = transform(data)
    assert np.asarray(output).dtype == np.float64
    assert transform.output_domain.member(output)


def test_pairwise_slopes_keeps_features_and_targets_aligned(pairing_rng):
    data = dp.sklearn.SklearnFit(
        X=np.array([[0.0], [1.0], [3.0], [6.0]]),
        y=np.array([0.0, 1.0, 9.0, 36.0]),
    )
    pairing_rng.shuffle = lambda indices: indices.__setitem__(
        slice(None), indices[::-1]
    )
    np.testing.assert_array_equal(framework.pairwise_slopes(data), [7.0, 3.0])


@pytest.mark.parametrize(
    "X, y, expected",
    [
        ([], [], []),
        ([0], [1], []),
        ([0, 1, 0, 1], [0, 3, 1, 7], []),
        ([0, 1, 0, 3], [0, 3, 1, 7], [2.0]),
        (
            [0, 0, 1, np.finfo(float).smallest_subnormal],
            [0, 0, np.inf, np.finfo(float).max],
            [],
        ),
        ([0, np.nan, np.inf, 3], [np.nan, 0, 1, 7], []),
    ],
)
def test_pairwise_slopes_empty_vertical_and_nonfinite(X, y, expected, pairing_rng):
    data = dp.sklearn.SklearnFit(np.array(X, dtype=float).reshape(-1, 1), np.array(y))
    with np.errstate(all="raise"):
        result = framework.pairwise_slopes(data)
    assert result.dtype == np.float64 and result.ndim == 1
    np.testing.assert_array_equal(result, expected)


def test_pairing_uses_opendp_csprng(pairing_rng):
    pairing_rng.shuffle = Mock(side_effect=np.random.default_rng(0).shuffle)
    data, _ = _training_data(9)
    np.testing.assert_array_equal(framework.pairwise_slopes(data), np.full(4, 2.0))
    pairing_rng.shuffle.assert_called_once()


@pytest.mark.parametrize("size", [3, 4])
def test_uniform_matching_and_row_permutation_invariance(size, pairing_rng):
    from collections import Counter
    from itertools import permutations

    X = np.array([[0.0], [1.0], [3.0], [6.0]])[:size]
    y = X[:, 0] ** 2
    distributions = []
    for row_order in permutations(range(size)):
        data = dp.sklearn.SklearnFit(X[list(row_order)], y[list(row_order)])
        outcomes = []
        for shuffle_order in permutations(range(size)):

            def shuffle(indices):
                assert len(indices) == size
                indices[:] = shuffle_order

            pairing_rng.shuffle = shuffle
            outcomes.append(tuple(sorted(framework.pairwise_slopes(data))))
        distributions.append(Counter(outcomes))
    assert all(distribution == distributions[0] for distribution in distributions)
    assert len(distributions[0]) == 3
    assert set(distributions[0].values()) == {2 if size == 3 else 8}


def test_pairwise_variable_size_stability_allows_replacing_a_pair(pairing_rng):
    from collections import Counter

    _, domain = _training_data(sized=False)
    transform = framework.make_pairwise_slopes(domain, dp.symmetric_distance())
    assert transform.map(1) == 2 and transform.map(2) == 4
    pairing_rng.shuffle = lambda indices: indices.__setitem__(
        slice(None), indices[::-1]
    )
    # Regression for #2950: one insertion can replace a matched slope.
    before = dp.sklearn.SklearnFit(np.array([[0.0], [1.0]]), [0.0, 1.0])
    after = dp.sklearn.SklearnFit(np.array([[0.0], [1.0], [3.0]]), [0.0, 1.0, 9.0])
    old_rows, new_rows = Counter(transform(before)), Counter(transform(after))
    distance = sum((old_rows - new_rows).values()) + sum((new_rows - old_rows).values())
    assert distance == 2 == transform.map(1)


@pytest.mark.parametrize(
    "x_size, y_size", [(None, None), (5, None), (None, 5), (5, 5), (0, 0)]
)
def test_residual_stability_and_retention_of_known_size(x_size, y_size):
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, num_columns=1, size=x_size),
        dp.vector_domain(dp.atom_domain(T=float), size=y_size),
    )
    bounds = [-10.0, 10.0]
    transform = framework.make_residuals(domain, dp.symmetric_distance(), 2.0, bounds)
    assert transform.output_domain.size == (x_size if x_size is not None else y_size)
    assert transform.map(1) == 1 and transform.map(2) == 2
    bounds[:] = [-1000.0, 1000.0]  # construction snapshots public sanitization bounds
    largest = np.finfo(float).max
    X = np.array([[-largest], [largest], [np.nan], [0.0], [1.0]])
    y = np.array([largest, -largest, np.inf, np.nan, 50.0])
    if x_size == 0:
        X, y = X[:0], y[:0]
    data = dp.sklearn.SklearnFit(X, y)
    assert domain.member(data)
    with np.errstate(all="raise"):
        result = transform(data)
    expected = [] if x_size == 0 else [10.0, -10.0, -10.0, -10.0, 48.0]
    np.testing.assert_array_equal(result, expected)
    assert transform.output_domain.member(result)
    if len(X):
        X[0], y[0] = [1.0], 3.0
        np.testing.assert_array_equal(transform(data)[1:], result[1:])


@pytest.mark.parametrize(
    "sized, measure, loss",
    [
        (False, dp.max_divergence(), 1e-6),
        (True, dp.max_divergence(), 0.1),
        (False, dp.zero_concentrated_divergence(), 0.1),
        (True, dp.zero_concentrated_divergence(), 1.0000000000000002),
    ],
)
def test_adaptive_composition_checks_budgets_and_conditions_on_private_slope(
    sized, measure, loss, monkeypatch
):
    data, domain = _training_data(sized=sized)
    compositions, queries, slopes = [], [], []
    original_compose = dp.c.make_adaptive_composition
    original_postprocess = framework._new_pure_function
    original_residuals = framework.make_residuals

    def compose(*args):
        compositions.append(args)
        return original_compose(*args)

    def postprocess(function):
        def fit(queryable):
            def query(measurement):
                queries.append(measurement)
                return queryable(measurement)

            return function(query)

        return original_postprocess(fit)

    def residuals(domain, metric, slope, bounds):
        slopes.append(slope)
        return original_residuals(domain, metric, slope, bounds)

    monkeypatch.setattr(dp.c, "make_adaptive_composition", compose)
    monkeypatch.setattr(framework, "_new_pure_function", postprocess)
    monkeypatch.setattr(framework, "make_residuals", residuals)
    measurement = _make(domain, 1, loss, measure)
    assert measurement.check(1, loss)
    calibration_distance, allocations = compositions[-1][-2:]
    assert calibration_distance == (2 if sized else 1)
    assert allocations[0] == allocations[1] and allocations[0] > 0
    assert slopes == []
    slope, intercept = measurement(data)
    assert len(queries) == 2 and slopes == [slope]
    for query, allocation in zip(queries, allocations):
        assert query.input_domain == domain  # both queries run on original private rows
        assert query.check(calibration_distance, allocation)
    assert np.isfinite([slope, intercept]).all()
    if sized:
        assert measurement.check(2, loss)


@pytest.mark.parametrize("sized", [False, True])
@pytest.mark.parametrize("kind", ["empty", "one", "vertical", "nonfinite"])
def test_degenerate_private_fit_still_releases_two_candidates(sized, kind):
    size = 0 if kind == "empty" else 1 if kind == "one" else 4
    data, domain = _training_data(size, sized)
    if kind == "vertical":
        data.X[:] = 0.0
    if kind == "nonfinite":
        data.X[:] = np.inf
        data.y[:] = np.nan
    fitted = _estimator().fit(
        data.X,
        data.y,
        domain=domain,
        privacy_unit=dp.unit_of(contributions=2 if sized else 1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    )
    assert -5.0 <= fitted.coef_[0] <= 5.0
    assert -10.0 <= fitted.intercept_ <= 10.0


@pytest.mark.parametrize(
    "bounds",
    [
        None,
        [1],
        [1, 2, 3],
        [1, 1],
        [2, 1],
        [0, np.inf],
        [np.nan, 1],
        ["0", "1"],
        [0, 10**1000],
    ],
)
@pytest.mark.parametrize("name", ["slope_bounds", "intercept_bounds"])
def test_public_bounds_validation(name, bounds):
    with pytest.raises(ValueError, match=name):
        _estimator(**{name: bounds})
    _, domain = _training_data()
    model = _estimator().set_params(**{name: bounds})
    with pytest.raises(ValueError, match=name):
        model.make(domain, dp.symmetric_distance(), dp.max_divergence(), 2, 1.0)


def test_constructor_and_unsupported_domain_validation():
    import inspect
    from sklearn.base import clone

    import opendp.extras.sklearn.linear_model as linear_model

    assert set(linear_model.__all__) <= set(dir(linear_model))
    parameters = inspect.signature(TheilSenRegressor).parameters
    assert list(parameters) == ["slope_bounds", "intercept_bounds"]
    assert all(
        p.kind == inspect.Parameter.KEYWORD_ONLY
        and p.default is inspect.Parameter.empty
        for p in parameters.values()
    )
    bounds = [-5.0, 5.0]
    model = _estimator(slope_bounds=bounds)
    assert clone(model).slope_bounds is not bounds
    assert model.get_params() == dict(
        slope_bounds=bounds, intercept_bounds=(-10.0, 10.0)
    )
    X = dp.numpy.array2_domain(T=float, num_columns=1, size=3)
    y = dp.vector_domain(dp.atom_domain(T=float), size=3)
    cases = [
        (X, "SklearnFit input domain"),
        (dp.sklearn.sklearn_fit_domain(X), "with y"),
        (
            dp.sklearn.sklearn_fit_domain(
                dp.numpy.array2_domain(T=float, num_columns=2), y
            ),
            "one feature",
        ),
        (dp.sklearn.sklearn_fit_domain(X, X), "vector target"),
        (
            dp.sklearn.sklearn_fit_domain(X, dp.vector_domain(dp.atom_domain(T=str))),
            "numeric",
        ),
        (
            dp.sklearn.sklearn_fit_domain(
                dp.numpy.array2_domain(T=bool, num_columns=1), y
            ),
            "numeric",
        ),
        (
            dp.sklearn.sklearn_fit_domain(
                X, dp.vector_domain(dp.atom_domain(T=float), size=2)
            ),
            "matching row counts",
        ),
    ]
    for domain, error in cases:
        with pytest.raises(ValueError, match=error):
            _make(domain)
    _, domain = _training_data()
    with pytest.raises(ValueError, match="symmetric_distance"):
        model.make(domain, dp.insert_delete_distance(), dp.max_divergence(), 2, 1.0)
    with pytest.raises(ValueError, match="output_measure"):
        _make(domain, measure=dp.smoothed_max_divergence())
    for distance in [0, -1, True, 1.5]:
        with pytest.raises(ValueError, match="d_in"):
            _make(domain, d_in=distance)
    for loss in [0.0, -1.0, np.nan, np.inf]:
        with pytest.raises(ValueError, match="d_out"):
            _make(domain, d_out=loss)
    with pytest.raises(ValueError, match="too small"):
        _make(domain, d_out=np.finfo(float).smallest_subnormal)


@pytest.mark.parametrize(
    "bounds",
    [
        (1.0, np.nextafter(1.0, np.inf)),
        (0.0, np.finfo(float).smallest_subnormal),
        (-np.finfo(float).smallest_subnormal, np.finfo(float).smallest_subnormal),
        (-np.finfo(float).max, np.finfo(float).max),
        (np.finfo(float).max / 2, np.finfo(float).max),
    ],
)
def test_candidate_grids_are_finite_strict_and_retain_endpoints(bounds):
    data, domain = _training_data()
    with np.errstate(all="raise"):
        grid = framework._candidate_grid(bounds)
        measurement = _make(domain, slope_bounds=bounds, intercept_bounds=bounds)
        release = measurement(data)
    assert len(grid) >= 2 and np.isfinite(grid).all()
    assert np.all(grid[1:] > grid[:-1])
    np.testing.assert_array_equal(grid[[0, -1]], bounds)
    assert ((bounds[0] <= grid) & (grid <= bounds[1])).all()
    assert np.isin(release, grid).all() and measurement.check(2, 1.0)


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_theil_sen_fitting_cloning_scoring_and_refitting(route):
    import pickle
    from sklearn.base import clone, is_regressor, RegressorMixin
    from sklearn.metrics import r2_score

    data, domain = _training_data()
    model = _estimator()
    assert is_regressor(model) and type(model).score is RegressorMixin.score
    from sklearn.exceptions import NotFittedError

    with pytest.raises(NotFittedError):
        model.predict([[1.0]])
    context = dp.Context.compositor(
        data,
        dp.unit_of(contributions=2),
        dp.loss_of(epsilon=2.0),
        domain=domain,
        split_evenly_over=2,
    )
    for _ in range(2):
        previous = model
        previous_state = previous.__dict__.copy()
        old_coef = getattr(previous, "coef_", None)
        if route == "context":
            model = context.query().sklearn(previous).release()
            assert model is not previous and previous.__dict__ == previous_state
        else:
            model = model.fit(
                data.X,
                data.y,
                privacy_unit=dp.unit_of(contributions=1),
                privacy_loss=dp.loss_of(epsilon=1.0),
            )
            assert model is previous
        assert model.coef_.shape == (1,) and isinstance(model.intercept_, float)
        assert model.n_features_in_ == 1 and np.isfinite(model.predict(data.X)).all()
        assert model.coef_ is not old_coef
        if route == "standalone":
            restored = pickle.loads(pickle.dumps(model))
            np.testing.assert_array_equal(
                restored.predict(data.X), model.predict(data.X)
            )
        assert clone(model).get_params() == model.get_params()
        assert not hasattr(clone(model), "coef_")
    if route == "context":
        assert context.current_privacy_loss() == [1.0, 1.0]
    model._ingest_release((2.0, 1.0))
    np.testing.assert_array_equal(model.predict([1.0, 2.0]), [3.0, 5.0])
    X, y, weights = [[1.0], [2.0], [3.0]], [3.0, 4.0, 8.0], [1.0, 2.0, 3.0]
    assert model.score(X, y, sample_weight=weights) == r2_score(
        y, model.predict(X), sample_weight=weights
    )
    with pytest.raises(ValueError, match="shape"):
        model.predict([[1.0, 2.0]])


def test_pca_transform_retains_private_targets_for_theil_sen():
    from opendp.extras.sklearn.decomposition import PCA

    data, original_domain = _training_data()
    X = np.column_stack([data.X[:, 0], np.ones(len(data.X))])
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=float, size=len(X), num_columns=2),
        original_domain.cast(dp.sklearn.SklearnFitDescriptor).y,
    )
    inputs = dp.sklearn.SklearnFit(X.copy(), data.y)
    context = dp.Context.compositor(
        inputs,
        dp.unit_of(contributions=2),
        dp.loss_of(epsilon=2.0),
        domain=domain,
        split_evenly_over=2,
    )
    pca = context.query().sklearn(PCA(n_components=1, clip_norm=1.0)).release()
    transform = pca.make_transform(domain, dp.symmetric_distance())
    assert transform(inputs).y is inputs.y
    estimator = _estimator()
    fitted_query = context.query().new_with(chain=transform).sklearn(estimator)
    assert fitted_query.resolve().map(2) <= 1.0
    fitted = fitted_query.release()
    assert fitted is not estimator and not hasattr(estimator, "coef_")
    assert fitted.coef_.shape == (1,) and context.current_privacy_loss() == [1.0, 1.0]
    np.testing.assert_array_equal(inputs.X, X)


def test_theil_sen_query_snapshots_configuration_and_is_lazy(monkeypatch):
    data, domain = _training_data()
    slope_bounds, intercept_bounds = np.array([-5.0, 5.0]), [-10.0, 10.0]
    model = _estimator(slope_bounds=slope_bounds, intercept_bounds=intercept_bounds)
    model._ingest_release((2.0, 1.0))
    old_coef = model.coef_
    context = dp.Context.compositor(
        data,
        dp.unit_of(contributions=2),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )
    calls = []
    original = framework.pairwise_slopes

    def slopes(data):
        calls.append(True)
        return original(data)

    monkeypatch.setattr(framework, "pairwise_slopes", slopes)
    query = context.query().sklearn(model)
    query.resolve()
    assert calls == [] and context.current_privacy_loss() == []
    slope_bounds[:] = [1000.0, 2000.0]
    intercept_bounds[:] = [10000.0, 20000.0]
    fitted = query.release()
    assert fitted is not model and fitted.coef_ is not old_coef
    assert len(calls) == 1 and context.current_privacy_loss() == [1.0]
    assert model.coef_ is old_coef and model.intercept_ == 1.0
    assert -5.0 <= fitted.coef_[0] <= 5.0 and -10.0 <= fitted.intercept_ <= 10.0
    state = fitted.coef_.copy(), fitted.intercept_
    with pytest.raises(dp.OpenDPException, match="out of queries"):
        query.release()
    np.testing.assert_array_equal(fitted.coef_, state[0])
    assert fitted.intercept_ == state[1] and len(calls) == 1


def test_theil_sen_fit_requires_privacy_and_rejects_sample_weight(monkeypatch):
    from opendp.context import Query

    data, domain = _training_data(10)
    estimator = _estimator()
    released = []
    monkeypatch.setattr(Query, "release", lambda self: released.append(True))
    with pytest.raises(TypeError, match="privacy_unit.*privacy_loss"):
        estimator.fit(data.X, data.y)
    with pytest.raises(TypeError, match="sample_weight"):
        estimator.fit(
            data.X,
            data.y,
            sample_weight=np.ones(10),
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
        )
    query = dp.Query(
        (
            dp.sklearn.sklearn_fit_domain(
                X=domain.cast(dp.sklearn.SklearnFitDescriptor).X
            ),
            dp.symmetric_distance(),
        ),
        dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    with pytest.raises(ValueError, match="with y"):
        query.sklearn(estimator)
    assert released == []
