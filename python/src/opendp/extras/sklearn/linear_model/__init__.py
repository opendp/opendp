
from opendp.extras.sklearn.linear_model._make_private_theil_sen import make_private_theil_sen as _make_private_theil_sen # noqa: F401


class LinearRegression():
    '''
    DP Linear Regression
    
    The interface is parallel to that offered by sklearn's
    `LinearRegression <https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LinearRegression.html>`_,
    but only a small fraction of the sklearn interface is supported.
    '''


    def fit(X, y):
        ...

    def predict(X):
        ...