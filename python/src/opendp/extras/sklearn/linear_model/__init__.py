from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_private_theil_sen as _make_private_theil_sen,
)  # noqa: F401
from opendp._lib import import_optional_dependency


class LinearRegression:
    """
    DP Linear Regression

    The interface is parallel to that offered by sklearn's
    `LinearRegression <https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LinearRegression.html>`_.
    The ``fit`` method returns an sklearn ``LinearRegression`` object.
    """

    def fit(
        self,
        X,
        y,
        x_bounds: tuple[float, float], # TODO: Wrap this so each feature has its own bound tuple?
        y_bounds: tuple[float, float],
        scale: float,
        runs: int = 1,
    ):
        """
        Fit DP linear model.

        :param X: Training data. Array-like of shape (n_samples, 1)
        :param y: Target values. Array-like of shape (n_samples,)
        :param x_bounds: Bounds for training data
        :param y_bounds: Bounds for target data
        :param scale: The scale of the noise to be added
        :param runs: Controls how many times randomized pairwise predictions are computed. Increasing this value can improve the robustness and accuracy of the results; However, it can also increase computational cost and amount of noise needed later in the algorithm.
        :return: A fitted sklearn ``LinearRegression``

        :example:

        >>> import opendp.prelude as dp
        >>> try:
        ...    import sklearn
        ... except ModuleNotFoundError:
        ...     import pytest
        ...     pytest.skip('Requires extra install')
        >>> lin_reg = dp.sklearn.linear_model.LinearRegression().fit(
        ...     X=[[1], [2], [3], [4], [5]],
        ...     y=[1, 2, 3, 4, 5],
        ...     x_bounds=(0,10),
        ...     y_bounds=(0,10),
        ...     scale=1,
        ... )
        >>> lin_reg.predict([[10]])
        array([...])
        """
        meas = _make_private_theil_sen(
            x_bounds=x_bounds, y_bounds=y_bounds, scale=scale, runs=runs
        )
        np = import_optional_dependency("numpy")
        slope, intercept = meas(np.stack([[x[0] for x in X], y], axis=1))

        from sklearn.linear_model import LinearRegression as LR

        fit_regression = LR()
        fit_regression.coef_ = np.array([slope])
        fit_regression.intercept_ = intercept
        return fit_regression

    def predict(X):
        """
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
        """
        raise NotImplementedError("Included only for documentation")
