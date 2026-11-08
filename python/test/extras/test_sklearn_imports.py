import pytest


def test_spherical_framework_import_without_estimator_dependencies():
    from opendp.extras.sklearn.cluster import make_private_spherical_kmeans

    assert callable(make_private_spherical_kmeans)


def test_logistic_regression_lazy_export():
    pytest.importorskip("sklearn")
    from opendp.extras.sklearn.linear_model import LogisticRegression
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression as implementation,
    )

    assert LogisticRegression is implementation
