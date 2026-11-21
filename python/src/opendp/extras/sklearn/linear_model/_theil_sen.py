"""Private Theil-Sen regressor."""

from __future__ import annotations

import numpy as np
from sklearn.base import BaseEstimator, RegressorMixin
from sklearn.utils.validation import check_is_fitted, validate_data

from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_private_theil_sen,
)


class TheilSenRegressor(
    RegressorMixin, MeasurementMixin, TransformationMixin, BaseEstimator
):
    """Differentially private univariate Theil-Sen regression.

    Estimates a private median of randomly paired slopes, followed by a private
    median of residuals to estimate the intercept.

    :param slope_bounds: Public interval for slope candidates.
    :param intercept_bounds: Public interval for intercept candidates.

    :ivar coef_: One-element array containing the privately estimated slope.
    :ivar intercept_: Privately estimated intercept.
    :ivar n_features_in_: Number of fitted input features (one).

    Reference: Sarathy and Vadhan, `Analyzing the Differentially Private
    Theil-Sen Estimator for Simple Linear Regression
    <https://doi.org/10.56553/popets-2025-0013>`_, PoPETS 2025.
    """

    def __init__(self, *, slope_bounds, intercept_bounds):
        self.slope_bounds = slope_bounds
        self.intercept_bounds = intercept_bounds

    def make_measurement(
        self,
        input_domain,
        input_metric,
        output_measure,
        d_in,
        d_out,
    ):
        return make_private_theil_sen(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            slope_bounds=self.slope_bounds,
            intercept_bounds=self.intercept_bounds,
        )

    def _ingest_release(self, release):
        slope, intercept = release
        self.coef_ = np.asarray([slope])
        self.intercept_ = float(intercept)
        self.n_features_in_ = 1

    def predict(self, X):
        check_is_fitted(self)
        X = validate_data(
            self, X, reset=False, ensure_all_finite=False, ensure_min_samples=0
        )
        with np.errstate(all="ignore"):
            return X[:, 0] * self.coef_[0] + self.intercept_

    def rowwise_output_domain(self, feature_domain, *, method):
        """Describe the output domain of independent per-row prediction."""
        import opendp.prelude as dp

        if method != "predict":
            raise ValueError(f"Method {method!r} does not support rowwise inference")
        check_is_fitted(self)
        desc = feature_domain.cast(dp.numpy.NPArray2Domain)
        if desc.num_columns != self.n_features_in_:
            raise ValueError("Prediction feature count differs from fitted model")
        if str(desc.T) not in {
            "f32",
            "f64",
            "i8",
            "i16",
            "i32",
            "i64",
            "u8",
            "u16",
            "u32",
            "u64",
        }:
            raise TypeError("Theil-Sen prediction requires numerical features")
        return dp.numpy.array2_domain(T=dp.f64, size=desc.size, num_columns=1, nan=True)
