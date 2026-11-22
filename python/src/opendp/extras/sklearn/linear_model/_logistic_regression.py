"""Sklearn-style logistic regression fitted with objective perturbation."""

import numpy as np
from scipy.special import expit  # type: ignore[import-untyped]
from sklearn.base import BaseEstimator, ClassifierMixin
from sklearn.utils.validation import check_is_fitted, validate_data

from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from ._make_private_objective_perturbation import make_private_objective_perturbation


class LogisticRegression(
    ClassifierMixin, MeasurementMixin, TransformationMixin, BaseEstimator
):
    """Differentially private binary logistic regression via objective perturbation.

    Unlike full-batch DP gradient descent, numerical optimization of the
    perturbed convex objective incurs no additional privacy expenditure.
    A first-order residual certificate and independent small output perturbation
    account for approximate numerical minimization.

    All feature rows are radially clipped to a public bound. An intercept
    (if requested) is ALSO ridge-regularized and an additional public ridge may
    be imposed for the privacy Jacobian. Thus the regularized objective may
    differ from nonprivate sklearn's LogisticRegression at the same C.

    :param C: inverse strength of coefficient L2 penalty on summed log loss
    :param fit_intercept: include a ridge-regularized intercept
    :param tol: upper bound on the perturbed objective's gradient residual
    :param max_iter: initial L-BFGS iteration budget, with certified continuation
    :param feature_norm: public per-row L2 bound (enforced by radial clipping)
    """

    def __init__(
        self,
        *,
        C=1.0,
        fit_intercept=True,
        tol=1e-8,
        max_iter=100,
        feature_norm=1.0,
    ):
        self.C = C
        self.fit_intercept = fit_intercept
        self.tol = tol
        self.max_iter = max_iter
        self.feature_norm = feature_norm

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        return make_private_objective_perturbation(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            C=self.C,
            fit_intercept=self.fit_intercept,
            tol=self.tol,
            max_iter=self.max_iter,
            feature_norm=self.feature_norm,
        )

    def _ingest_release(self, release):
        theta = np.asarray(release)
        self.coef_ = (theta[:-1] if self.fit_intercept else theta).reshape(1, -1)
        self.intercept_ = theta[-1:] if self.fit_intercept else np.zeros(1)
        self.classes_ = np.array([0, 1])
        self.n_features_in_ = self.coef_.shape[1]

    def predict_proba(self, X):
        check_is_fitted(self, ["coef_", "intercept_", "classes_"])
        X = validate_data(
            self, X, reset=False, ensure_all_finite=False, ensure_min_samples=0
        )
        X = np.nan_to_num(np.asarray(X, dtype=float), nan=0.0, posinf=0.0, neginf=0.0)
        lengths = np.linalg.norm(X, axis=1)
        X = X * np.minimum(
            1.0, self.feature_norm / np.maximum(lengths, 1e-300)
        )[:, None]
        with np.errstate(all="ignore"):
            z = np.nan_to_num(X @ self.coef_[0] + self.intercept_[0], nan=0.0)
        p = expit(z)
        return np.column_stack([1 - p, p])

    def predict(self, X):
        return self.classes_[(self.predict_proba(X)[:, 1] > 0.5).astype(int)]

    def rowwise_output_domain(self, feature_domain, *, method):
        """Describe the output domain of independent per-row classification."""
        from opendp.extras.numpy import NPArray2Domain, array2_domain

        check_is_fitted(self, ["coef_", "intercept_", "classes_"])
        x = feature_domain.cast(NPArray2Domain)
        if x.num_columns != self.n_features_in_:
            raise ValueError("Prediction feature count differs from fitted model")
        if not np.array_equal(self.classes_, [0, 1]):
            raise ValueError("Only 0/1 classes are currently supported")

        if method == "predict":
            num_columns = 1
            dtype = np.int64
        elif method == "predict_proba":
            num_columns = 2
            dtype = np.float64
        else:
            raise ValueError(f"Method {method!r} does not support rowwise inference")

        return array2_domain(T=dtype, num_columns=num_columns, size=x.size, nan=False)
