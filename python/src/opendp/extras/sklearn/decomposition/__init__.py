"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.decomposition``.

Differentially private scikit-learn-compatible PCA. The estimator contains only
PCA parameters; privacy and dataset information are supplied to ``fit`` or by an
OpenDP :class:`~opendp.context.Context` query.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
from numbers import Integral
from typing import TYPE_CHECKING

from sklearn.base import (
    BaseEstimator,
    ClassNamePrefixFeaturesOutMixin,
    TransformerMixin,
)
from sklearn.decomposition import PCA as SklearnPCA
from sklearn.utils.extmath import fast_logdet
from sklearn.utils.validation import check_array, check_is_fitted, validate_data

from opendp._internal import _make_measurement, _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.context import _check_sized_contributions, register
from opendp.extras._utilities import to_then
from opendp.extras.numpy._make_np_mean import make_private_np_mean
from opendp.extras.numpy._make_np_sscp import then_np_sscp
from opendp.extras.numpy import make_np_clamp
from opendp.extras.sklearn._domains import SklearnFitDescriptor
from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from opendp.extras.sklearn._make_eigendecomposition import (
    then_private_eigendecomposition,
)
from opendp.mod import Domain, Measure, Measurement, Metric

if TYPE_CHECKING:  # pragma: no cover
    import numpy


@dataclass(kw_only=True, frozen=True)
class PCARelease:
    """The private PCA release consumed by :class:`PCA`."""

    mean: numpy.ndarray
    singular_values: numpy.ndarray
    components: numpy.ndarray
    n_samples: int
    n_components: int | float | str | None = None


def make_private_pca(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    d_in,
    d_out,
    *,
    clip_norm: float | None = None,
    center: str | None = None,
    num_components: int | float | str | None = None,
) -> Measurement:
    """Construct a Measurement that releases differentially private PCA parameters.

    Fitting requires an input domain with known size.
    An L2 domain bound or ``clip_norm`` is required.
    Bounds and clipping are relative to the public domain origin (zero by default).

    :param input_domain: Input array domain with public size and feature count.
    :param input_metric: Symmetric distance on input rows.
    :param output_measure: Max divergence (pure epsilon-DP).
    :param d_in: Positive integral symmetric distance; use two edits per row replacement.
    :param d_out: Positive epsilon privacy allowance at ``d_in``.
    :param clip_norm: Public positive finite L2 clipping radius, or None to use the domain's L2 bound.
    :param center: None for a privately estimated mean, or 'origin' for the public domain origin.
    :param num_components: Number of components, fraction of released variance in (0, 1), 'mle', or None for all available components.
    :return: A Measurement releasing a :class:`PCARelease`.
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")

    if input_metric != dp.symmetric_distance():
        raise ValueError("PCA currently supports symmetric_distance() only")
    if output_measure != dp.max_divergence():
        raise ValueError("PCA currently supports max_divergence() only")
    if isinstance(d_in, bool) or not isinstance(d_in, Integral) or d_in <= 0:
        raise ValueError("d_in must be a positive integer")
    d_in = int(d_in)
    _check_sized_contributions(input_domain, input_metric, d_in)
    if d_out <= 0:
        raise ValueError("d_out must be positive")
    if center is not None and not (isinstance(center, str) and center == "origin"):
        raise ValueError("center must be None or 'origin'")

    desc = input_domain.cast(dp.numpy.NPArray2Domain)
    n_samples, n_features = desc.size, desc.num_columns
    if n_samples is None or n_features is None:
        raise ValueError("PCA requires known input-domain size and num_columns")
    if n_samples < 2 or n_features < 1:
        raise ValueError("PCA requires at least two samples and one feature")
    if isinstance(num_components, bool):
        raise ValueError(
            "num_components must be None, an integer, a fraction, or 'mle'"
        )
    if isinstance(num_components, int):
        if not 1 <= num_components <= min(n_samples, n_features):
            raise ValueError(
                "num_components must be between 1 and min(n_samples, n_features)"
            )
    elif isinstance(num_components, float):
        if not 0 < num_components < 1:
            raise ValueError("num_components must be in (0, 1) when fractional")
    elif isinstance(num_components, str) and num_components == "mle":
        if n_samples < n_features:
            raise ValueError("num_components='mle' requires n_samples >= n_features")
    elif num_components is not None:
        raise ValueError(
            "num_components must be None, an integer, a fraction, or 'mle'"
        )
    num_eigenvectors = (
        num_components
        if isinstance(num_components, int)
        else min(n_samples, n_features)
    )
    weights = [1.0] * num_eigenvectors

    origin = np.array(
        0.0 if desc.origin is None else desc.origin, dtype=float, copy=True
    )
    if origin.ndim == 0:
        origin = np.full(n_features, origin.item())
    radius = desc.norm if desc.p == 2 else None
    if clip_norm is not None:
        clip_norm = float(clip_norm)
        if not np.isfinite(clip_norm) or clip_norm <= 0:
            raise ValueError("clip_norm must be positive and finite")
        radius = clip_norm if radius is None else min(radius, clip_norm)
    if radius is None:
        raise ValueError("PCA requires an L2-bounded input domain or clip_norm")
    if radius <= 0 or not np.isfinite(radius) or not np.all(np.isfinite(origin)):
        raise ValueError("PCA requires a positive finite L2 radius and finite origin")

    # An already bounded input needs no clipping. Domains permitting NaNs
    # still require sanitization before finite-valued means and covariance.
    t_clip = None
    ball_domain = input_domain
    if desc.p != 2 or desc.norm is None or radius < desc.norm or desc.nan:
        t_clip = make_np_clamp(input_domain, input_metric, radius, p=2, origin=origin)
        ball_domain = t_clip.output_domain
    centered_domain = dp.numpy.array2_domain(
        T=float,
        size=n_samples,
        num_columns=n_features,
        norm=radius,
        p=2,
        nan=False,
    )
    estimate_mean = center is None
    mean_budget = _smaller(float(d_out) / 3) if estimate_mean else 0.0
    eigen_budget = _smaller(float(d_out) / 3 * 2) if estimate_mean else float(d_out)

    def eigendecomposition(reference):
        t_reclip = (
            make_np_clamp(ball_domain, input_metric, radius, p=2, origin=reference)
            if estimate_mean
            else None
        )
        t_center = _make_transformation(
            t_reclip.output_domain if t_reclip is not None else ball_domain,
            input_metric,
            centered_domain,
            input_metric,
            lambda data: np.asarray(data, dtype=float) - reference,
            lambda distance: distance,
        )
        if t_reclip is not None:
            t_center = t_reclip >> t_center
        t_sscp = t_center >> then_np_sscp()
        measurement = dp.binary_search_chain(
            lambda scale: t_sscp
            >> then_private_eigendecomposition(
                output_measure, scale, scale, weights=weights
            ),
            d_in=d_in,
            d_out=eigen_budget,
            T=float,
        )

        return measurement >> _new_pure_function(
            lambda out: PCARelease(
                mean=np.array(reference, copy=True),
                singular_values=np.sqrt(np.maximum(out[0], 0))[::-1],
                components=out[1].T,
                n_samples=n_samples,
                n_components=num_components,
            )
        )

    if not estimate_mean:
        measurement = eigendecomposition(origin)
    else:
        # Bound ||x-c||_1 <= sqrt(d) R without changing the L2-clipped rows.
        # Round both sqrt and multiplication upwards for conservative sensitivity.
        l1_radius = np.nextafter(
            np.nextafter(np.sqrt(n_features), np.inf) * radius, np.inf
        )
        l1_domain = dp.numpy.array2_domain(
            **(
                asdict(ball_domain.descriptor)
                | {
                    "norm": l1_radius,
                    "p": 1,
                    "origin": origin,
                    "T": float,
                }
            )
        )
        t_l1 = _make_transformation(
            ball_domain,
            input_metric,
            l1_domain,
            input_metric,
            lambda data: np.asarray(data, dtype=float),
            lambda distance: distance,
        )
        m_mean = dp.binary_search_chain(
            lambda scale: t_l1 >> make_private_np_mean(l1_domain, input_metric, scale),
            d_in=d_in,
            d_out=mean_budget,
            T=float,
        )
        projection = make_np_clamp(
            dp.numpy.array2_domain(T=float, size=1, num_columns=n_features),
            input_metric,
            radius,
            p=2,
            origin=origin,
        )
        m_mean = m_mean >> _new_pure_function(lambda mean: projection(mean[None, :])[0])
        compositor = dp.c.make_adaptive_composition(
            ball_domain,
            input_metric,
            output_measure,
            d_in=d_in,
            d_mids=[mean_budget, eigen_budget],
        )

        def function(data):
            qbl = compositor(data)
            reference = qbl(m_mean)
            return qbl(eigendecomposition(reference))

        measurement = _make_measurement(
            ball_domain,
            input_metric,
            output_measure,
            function,
            lambda distance: 0.0 if distance < 2 else compositor.map(distance),
        )

    return t_clip >> measurement if t_clip is not None else measurement


then_private_pca = to_then(make_private_pca)
register(make_private_pca)


class PCA(
    MeasurementMixin,
    TransformationMixin,
    ClassNamePrefixFeaturesOutMixin,
    TransformerMixin,
    BaseEstimator,
):
    """Differentially private principal component analysis (PCA).

    Linear dimensionality reduction with privately estimated principal axes.
    Accepts dense NumPy arrays and provides the transformation interface of
    `sklearn.decomposition.PCA <https://scikit-learn.org/stable/modules/generated/sklearn.decomposition.PCA.html>`_.

    Fitting requires an input domain with known size.
    An L2 domain bound or ``clip_norm`` is required.
    Only pure epsilon-DP is supported.
    Use ``privacy_unit=dp.unit_of(contributions=2)`` to protect one row replacement.

    :param n_components: Number of components to retain, from 1 to
        ``min(n_samples, n_features)``; a fraction of released variance in (0, 1);
        'mle' to estimate the dimension; or None for all available components.
        'mle' requires ``n_samples >= n_features`` and retains at least one component.
    :param whiten: Whether to scale projections by the inverse square roots of
        the released variances. Defaults to False.
    :param clip_norm: Public positive finite L2 clipping radius around the domain
        origin (zero by default). None uses the domain's L2 bound. If both bounds
        are supplied, the smaller radius applies.
    :param center: None uses a privately estimated mean. 'origin' uses the public
        domain origin, which need not equal the empirical mean.

    :ivar components_: Released principal axes, one per row.
    :ivar mean_: Privately estimated mean, or the public origin when ``center='origin'``.
    :ivar explained_variance_: Variances derived from released eigenvalues.
    :ivar explained_variance_ratio_: Fractions of total released variance.
    :ivar singular_values_: Singular values derived from released eigenvalues.
    :ivar noise_variance_: Mean discarded variance in the probabilistic PCA model.
    :ivar n_components_: Number of retained components.
    :ivar n_features_in_: Number of fitted input features.
    :ivar n_samples_: Public number of samples used in fitting.

    Example:

    .. code:: pycon

        >>> import numpy as np
        >>> import opendp.prelude as dp
        >>> dp.enable_features("contrib", "idealized-numerics")
        >>> X = np.array([[-1., -1.], [0., 1.], [1., 0.]])
        >>> domain = dp.sklearn.sklearn_fit_domain(
        ...     dp.numpy.array2_domain(T=float, size=3, num_columns=2)
        ... )
        >>> pca = dp.sklearn.decomposition.PCA(n_components=1, clip_norm=1.0)
        >>> _ = pca.fit(
        ...     X, domain=domain,
        ...     privacy_unit=dp.unit_of(contributions=2),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ... )
        >>> pca.transform(np.zeros((2, 2))).shape
        (2, 1)

    Reference: Kareem Amin et al., `Differentially Private Covariance Estimation
    <https://papers.nips.cc/paper_files/paper/2019/hash/4158f6d19559955bae372bb00f6204e4-Abstract.html>`_,
    NeurIPS, 2019.
    """

    # These operate solely on the released fitted attributes.
    get_covariance = SklearnPCA.get_covariance
    get_precision = SklearnPCA.get_precision
    score = SklearnPCA.score

    def __init__(self, n_components=None, whiten=False, clip_norm=None, center=None):
        self.n_components = n_components
        self.whiten = whiten
        self.clip_norm = clip_norm
        self.center = center

    @property
    def _n_features_out(self):
        return self.n_components_

    def make_measurement(
        self,
        input_domain,
        input_metric,
        output_measure,
        d_in,
        d_out,
    ) -> Measurement:
        fit_desc = input_domain.cast(SklearnFitDescriptor)
        X_domain = fit_desc.X
        m_X = make_private_pca(
            X_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            clip_norm=self.clip_norm,
            center=self.center,
            num_components=self.n_components,
        )
        # Removing one complete sklearn row removes at most one X row. Thus
        # this estimator-specific wrapper has the same privacy map as m_X.
        return _make_measurement(
            input_domain,
            input_metric,
            m_X.output_measure,
            lambda data: m_X(data.X),
            m_X.map,
        )

    def _ingest_release(self, release: PCARelease) -> None:
        np = import_optional_dependency("numpy")
        import_optional_dependency("sklearn")
        from sklearn.decomposition._pca import _infer_dimension
        from sklearn.utils.extmath import svd_flip

        n_samples = release.n_samples
        mean = np.array(release.mean, copy=True)
        singular_values = np.array(release.singular_values, copy=True)
        components = np.array(release.components, copy=True)
        n_features = components.shape[1]
        U, components = svd_flip(components.T, components)
        del U

        explained_variance = singular_values**2 / (n_samples - 1)
        total_variance = np.sum(explained_variance)
        explained_variance_ratio = (
            explained_variance / total_variance
            if total_variance > 0
            else np.zeros_like(explained_variance)
        )

        # Public parameters were validated when the measurement was constructed.
        n_components = release.n_components
        if n_components is None:
            n_components = min(n_samples, n_features)
        elif n_components == "mle":
            # sklearn may select zero for one feature or a zero spectrum.
            n_components = max(1, _infer_dimension(explained_variance, n_samples))
        elif isinstance(n_components, float):
            # With no released variance, one component is sufficient.
            n_components = (
                1
                if total_variance == 0
                else np.searchsorted(
                    np.cumsum(explained_variance_ratio), n_components, side="right"
                )
                + 1
            )
        n_components = min(int(n_components), n_samples, n_features)

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

    def transform(self, X):
        """Project rows, including empty batches, using released PCA parameters."""
        np = import_optional_dependency("numpy")
        check_is_fitted(self)
        X = validate_data(
            self,
            X,
            reset=False,
            dtype=[np.float64, np.float32],
            accept_sparse=("csr", "csc"),
            ensure_all_finite=False,
            ensure_min_samples=0,
        )
        with np.errstate(all="ignore"):
            transformed = X @ self.components_.T
            transformed -= self.mean_ @ self.components_.T
            if self.whiten:
                scale = np.sqrt(self.explained_variance_)
                scale = np.maximum(scale, np.finfo(scale.dtype).eps)
                transformed /= scale
        return transformed

    def inverse_transform(self, X):
        """Reconstruct rows, including empty batches, from released PCA parameters."""
        np = import_optional_dependency("numpy")
        check_is_fitted(self)
        X = check_array(
            X,
            dtype=[np.float64, np.float32],
            ensure_all_finite=False,
            ensure_min_samples=0,
        )
        if X.shape[1] != self.n_components_:
            raise ValueError("Input width must match the fitted number of components")
        with np.errstate(all="ignore"):
            components = (
                np.sqrt(self.explained_variance_[:, None]) * self.components_
                if self.whiten
                else self.components_
            )
            return X @ components + self.mean_

    def score_samples(self, X):
        """Log-likelihood of each row under the released probabilistic PCA model."""
        np = import_optional_dependency("numpy")
        check_is_fitted(self)
        X = validate_data(
            self,
            X,
            reset=False,
            dtype=[np.float64, np.float32],
            ensure_all_finite=False,
            ensure_min_samples=0,
        )
        # Inverting a singular released covariance may fail. This depends only
        # on the already-released model, not on the records being scored.
        precision = self.get_precision()
        with np.errstate(all="ignore"):
            centered = X - self.mean_
            scores = -0.5 * np.sum(centered * (centered @ precision), axis=1)
            scores -= 0.5 * (
                self.n_features_in_ * np.log(2.0 * np.pi) - fast_logdet(precision)
            )
        return scores

    def rowwise_output_domain(self, input_domain, *, method):
        """Describe the output domain for rowwise PCA inference."""
        import opendp.prelude as dp

        if method not in ("transform", "inverse_transform", "score_samples"):
            raise ValueError(f"Method {method!r} does not support rowwise inference")
        check_is_fitted(self)
        desc = input_domain.cast(dp.numpy.NPArray2Domain)
        if method == "transform":
            input_width, output_width = self.n_features_in_, self.n_components_
        elif method == "inverse_transform":
            input_width, output_width = self.n_components_, self.n_features_in_
        else:  # score_samples
            input_width, output_width = self.n_features_in_, 1

        if desc.num_columns != input_width:
            raise ValueError("input_domain must match the fitted number of features")
        if desc.T not in ("f32", "f64"):
            raise ValueError("Rowwise PCA inference requires floating-point input")
        return dp.numpy.array2_domain(T=float, size=desc.size, num_columns=output_width)


def _smaller(v):
    """Return the next non-negative float closer to zero."""
    np = import_optional_dependency("numpy")
    if v < 0:
        raise ValueError("expected non-negative value")  # pragma: no cover
    return v if v == 0 else np.nextafter(v, -1)
