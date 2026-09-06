"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.linear_model``.

If you're interested in the underlying algorithm, we've also
`implemented Theil-Sen Regression as a demonstration of OpenDP plugins <../user-guide/plugins/theil-sen-regression.html>`_.
"""

from typing import Iterable
from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_private_theil_sen as _make_private_theil_sen,
)  # noqa: F401
from opendp._lib import import_optional_dependency
from opendp.mod import Measure
from opendp.extras.sklearn.linear_model._make_private_logistic_regression import (
    make_private_logistic_regression as _make_private_logistic_regression,
)
from opendp.extras.sklearn._estimator import DPEstimator

__all__ = ["LinearRegression", "LogisticRegression"]


class LinearRegression:
    """
    DP Linear Regression

    The interface is parallel to that offered by sklearn's
    `LinearRegression <https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LinearRegression.html>`_.
    The ``fit`` method returns an sklearn ``LinearRegression`` object.

    :param x_bounds: Bounds for training data; For the moment, only lists containing a single tuple are supported
    :param y_bounds: Bounds for target data
    :param scale: The scale of the noise to be added
    :param runs: Controls how many times randomized pairwise predictions are computed. Increasing this value can improve the robustness and accuracy of the results; However, it can also increase computational cost and amount of noise needed later in the algorithm.
    :param candidates_count: How many evenly spaced candidates to generate
    :param fraction_bounds: predict y values at these cut percentiles of x_bounds.
    """

    def __init__(
        self,
        output_measure: Measure,
        x_bounds: Iterable[tuple[float, float]],
        y_bounds: tuple[float, float],
        scale: float,
        runs: int = 1,
        candidates_count: int = 100,
        fraction_bounds: tuple[float, float] = (0.25, 0.75),
    ):
        x_bounds = list(x_bounds)  # Shouldn't be so large that this is a problem
        if len(x_bounds) != 1:
            msg = f"For now, the x_bounds array must consist of a single tuple, not {x_bounds}"
            raise Exception(msg)

        self.measurement = _make_private_theil_sen(
            output_measure=output_measure,
            x_bounds=x_bounds[0],
            y_bounds=y_bounds,
            scale=scale,
            runs=runs,
            candidates_count=candidates_count,
            fraction_bounds=fraction_bounds,
        )

    def fit(
        self,
        X,
        y,
    ):
        """
        Fit DP linear model.

        :param X: Training data. Array-like of shape (n_samples, 1)
        :param y: Target values. Array-like of shape (n_samples,)
        :return: A fitted sklearn ``LinearRegression``

        :example:

        >>> import opendp.prelude as dp
        >>> try:
        ...    import sklearn
        ... except ModuleNotFoundError:
        ...     import pytest
        ...     pytest.skip('Requires extra install')
        >>> dp.enable_features("idealized-numerics")
        >>> lin_reg = dp.sklearn.linear_model.LinearRegression(
        ...     dp.max_divergence(),
        ...     x_bounds=[(0, 10)],
        ...     y_bounds=(0, 10),
        ...     scale=1,
        ... ).fit(
        ...     X=[[1], [2], [3], [4], [5]],
        ...     y=[1, 2, 3, 4, 5],
        ... )
        >>> lin_reg.predict([[10]])
        array([...])
        """
        np = import_optional_dependency("numpy")
        from sklearn.linear_model import LinearRegression as LR

        X = np.array(X)
        slope, intercept = self.measurement(np.stack([X[:, 0], y], axis=1))

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
        raise NotImplementedError("Included only for documentation")  # pragma: no cover


class LogisticRegression(DPEstimator):
    """
    DP Logistic Regression

    The interface is parallel to that offered by sklearn's
    `LogisticRegression <https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LogisticRegression.html>`_.
    Fitting is performed with differentially private stochastic gradient descent (DP-SGD):
    each iteration clips per-example gradients to an L2 norm bound, sums them under Gaussian
    noise, and takes a gradient step, with the privacy budget split across iterations under
    zero-concentrated differential privacy. After ``fit``, the model exposes ``coef_``,
    ``intercept_``, and ``classes_``, and supports ``predict`` and ``predict_proba``.

    Data is expected as a single 2-dimensional array whose last column is the binary target
    and whose remaining columns are the features.

    :param n_iters: Number of DP-SGD iterations. Also controls the budget split: the total privacy budget is divided across this many gradient steps, so more iterations means more noise per step.
    :param learning_rate: Step size for the gradient update applied to the weights each iteration.
    :param clip_norm: Per-example gradient clipping bound. Each example's gradient is scaled to have L2 norm at most this value, bounding one record's influence on the update. Larger values distort gradients less but require more noise.
    :param l2_penalty: Strength of optional L2 regularization added to the gradient update. Defaults to 0.0 (no regularization).
    """

    def __init__(self, n_iters=100, learning_rate=0.1, clip_norm=1.0, l2_penalty=0.0):
        self.n_iters = n_iters
        self.learning_rate = learning_rate
        self.clip_norm = clip_norm
        self.l2_penalty = l2_penalty

    def _prepare_fit_query(self, X, y=None, **fit_params):
        """Normalize fit arguments into a single input query.

        This estimator expects the target to already be the **last column** of the
        query's data, with the preceding columns as features. The target is not passed
        separately: ``y`` is accepted for scikit-learn signature compatibility but is not
        used to relocate or attach a target column, and supplying it has no effect on the
        released model. Fit metadata (``fit_params``) is not supported and is rejected.

        :param X: a Context query whose data has the binary target as its final column
        :param y: accepted for sklearn compatibility; ignored (the target must already be the last column of ``X``)
        :param fit_params: not supported; any fit metadata raises
        :return: the input query ``X`` unchanged
        :raises TypeError: if any ``fit_params`` are supplied
        """

        self._reject_fit_params(fit_params)
        return X  # we are assuming that the last column is the target (open design question)

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        return _make_private_logistic_regression(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            n_iters=self.n_iters,
            learning_rate=self.learning_rate,
            clip_norm=self.clip_norm,
            l2_penalty=self.l2_penalty,
        )

    def _ingest_release(self, release):
        np = import_optional_dependency("numpy")
        theta = np.asarray(release)
        self.coef_ = theta[:-1]
        self.intercept_ = theta[-1]
        self.classes_ = np.array([0, 1])

    def predict_proba(self, X):
        np = import_optional_dependency("numpy")
        special = import_optional_dependency("scipy.special")
        Xb = np.hstack([np.asarray(X), np.ones((len(X), 1))])
        p = special.expit(Xb @ np.concatenate([self.coef_, [self.intercept_]]))
        return np.column_stack([1 - p, p])

    def predict(self, X):
        return (self.predict_proba(X)[:, 1] >= 0.5).astype(int)
