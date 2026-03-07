import re

import pytest

from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_private_theil_sen,
)
import opendp.prelude as dp


def test_private_theil_sen():
    np = pytest.importorskip("numpy")
    x_bounds = -3, 3
    y_bounds = -10, 10
    meas = make_private_theil_sen(dp.max_divergence(), x_bounds, y_bounds, scale=1.0)
    assert meas.map(1) == 2

    def f(x):
        return x * 2 + 1

    x = np.random.normal(size=100, loc=0, scale=1.0)
    y = f(x) + np.random.normal(size=100, loc=0, scale=0.5)

    slope, intercept = meas(np.stack([x, y], axis=1))
    # There is a non-zero chance these may fail.
    assert 1.5 < slope < 2.5
    assert 0.5 < intercept < 1.5


def test_input_validation():
    pytest.importorskip("numpy")
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
