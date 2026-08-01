"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.decomposition``.

Differentially private scikit-learn-compatible PCA. The estimator contains only
PCA parameters; privacy and dataset information are supplied by an OpenDP
:class:`~opendp.context.Context` query.
"""

from __future__ import annotations

import warnings
from dataclasses import asdict, dataclass
from typing import Optional, Sequence, TYPE_CHECKING

from opendp._internal import _make_measurement, _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.context import register
from opendp.extras._utilities import to_then
from opendp.extras.numpy._make_np_mean import make_private_np_mean
from opendp.extras.numpy import then_np_clip
from opendp.extras.sklearn._estimator import DPEstimator
from opendp.extras.sklearn._make_eigendecomposition import (
    then_private_np_eigendecomposition,
)
from opendp.mod import Domain, Measurement, Metric

if TYPE_CHECKING:  # pragma: no cover
    import numpy


@dataclass(kw_only=True, frozen=True)
class PCARelease:
    """The private PCA release consumed by :class:`PCA`."""

    mean: numpy.ndarray
    singular_values: numpy.ndarray
    components: numpy.ndarray


@dataclass(kw_only=True, frozen=True)
class PCAEpsilons:
    """Internal epsilon allocation for the PCA release."""

    eigvals: float
    eigvecs: Sequence[float]
    mean: Optional[float]


def _make_private_pca_with_unit_epsilon(
    input_domain: Domain,
    input_metric: Metric,
    unit_epsilon: float | PCAEpsilons,
    *,
    row_norm: float | None = None,
    num_components: int | None = None,
) -> Measurement:
    """Build the historical PCA mechanism using its internal epsilon units."""
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")

    input_desc = input_domain.descriptor
    if input_desc.size is None:
        raise ValueError("input_domain's size must be known")  # pragma: no cover
    if input_desc.num_columns is None:
        raise ValueError("input_domain's num_columns must be known")  # pragma: no cover
    if input_desc.p not in {None, 2}:
        raise ValueError("input_domain's norm must be an L2 norm")  # pragma: no cover
    if input_desc.num_columns < 1:
        raise ValueError("input_domain's num_columns must be >= 1")  # pragma: no cover

    num_components = (
        input_desc.num_columns if num_components is None else num_components
    )
    if not isinstance(num_components, int) or num_components < 1:
        raise ValueError("num_components must be a positive integer")  # pragma: no cover
    num_components = min(num_components, input_desc.num_columns)

    if isinstance(unit_epsilon, float):
        num_eigvec_releases = min(num_components, input_desc.num_columns - 1)
        # A one-dimensional PCA has no eigenvector release to allocate.
        if num_eigvec_releases == 0:
            num_eigvec_releases = 1  # pragma: no cover
        unit_epsilon = _split_pca_epsilon_evenly(
            unit_epsilon,
            num_eigvec_releases,
            estimate_mean=input_desc.origin is None,
        )

    if not isinstance(unit_epsilon, PCAEpsilons):
        raise ValueError("unit_epsilon must be a float or PCAEpsilons")  # pragma: no cover

    def _make_eigendecomposition(norm, origin):
        return (
            (input_domain, input_metric)
            >> then_np_clip(norm, p=2, origin=origin)
            >> then_center()
            >> then_private_np_eigendecomposition(
                unit_epsilon.eigvals,
                unit_epsilon.eigvecs,
                num_components=num_components,
            )
            >> _new_pure_function(
                lambda out: PCARelease(
                    mean=origin,
                    singular_values=np.sqrt(np.maximum(out[0], 0))[::-1],
                    components=out[1].T,
                )
            )
        )

    if input_desc.norm is not None:
        if unit_epsilon.mean is not None:
            raise ValueError("unit_epsilon.mean must be zero because origin is known")  # pragma: no cover
        return _make_eigendecomposition(input_desc.norm, input_desc.origin)

    if row_norm is None:
        raise ValueError("must have either bounded input_domain or specify row_norm")  # pragma: no cover

    # The internal PCA mechanism is calibrated for a two-record change.
    unit_d_in = 2
    compositor = dp.c.make_adaptive_composition(
        input_domain,
        input_metric,
        dp.max_divergence(),
        d_in=unit_d_in,
        d_mids=[unit_epsilon.mean, _make_eigendecomposition(row_norm, 0).map(unit_d_in)],
    )

    def _function(data):
        qbl = compositor(data)
        m_mean = dp.binary_search_chain(
            lambda scale: make_private_np_mean(
                input_domain, input_metric, scale, norm=row_norm, p=1
            ),
            d_in=unit_d_in,
            d_out=unit_epsilon.mean,
            T=float,
        )
        origin = qbl(m_mean)
        return qbl(_make_eigendecomposition(row_norm, origin))

    return _make_measurement(
        input_domain,
        input_metric,
        compositor.output_measure,
        _function,
        compositor.map,
    )


def make_private_pca(
    input_domain: Domain,
    input_metric: Metric,
    output_measure,
    d_in,
    d_out,
    *,
    num_components: int | None = None,
) -> Measurement:
    """Construct a calibrated differentially private PCA measurement.

    The public constructor is calibrated in the units of ``output_measure``;
    the legacy epsilon allocation is kept private to this module.
    """
    import opendp.prelude as dp

    if input_metric != dp.symmetric_distance():
        raise ValueError("PCA currently supports symmetric_distance() only")  # pragma: no cover
    if output_measure != dp.max_divergence():
        raise ValueError("PCA currently supports max_divergence() only")  # pragma: no cover
    if d_in <= 0 or d_out <= 0:
        raise ValueError("d_in and d_out must be positive")  # pragma: no cover
    if d_in != 1:
        raise ValueError("PCA currently supports d_in=1 only")

    desc = input_domain.descriptor
    if not hasattr(desc, "num_columns") or not hasattr(desc, "size"):
        raise ValueError("input_domain must be a two-dimensional NumPy array domain")  # pragma: no cover
    if desc.size is None or desc.num_columns is None:
        raise ValueError("input_domain size and num_columns must be known")  # pragma: no cover
    if desc.norm is None:
        raise ValueError(
            "PCA requires an L2-bounded input domain. Apply "
            "query.np_clip(p=2, norm=...) before fitting."
        )
    if desc.p != 2:
        raise ValueError(
            "PCA requires an L2 norm bound; apply np_clip with p=2."
        )

    # Calibrate the historical mechanism to the Context privacy budget. The
    # input domain already certifies the row norm and origin.
    unit_epsilon = 2 * float(d_out) / float(d_in)
    return _make_private_pca_with_unit_epsilon(
        input_domain,
        input_metric,
        unit_epsilon,
        num_components=num_components,
    )


then_private_pca = to_then(make_private_pca)
register(make_private_pca)


class PCA(DPEstimator):
    """Differentially private PCA with a sklearn estimator interface."""

    def __init__(
        self,
        n_components=None,
        *,
        whiten=False,
        row_norm=None,
        epsilon=None,
        n_samples=None,
        n_features=None,
        n_changes=1,
    ):
        self.n_components = n_components
        self.whiten = whiten
        self.row_norm = row_norm

        # Removal target: OpenDP 0.16.0. Remove epsilon, n_samples,
        # n_features, n_changes, raw-array fit, measurement(), _legacy_mode,
        # and _make_legacy_measurement() together.
        self.epsilon = epsilon
        self.n_samples = n_samples
        self.n_features = n_features
        self.n_changes = n_changes

        legacy_values = (epsilon, n_samples, n_features)
        if any(value is not None for value in legacy_values) and not all(
            value is not None for value in legacy_values
        ):
            raise TypeError(
                "Legacy PCA requires epsilon, n_samples, and n_features together"
            )

        if epsilon is not None and not epsilon > 0:
            raise ValueError("epsilon must be positive")
        if n_samples is not None and not n_samples > 1:
            raise ValueError("n_samples must be greater than 1")
        if n_features is not None and not n_features >= 1:
            raise ValueError("n_features must be at least 1")
        if not n_changes >= 1:
            raise ValueError("n_changes must be at least 1")

        if all(value is not None for value in legacy_values):
            self._validate_n_components(n_components, n_samples, n_features)

    @property
    def _legacy_mode(self) -> bool:
        legacy_values = (
            self.epsilon,
            self.n_samples,
            self.n_features,
        )

        if any(value is not None for value in legacy_values) and not all(
            value is not None for value in legacy_values
        ):
            raise TypeError(
                "Legacy PCA requires epsilon, n_samples, and n_features together"
            )

        if not all(value is not None for value in legacy_values):
            return False

        if self.epsilon <= 0:
            raise ValueError("epsilon must be positive")
        if self.n_samples <= 1:
            raise ValueError("n_samples must be greater than 1")
        if self.n_features < 1:
            raise ValueError("n_features must be at least 1")
        if self.n_changes < 1:
            raise ValueError("n_changes must be at least 1")

        self._validate_n_components(
            self.n_components,
            self.n_samples,
            self.n_features,
        )
        return True

    def make(
        self,
        input_domain,
        input_metric,
        output_measure,
        d_in,
        d_out,
    ) -> Measurement:
        if self.row_norm is not None and not self._legacy_mode:
            raise TypeError(
                "Apply norm clipping to the input domain or query with "
                "np_clip(p=2, norm=...) instead of setting row_norm."
            )

        desc = input_domain.descriptor
        if getattr(desc, "size", None) is None or getattr(desc, "num_columns", None) is None:
            raise ValueError("PCA requires known input-domain size and num_columns")
        self._validate_n_components(self.n_components, desc.size, desc.num_columns)
        # Integer component counts can avoid releasing unused eigenvectors.
        # Other modes need the full decomposition for postprocessing.
        num_components = (
            self.n_components
            if isinstance(self.n_components, int)
            and not isinstance(self.n_components, bool)
            else None
        )
        return make_private_pca(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            num_components=num_components,
        )

    @staticmethod
    def _validate_n_components(n_components, n_samples, n_features):
        if n_components is None:
            return
        if n_components == "mle":
            if n_samples < n_features:
                raise ValueError("n_components='mle' requires n_samples >= n_features")
            return
        if isinstance(n_components, bool):
            raise ValueError("n_components must be None, an integer, a fraction, or 'mle'")
        if isinstance(n_components, int):
            if not 1 <= n_components <= min(n_samples, n_features):
                raise ValueError("n_components must be between 1 and min(n_samples, n_features)")
            return
        if isinstance(n_components, float):
            if not 0 < n_components < 1:
                raise ValueError("n_components must be in (0, 1) when fractional")
            return
        raise ValueError("n_components must be None, an integer, a fraction, or 'mle'")

    @staticmethod
    def _preflight_sklearn():
        import_optional_dependency("sklearn")
        from sklearn.decomposition._pca import _infer_dimension
        from sklearn.utils.extmath import svd_flip

        return _infer_dimension, svd_flip

    def fit(self, X, y=None, **fit_params):
        from opendp.context import Query

        if isinstance(X, Query):
            if self._legacy_mode:
                raise TypeError(
                    "Do not provide epsilon, n_samples, or n_features when fitting "
                    "PCA through an OpenDP Context"
                )
            if self.row_norm is not None:
                raise TypeError(
                    "row_norm is only supported by the deprecated array-based PCA "
                    "API. Apply norm clipping to the query instead, for example "
                    "context.query().np_clip(p=2, norm=...)."
                )
            return super().fit(X, y=y, **fit_params)

        if not self._legacy_mode:
            raise TypeError(
                "PCA.fit() expects an OpenDP Context query. "
                "The deprecated array-based API requires epsilon, n_samples, "
                "and n_features in the constructor."
            )

        warnings.warn(
            "Passing an array directly to PCA.fit() is deprecated. "
            "Construct PCA with algorithm parameters only and fit a bounded "
            "query, for example PCA(...).fit(context.query().np_clip(p=2, "
            "norm=...)).",
            FutureWarning,
            stacklevel=2,
        )
        return self._fit_legacy(X, y=y, **fit_params)

    def _prepare_fit_query(self, X, y=None, **fit_params):
        # Validate sklearn before the query consumes any privacy budget.
        self._preflight_sklearn()
        # PCA ignores y, as sklearn's PCA does.
        self._reject_fit_params(fit_params)
        return X

    def _make_legacy_measurement(self) -> Measurement:
        if hasattr(self, "components_"):
            raise ValueError("DP-PCA model has already been fitted")

        import opendp.prelude as dp

        n_estimated_components = (
            self.n_components
            if isinstance(self.n_components, int)
            and not isinstance(self.n_components, bool)
            else self.n_features
        )
        input_domain = dp.numpy.array2_domain(
            num_columns=self.n_features,
            size=self.n_samples,
            T=float,
        )
        unit_epsilon = self.epsilon / self.n_changes * 2
        return _make_private_pca_with_unit_epsilon(
            input_domain,
            dp.symmetric_distance(),
            unit_epsilon,
            row_norm=self.row_norm,
            num_components=n_estimated_components,
        )

    def _fit_legacy(self, X, y=None, **fit_params):
        if not self._legacy_mode:
            raise TypeError(
                "PCA.fit() expects an OpenDP Context query. "
                "The deprecated array-based API requires epsilon, n_samples, "
                "and n_features in the constructor."
            )

        np = import_optional_dependency("numpy")

        self._reject_fit_params(fit_params)
        self._preflight_sklearn()

        if hasattr(self, "components_"):
            raise ValueError("DP-PCA model has already been fitted")

        X = np.asarray(X)
        expected_shape = (self.n_samples, self.n_features)
        if X.ndim != 2 or X.shape != expected_shape:
            raise ValueError(f"X must have shape {expected_shape}")

        self._fit_n_samples = self.n_samples
        self._fit_n_features = self.n_features
        release = self._make_legacy_measurement()(X)
        self._ingest_release(release)
        return self

    def measurement(self):
        if not self._legacy_mode:
            raise TypeError(
                "measurement() is only available for the deprecated PCA "
                "constructor. Use make(...) or fit(context.query())."
            )

        warnings.warn(
            "PCA.measurement() is deprecated. Use PCA.make(...) or fit an "
            "OpenDP Context query instead.",
            FutureWarning,
            stacklevel=2,
        )

        self._preflight_sklearn()
        measurement = self._make_legacy_measurement()

        def ingest_and_return(release):
            self._fit_n_samples = self.n_samples
            self._fit_n_features = self.n_features
            self._ingest_release(release)
            return self

        return measurement >> _new_pure_function(ingest_and_return)

    def _ingest_release(self, release: PCARelease) -> None:
        np = import_optional_dependency("numpy")
        _infer_dimension, svd_flip = self._preflight_sklearn()

        if not isinstance(release, PCARelease):
            raise TypeError("PCA expected a PCARelease")

        n_samples = self._fit_n_samples
        n_features = self._fit_n_features
        if n_samples <= 1:
            raise ValueError("PCA requires at least two samples")

        mean = np.asarray(release.mean)
        singular_values = np.asarray(release.singular_values)
        components = np.asarray(release.components)
        U, components = svd_flip(components.T, components)
        del U

        explained_variance = singular_values**2 / (n_samples - 1)
        total_variance = np.sum(explained_variance)
        explained_variance_ratio = explained_variance / total_variance

        n_components = self.n_components
        if n_components is None:
            n_components = min(n_samples, n_features)
        elif n_components == "mle":
            if n_samples < n_features:
                raise ValueError("n_components='mle' requires n_samples >= n_features")
            n_components = _infer_dimension(explained_variance, n_samples)
        elif isinstance(n_components, float):
            if not 0 < n_components < 1:
                raise ValueError("n_components must be in (0, 1) when fractional")
            n_components = (
                np.searchsorted(
                    np.cumsum(explained_variance_ratio), n_components, side="right"
                )
                + 1
            )
        elif isinstance(n_components, int) and not isinstance(n_components, bool):
            if not 1 <= n_components <= min(n_samples, n_features):
                raise ValueError("n_components must be between 1 and min(n_samples, n_features)")
        else:
            raise ValueError("n_components must be None, an integer, a fraction, or 'mle'")

        n_components = int(n_components)
        if n_components < 1 or n_components > len(explained_variance):
            raise ValueError("n_components is incompatible with the released PCA")

        self.mean_ = mean
        self.components_ = components[:n_components, :]
        self.n_components_ = n_components
        self.explained_variance_ = explained_variance[:n_components]
        self.explained_variance_ratio_ = explained_variance_ratio[:n_components]
        self.singular_values_ = singular_values[:n_components]
        self.noise_variance_ = (
            float(np.mean(explained_variance[n_components:]))
            if n_components < min(n_features, n_samples)
            else 0.0
        )
        self.n_samples_ = n_samples
        self.n_features_in_ = n_features
        del self._fit_n_samples
        del self._fit_n_features

    def then(self, output_measure, d_in, d_out):
        # Capture public shape metadata for the release postprocessor without
        # putting dataset dimensions in the estimator constructor.
        def make(input_domain, input_metric):
            desc = input_domain.descriptor
            self._fit_n_samples = desc.size
            self._fit_n_features = desc.num_columns
            return self.make(
                input_domain, input_metric, output_measure, d_in, d_out
            )

        from opendp.mod import _PartialConstructor

        return _PartialConstructor(make)

    def _check_is_fitted(self):
        if not hasattr(self, "components_"):
            raise ValueError("PCA instance is not fitted yet")

    def _whitening_scale(self):
        np = import_optional_dependency("numpy")
        scale = np.sqrt(self.explained_variance_)
        return np.maximum(scale, np.finfo(scale.dtype).eps)

    def transform(self, X):
        np = import_optional_dependency("numpy")
        self._check_is_fitted()
        X = np.asarray(X)
        if X.ndim != 2 or X.shape[1] != self.n_features_in_:
            raise ValueError(f"X must have shape (n_samples, {self.n_features_in_})")
        transformed = (X - self.mean_) @ self.components_.T
        if self.whiten:
            transformed /= self._whitening_scale()
        return transformed

    def inverse_transform(self, X):
        np = import_optional_dependency("numpy")
        self._check_is_fitted()
        X = np.asarray(X)
        if X.ndim != 2 or X.shape[1] != self.n_components_:
            raise ValueError(f"X must have shape (n_samples, {self.n_components_})")
        if self.whiten:
            X = X * self._whitening_scale()
        return X @ self.components_ + self.mean_

    def get_covariance(self):
        np = import_optional_dependency("numpy")
        self._check_is_fitted()

        explained_variance = np.maximum(
            self.explained_variance_ - self.noise_variance_,
            0.0,
        )

        components = self.components_
        if self.whiten:
            components = components * np.sqrt(self.explained_variance_)[:, None]

        covariance = (
            components.T * explained_variance
        ) @ components
        covariance.flat[:: len(covariance) + 1] += self.noise_variance_
        return covariance

    def get_precision(self):
        np = import_optional_dependency("numpy")
        self._check_is_fitted()
        return np.linalg.inv(self.get_covariance())

    def fit_transform(self, X, y=None, **fit_params):
        raise NotImplementedError(
            "fit_transform would release transformed private training records; "
            "fit through an OpenDP Context and call transform on public data."
        )


def _smaller(v):
    """Return the next non-negative float closer to zero."""
    np = import_optional_dependency("numpy")
    if v < 0:
        raise ValueError("expected non-negative value")  # pragma: no cover
    return v if v == 0 else np.nextafter(v, -1)


def _split_pca_epsilon_evenly(unit_epsilon, num_eigvec_releases, estimate_mean=False):
    num_queries = 3 if estimate_mean else 2
    per_query_epsilon = unit_epsilon / num_queries
    per_evec_epsilon = per_query_epsilon / num_eigvec_releases
    return PCAEpsilons(
        eigvals=per_query_epsilon,
        eigvecs=[_smaller(per_evec_epsilon)] * num_eigvec_releases,
        mean=_smaller(per_query_epsilon) if estimate_mean else None,
    )


def _make_center(input_domain, input_metric):
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")
    input_desc = input_domain.descriptor
    center = (
        input_desc.origin
        if input_desc.origin is not None
        else np.zeros(input_desc.num_columns)
    )
    kwargs = asdict(input_desc) | {
        "origin": np.zeros(input_desc.num_columns)
    }
    return _make_transformation(
        input_domain,
        input_metric,
        dp.numpy.array2_domain(**kwargs),
        input_metric,
        lambda arg: arg - center,
        lambda d_in: d_in,
    )


then_center = to_then(_make_center)
