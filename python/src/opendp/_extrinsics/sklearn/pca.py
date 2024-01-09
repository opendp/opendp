from __future__ import annotations
from opendp._extrinsics.make_np_pca import make_private_np_pca
from opendp.mod import Measurement


try:
    from sklearn.decomposition import PCA as SKLPCA  # type: ignore[import]
except ImportError:

    class SKLPCA(object):  # type: ignore[no-redef]
        def __init__(*args, **kwargs):
            raise ImportError(
                "please install scikit-learn to use the sklearn API: https://scikit-learn.org/stable/install.html"
            )


class PCA(SKLPCA):
    def __init__(
        self,
        *,
        epsilon: float,
        row_norm: float,
        n_samples: int,
        n_features: int,
        n_components: int | float | str | None = None,
        n_changes: int = 1,
        whiten: bool = False,
    ) -> None:
        super().__init__(
            n_components or n_features,
            whiten=whiten,
        )
        self.epsilon = epsilon
        self.row_norm = row_norm
        self.n_samples = n_samples
        self.n_features_in_ = n_features
        self.n_changes = n_changes

    @property
    def n_features(self):
        return self.n_features_in_

    # this overrides the scikit-learn method to instead use the opendp-core constructor
    def _fit(self, X):
        return self._prepare_fitter()(X)

    def _prepare_fitter(self) -> Measurement:
        """Returns a measurement that computes the mean and eigendecomposition,
        and then apply those releases to self."""
        import opendp.prelude as dp

        if hasattr(self, "components_"):
            raise ValueError("DP-PCA model has already been fitted")

        input_domain = dp.np_array2_domain(
            num_columns=self.n_features_in_, size=self.n_samples, T=float
        )
        input_metric = dp.symmetric_distance()

        n_estimated_components = (
            self.n_components
            if isinstance(self.n_components, int)
            else self.n_features_in_
        )

        return make_private_np_pca(
            input_domain,
            input_metric,
            self.epsilon / self.n_changes * 2,
            norm=self.row_norm,
            num_components=n_estimated_components,
        ) >> self._postprocess

    def _postprocess(self, values):
        """A function that applies a release of the mean and eigendecomposition to self"""
        import numpy as np
        from sklearn.utils.extmath import stable_cumsum, svd_flip # type: ignore[import]
        from sklearn.decomposition._pca import _infer_dimension # type: ignore[import]

        self.mean_, S, Vt = values

        # flip eigenvectors' sign to enforce deterministic output
        _, components_ = svd_flip(Vt.T, Vt)
        U = None

        # CODE BELOW THIS POINT IS FROM SKLEARN
        # Get variance explained by singular values
        explained_variance_ = np.sort((S**2) / (self.n_samples - 1))[::-1]
        total_var = explained_variance_.sum()
        explained_variance_ratio_ = explained_variance_ / total_var
        singular_values_ = S.copy()  # Store the singular values.

        # Postprocess the number of components required
        if self.n_components == "mle":
            n_components = _infer_dimension(explained_variance_, self.n_samples)
        elif 0 < self.n_components < 1.0:
            # number of components for which the cumulated explained
            # variance percentage is superior to the desired threshold
            # side='right' ensures that number of features selected
            # their variance is always greater than n_components float
            # passed. More discussion in issue: https://github.com/scikit-learn/scikit-learn/pull/15669
            ratio_cumsum = stable_cumsum(explained_variance_ratio_)
            n_components = (
                np.searchsorted(ratio_cumsum, self.n_components, side="right") + 1
            )
        else:
            n_components = self.n_components
        # Compute noise covariance using Probabilistic PCA model
        # The sigma2 maximum likelihood (cf. eq. 12.46)
        if n_components < min(self.n_features_in_, self.n_samples):
            self.noise_variance_ = explained_variance_[n_components:].mean()
        else:
            self.noise_variance_ = 0.0

        self.components_ = components_[:n_components]
        self.n_components_ = n_components
        self.explained_variance_ = explained_variance_[:n_components]
        self.explained_variance_ratio_ = explained_variance_ratio_[:n_components]
        self.singular_values_ = singular_values_[:n_components]

        return U, S, Vt

    def measurement(self) -> Measurement:
        """Return a measurement that releases a fitted model."""
        return self._prepare_fitter() >> (lambda _: self)

    # overrides an sklearn method
    def _validate_params(*args, **kwargs):
        pass
