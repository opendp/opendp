from opendp.extras.sklearn.linear_model._make_private_theil_sen import make_private_theil_sen as _make_private_theil_sen # noqa: F401
from opendp._lib import import_optional_dependency


class LinearRegression():
    '''
    DP Linear Regression
    
    The interface is parallel to that offered by sklearn's
    `LinearRegression <https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LinearRegression.html>`_.
    The ``fit`` method returns an sklearn ``LinearRegression`` object.
    '''
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

        :param X: TODO
        :param y: TODO
        :param x_bounds: TODO
        :param y_bounds: TODO
        :param scale: TODO
        :param runs: Controls how many times randomized pairwise predictions are computed. 
            The default is 1. Increasing this value can improve the robustness and accuracy of the results; 
            however, it can also increase computational cost and amount of noise needed later in the algorithm.
        :returns: sklearn.linear_model.LinearRegression
        '''
        meas = _make_private_theil_sen(x_bounds=x_bounds, y_bounds=y_bounds, scale=scale, runs=runs)
        np = import_optional_dependency('numpy')
        slope, intercept = meas(np.stack([X, y], axis=1))

        from sklearn.linear_model import LinearRegression as LR
        fit_regression = LR()
        fit_regression.coef_ = np.array([slope])
        fit_regression.intercept_ = intercept
        return fit_regression
    
    def predict(X):
        '''
        The ``fit()`` method returns a new sklearn object, so this method is never actually used.
        The sklearn documentation of the method with the same name is copied here only for reference.

        > Predict using the linear model.
        > 
        > ### Parameters
        > *X : array-like or sparse matrix, shape (n_samples, n_features)*
        > 
        > Samples.
        >
        > ### Returns
        > *C : array, shape (n_samples,)*
        > 
        > Returns predicted values.

        .. end-markdown

        :raises NotImplementedError: This method is included only for documention.
        '''
        raise NotImplementedError("Included only for documentation")
        