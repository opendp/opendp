import numpy as np

from opendp.extras.sklearn.linear_model._make_private_theil_sen import make_private_theil_sen as _make_private_theil_sen # noqa: F401


class LinearRegression():
    '''
    DP Linear Regression
    
    The interface is parallel to that offered by sklearn's
    `LinearRegression <https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LinearRegression.html>`_,
    but only a fraction of the sklearn interface is supported.
    '''
    def __init__(self):
        # TODO: Should these be exposed via set_params and get_params? Or kept private?
        self._slope = None
        self._intercept = None


    def fit(
            X,
            y,
            x_bounds: tuple[float, float],
            y_bounds: tuple[float, float],
            scale: float,
            runs: int=1
        ):
        '''
        Fit DP linear model.

        :param runs: Controls how many times randomized pairwise predictions are computed. 
        The default is 1. Increasing this value can improve the robustness and accuracy of the results; 
        however, it can also increase computational cost and amount of noise needed later in the algorithm.
        '''
        # Should this return a new instance of this class?
        # or should we create a new sklearn.linear_model.LinearRegression
        # instance and set parameters on it?
        meas = _make_private_theil_sen(x_bounds=x_bounds, y_bounds=y_bounds, scale=scale, runs=runs)
        slope, intercept = meas(np.stack([X, y], axis=1))

        fit_regression = LinearRegression()
        fit_regression._slope = slope
        fit_regression._intercept = intercept
        return fit_regression
        ...

    def predict(X):
        '''
        Predict using the DP linear model.
        '''
        