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
    return _estimator(**params).make_measurement(
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
        (True, 2, dp.max_divergence()),
        (True, 2, dp.zero_concentrated_divergence()),
    ],
)
def test_private_theil_sen_measurement_consumes_aligned_data(sized, d_in, measure):
    data, domain = _training_data(sized=sized)
    # Grid includes the true coefficients; the privacy budget is large.
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
    # Regression for #2950.
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
    bounds[:] = [-1000.0, 1000.0]
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
    "sized, d_in, measure, loss",
    [
        (False, 1, dp.max_divergence(), 1e-6),
        (True, 2, dp.max_divergence(), 0.1),
        (False, 3, dp.zero_concentrated_divergence(), 0.1),
        (True, 4, dp.zero_concentrated_divergence(), 1.0000000000000002),
    ],
)
def test_adaptive_composition_checks_budgets_and_conditions_on_private_slope(
    sized, d_in, measure, loss, monkeypatch
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
    measurement = _make(domain, d_in, loss, measure)
    assert measurement.check(d_in, loss)
    calibration_distance, allocations = compositions[-1][-2:]
    assert calibration_distance == d_in
    assert allocations[0] == allocations[1] and allocations[0] > 0
    assert slopes == []
    slope, intercept = measurement(data)
    assert len(queries) == 2 and slopes == [slope]
    for query, allocation in zip(queries, allocations):
        assert query.input_domain == domain
        assert query.check(calibration_distance, allocation)
    assert np.isfinite([slope, intercept]).all()
    if sized:
        assert measurement.check(2, loss)


@pytest.mark.parametrize("sized", [False, True])
@pytest.mark.parametrize("kind", ["one", "vertical", "nonfinite"])
def test_degenerate_private_fit_still_releases_two_candidates(sized, kind):
    size = 1 if kind == "one" else 4
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
    model = _estimator(**{name: bounds})
    assert getattr(model, name) is bounds
    _, domain = _training_data()
    with pytest.raises(ValueError, match=name):
        _make(domain, **{name: bounds})


@pytest.mark.parametrize("d_in", [1, 3])
def test_fixed_size_rejects_odd_distances(d_in):
    _, domain = _training_data()
    with pytest.raises(ValueError, match="d_in must be even for fixed-size domains"):
        _make(domain, d_in=d_in)


def test_zero_size_rejected():
    _, domain = _training_data(0)
    with pytest.raises(ValueError, match="input size must be positive"):
        _make(domain)


def test_unsized_empty_fit():
    data, domain = _training_data(0, sized=False)
    fitted = _estimator().fit(
        data.X,
        data.y,
        domain=domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    )
    assert -5.0 <= fitted.coef_[0] <= 5.0
    assert -10.0 <= fitted.intercept_ <= 10.0


def test_unsupported_domain_and_public_parameter_validation():
    model = _estimator()
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
        model.make_measurement(
            domain, dp.insert_delete_distance(), dp.max_divergence(), 2, 1.0
        )
    with pytest.raises(ValueError, match="output_measure"):
        _make(domain, measure=dp.smoothed_max_divergence())
    for distance in [0, -1, True, 1.5]:
        with pytest.raises(ValueError, match="d_in"):
            _make(domain, d_in=distance)
    for loss in [0.0, -1.0, np.nan, np.inf, True]:
        with pytest.raises(ValueError, match="d_out"):
            _make(domain, d_out=loss)
    with pytest.raises(ValueError, match="too small"):
        _make(domain, d_out=np.finfo(float).smallest_subnormal)


@pytest.mark.parametrize(
    "bounds",
    [
        (1.0, np.nextafter(1.0, np.inf)),
        (0.0, np.finfo(float).smallest_subnormal),
    ],
)
def test_adjacent_candidate_bounds(bounds):
    data, domain = _training_data()
    measurement = _make(domain, slope_bounds=bounds, intercept_bounds=bounds)
    assert np.isin(measurement(data), bounds).all()
    assert measurement.check(2, 1.0)


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_theil_sen_fitting_and_public_scoring(route):
    from sklearn.base import is_regressor
    from sklearn.metrics import r2_score

    data, domain = _training_data()
    model = _estimator()
    if route == "context":
        context = dp.Context.compositor(
            data,
            dp.unit_of(contributions=2),
            dp.loss_of(epsilon=1.0),
            domain=domain,
            split_evenly_over=1,
        )
        fitted = context.query().sklearn(model).release()
        assert fitted is not model and not hasattr(model, "coef_")
        assert context.current_privacy_loss() == [1.0]
    else:
        fitted = model.fit(
            data.X,
            data.y,
            domain=domain,
            privacy_unit=dp.unit_of(contributions=2),
            privacy_loss=dp.loss_of(epsilon=1.0),
        )
        assert fitted is model

    assert is_regressor(fitted)
    assert fitted.coef_.shape == (1,)
    assert isinstance(fitted.intercept_, float)
    assert fitted.n_features_in_ == 1
    fitted._ingest_release((2.0, 1.0))
    np.testing.assert_array_equal(fitted.predict([[1.0], [2.0]]), [3.0, 5.0])
    X, y, weights = [[1.0], [2.0], [3.0]], [3.0, 4.0, 8.0], [1.0, 2.0, 3.0]
    assert fitted.score(X, y, sample_weight=weights) == r2_score(
        y, fitted.predict(X), sample_weight=weights
    )


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


def test_theil_sen_rowwise_prediction_preserves_targets_and_snapshots():
    model = _estimator()
    model._ingest_release((2.0, 1.0))
    X = np.array([[1.0], [2.0]], dtype=np.float64)
    y = np.array([0, 1], dtype=np.int32)
    feature_domain = dp.numpy.array2_domain(T=dp.f64, num_columns=1, nan=False)
    domain = dp.sklearn.sklearn_fit_domain(
        feature_domain,
        dp.vector_domain(dp.atom_domain(T=dp.i32)),
    )
    output = model.rowwise_output_domain(feature_domain, method="predict")
    assert output.cast(dp.numpy.NPArray2Domain).num_columns == 1

    from opendp.extras.sklearn._rowwise import _make_rowwise_transform

    trans = _make_rowwise_transform(
        domain, dp.symmetric_distance(), model, method="predict"
    )
    assert trans.map(1) == 1
    result = trans(dp.sklearn.SklearnFit(X, y))
    np.testing.assert_array_equal(result.X, [[3.0], [5.0]])
    np.testing.assert_array_equal(result.y, y)
    assert trans.output_domain.member(result)

    model.coef_[0] = 99.0
    model.intercept_ = 99.0
    np.testing.assert_array_equal(trans(dp.sklearn.SklearnFit(X, y)).X, [[3.0], [5.0]])
    with pytest.raises(ValueError, match="does not support rowwise inference"):
        model.rowwise_output_domain(feature_domain, method="transform")


def test_theil_sen_rowwise_rejects_bad_feature_domains():
    model = _estimator()
    model._ingest_release((2.0, 1.0))
    with pytest.raises(ValueError, match="feature count"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(T=dp.f64, num_columns=2), method="predict"
        )
    with pytest.raises(TypeError, match="numerical features"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(T=bool, num_columns=1), method="predict"
        )


def test_theil_sen_rowwise_handles_empty_and_nonfinite_inputs():
    from opendp.extras.sklearn._rowwise import _make_rowwise_transform

    model = _estimator()
    model._ingest_release((2.0, 1.0))
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=1, nan=True)
    )
    trans = _make_rowwise_transform(
        domain, dp.symmetric_distance(), model, method="predict"
    )
    for X in (
        np.empty((0, 1)),
        np.array([[np.inf], [-np.inf], [np.nan], [np.finfo(float).max]]),
    ):
        with np.errstate(all="raise"):
            result = trans(dp.sklearn.SklearnFit(X))
        assert result.X.shape == (len(X), 1)
        assert trans.output_domain.member(result)
