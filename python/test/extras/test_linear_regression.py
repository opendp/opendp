import pytest
import opendp.prelude as dp
from opendp._lib import import_optional_dependency


def test_private_theil_sen_measurement():
    np = pytest.importorskip("numpy")
    from opendp.extras.sklearn.linear_model import make_private_theil_sen

    domain = dp.numpy.array2_domain(num_columns=2, size=20, T=float)
    measurement = make_private_theil_sen(
        domain,
        dp.symmetric_distance(),
        dp.max_divergence(),
        d_in=1,
        d_out=1.0,
        x_bounds=((-3.0, 3.0),),
        y_bounds=(-10.0, 10.0),
    )
    assert measurement.map(1) <= 1.0
    slope, intercept = measurement(
        np.column_stack([np.arange(20, dtype=float), np.arange(20, dtype=float)])
    )
    assert np.asarray(slope).shape == ()
    assert np.asarray(intercept).shape == ()


def test_theil_sen_estimator_context_fit_and_methods():
    np = pytest.importorskip("numpy")
    sklearn = pytest.importorskip("sklearn")
    assert sklearn.base.is_regressor(
        dp.sklearn.linear_model.TheilSenRegressor(
            x_bounds=((-3.0, 3.0),), y_bounds=(-10.0, 60.0)
        )
    )

    training = np.column_stack(
        [np.arange(20, dtype=float), 2.0 * np.arange(20, dtype=float) + 1.0]
    )
    context = dp.Context.compositor(
        data=training,
        domain=dp.numpy.array2_domain(num_columns=2, size=20, T=float),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    estimator = dp.sklearn.linear_model.TheilSenRegressor(
        x_bounds=((-3.0, 25.0),),
        y_bounds=(-10.0, 60.0),
    )

    assert not hasattr(estimator, "coef_")
    assert estimator.fit(context.query()) is estimator  # type: ignore[arg-type]
    assert estimator.coef_.shape == (1,)
    assert isinstance(estimator.intercept_, float)
    assert estimator.predict([[1.0], [2.0]]).shape == (2,)
    assert estimator.score([[1.0], [2.0]], [3.0, 5.0]) <= 1.0


def test_theil_sen_fitted_methods_and_error_paths():
    np = pytest.importorskip("numpy")
    estimator = dp.sklearn.linear_model.TheilSenRegressor(
        x_bounds=((-3.0, 3.0),), y_bounds=(-10.0, 10.0)
    )
    with pytest.raises(ValueError, match="not fitted"):
        estimator.predict([[1.0]])

    estimator._ingest_release((2.0, 1.0))
    assert np.array_equal(estimator.coef_, np.array([2.0]))
    assert estimator.intercept_ == 1.0
    assert np.array_equal(estimator.predict([1.0, 2.0]), np.array([3.0, 5.0]))
    assert estimator.score([[1.0], [2.0]], [3.0, 5.0]) == 1.0
    with pytest.raises(NotImplementedError, match="sample_weight"):
        estimator.score([[1.0]], [3.0], sample_weight=[1.0])
    with pytest.raises(ValueError, match="shape"):
        estimator.predict([[1.0, 2.0]])
    with pytest.raises(ValueError, match="one-dimensional"):
        estimator.score([[1.0]], [[3.0]])


def test_theil_sen_constructor_and_fit_validation():
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

    with pytest.raises(ValueError, match="two-column"):
        estimator.make(
            dp.numpy.array2_domain(num_columns=1, size=10, T=float),
            dp.symmetric_distance(),
            dp.max_divergence(),
            1,
            1.0,
        )


def test_theil_sen_scale_is_not_bounded_by_response_range():
    pytest.importorskip("numpy")
    from opendp.extras.sklearn.linear_model import make_private_theil_sen

    domain = dp.numpy.array2_domain(num_columns=2, size=20, T=float)
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


def test_theil_sen_rejects_separate_targets_and_weights():
    pytest.importorskip("sklearn")
    estimator = dp.sklearn.linear_model.TheilSenRegressor(
        x_bounds=((-3.0, 3.0),), y_bounds=(-10.0, 10.0)
    )
    query = dp.Query(
        (
            dp.numpy.array2_domain(num_columns=2, size=10, T=float),
            dp.symmetric_distance(),
        ),
        dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    with pytest.raises(TypeError, match="second column"):
        estimator.fit(query, [1.0] * 10)
    with pytest.raises(TypeError, match="Unexpected fit parameters"):
        estimator.fit(query, sample_weight=[1.0] * 10)
