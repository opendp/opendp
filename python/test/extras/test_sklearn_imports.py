import pytest


def test_sgd_classifier_lazy_export():
    pytest.importorskip("sklearn")
    from opendp.extras.sklearn.linear_model import SGDClassifier
    from opendp.extras.sklearn.linear_model._sgd_classifier import (
        SGDClassifier as implementation,
    )

    assert SGDClassifier is implementation


def test_objective_logistic_regression_lazy_export():
    pytest.importorskip("sklearn")
    from opendp.extras.sklearn.linear_model import LogisticRegression
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression as implementation,
    )
    assert LogisticRegression is implementation
