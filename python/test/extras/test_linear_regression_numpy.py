import re

import pytest

from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_pairwise_predict,
    make_private_theil_sen,
)
import opendp.prelude as dp

np = pytest.importorskip("numpy")


def test_private_theil_sen():
    x_bounds = -3, 3
    y_bounds = -10, 10
    input_domain = dp.numpy.array2_domain(num_columns=2, T=float)
    input_metric = dp.symmetric_distance()
    meas = make_private_theil_sen(
        input_domain,
        input_metric,
        dp.max_divergence(),
        x_bounds,
        y_bounds,
        scale=1.0,
    )
    assert meas.map(1) == 4

    def f(x):
        return x * 2 + 1

    x = np.random.normal(size=100, loc=0, scale=1.0)
    y = f(x) + np.random.normal(size=100, loc=0, scale=0.5)

    slope, intercept = meas(np.stack([x, y], axis=1))
    # There is a non-zero chance these may fail.
    assert 1.5 < slope < 2.5
    assert 0.5 < intercept < 1.5


def test_pairwise_predict_stability():
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


@pytest.mark.parametrize("dtype", [np.int32, np.float32, np.float64])
@pytest.mark.parametrize("cuts_dtype", [np.float32, np.float64])
def test_pairwise_predict_output_domain(dtype, cuts_dtype):
    trans = make_pairwise_predict(
        dp.numpy.array2_domain(num_columns=2, T=dtype),
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
    with pytest.raises(ValueError, match="numeric"):
        make_pairwise_predict(
            dp.numpy.array2_domain(num_columns=2, T=T),
            dp.symmetric_distance(),
            np.array([-1.0, 1.0]),
        )


def test_input_validation():
    msg = "For now, the x_bounds array must consist of a single tuple, not [0, 10]"
    with pytest.raises(Exception, match=re.escape(msg)):
        dp.sklearn.linear_model.LinearRegression(
            output_measure=dp.max_divergence(),
            x_bounds=(0, 10),  # type: ignore[arg-type]
            y_bounds=(0, 10),
            scale=1,
        ).fit(
            X=[[1], [2], [3], [4], [5]],
            y=[1, 2, 3, 4, 5],
        )
