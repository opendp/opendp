'''
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.sklearn.linear_model``.    

If you're interested in the underlying algorithm, we've also
`implemented Theil-Sen Regression as a demonstration of OpenDP plugins <../user-guide/plugins/theil-sen-regression.html>`_.
'''

from typing import Iterable
from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_private_theil_sen as _make_private_theil_sen,
)  # noqa: F401
from opendp._lib import import_optional_dependency
from opendp.mod import assert_features


class LinearRegression:
    """
    DP Linear Regression

    The interface is parallel to that offered by sklearn's
    `LinearRegression <https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LinearRegression.html>`_.
    The ``fit`` method returns an sklearn ``LinearRegression`` object.

    :param fraction_bounds: Lower and upper bounds x_bounds to retain when ``fit`` is called.
    """
    def __init__(self, fraction_bounds: tuple[float, float] = (0.25, 0.75)):
        self.fraction_bounds = fraction_bounds

    def fit(
        self,
        X,
        y,
        x_bounds: Iterable[tuple[float, float]],
        y_bounds: tuple[float, float],
        scale: float,
        runs: int = 1,
        candidates_count: int = 100,
    ):
        """
        Fit DP linear model.

        :param X: Training data. Array-like of shape (n_samples, 1)
        :param y: Target values. Array-like of shape (n_samples,)
        :param x_bounds: Bounds for training data; For the moment, only lists containing a single tuple are supported
        :param y_bounds: Bounds for target data
        :param scale: The scale of the noise to be added
        :param runs: Controls how many times randomized pairwise predictions are computed. Increasing this value can improve the robustness and accuracy of the results; However, it can also increase computational cost and amount of noise needed later in the algorithm.
        :param candidates_count: How many evenly spaced candidates to generate
        :return: A fitted sklearn ``LinearRegression``

        :example:

        >>> import opendp.prelude as dp
        >>> try:
        ...    import sklearn
        ... except ModuleNotFoundError:
        ...     import pytest
        ...     pytest.skip('Requires extra install')
        >>> dp.enable_features("floating-point")
        >>> lin_reg = dp.sklearn.linear_model.LinearRegression().fit(
        ...     X=[[1], [2], [3], [4], [5]],
        ...     y=[1, 2, 3, 4, 5],
        ...     x_bounds=[(0, 10)],
        ...     y_bounds=(0, 10),
        ...     scale=1,
        ... )
        >>> lin_reg.predict([[10]])
        array([...])
        """
        x_bounds = list(x_bounds) # Shouldn't be so large that this is a problem
        if len(x_bounds) != 1:
            raise Exception(f'For now, the x_bounds array must consist of a single tuple, not {x_bounds}')
        meas = _make_private_theil_sen(
            x_bounds=x_bounds[0],
            y_bounds=y_bounds,
            scale=scale,
            runs=runs,
            candidates_count=candidates_count,
            fraction_bounds=self.fraction_bounds,
        )
        np = import_optional_dependency("numpy")
        X = np.array(X)
        slope, intercept = meas(np.stack([X[:, 0], y], axis=1))

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
        raise NotImplementedError("Included only for documentation") # pragma: no cover
