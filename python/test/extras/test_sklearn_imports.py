import pytest


def test_logistic_regression_lazy_export():
    pytest.importorskip("sklearn")
    from sklearn.base import clone
    from opendp.extras.sklearn.linear_model import LogisticRegression
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression as implementation,
    )

    assert LogisticRegression is implementation
    model = LogisticRegression(n_iters=3)
    assert clone(model).get_params() == model.get_params()
