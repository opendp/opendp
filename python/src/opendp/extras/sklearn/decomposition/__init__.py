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

from dataclasses import asdict, dataclass, replace
from typing import TYPE_CHECKING

from sklearn.decomposition import PCA as SklearnPCA
from sklearn.utils.validation import check_is_fitted

from opendp._internal import _make_measurement, _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.context import register
from opendp.extras._utilities import to_then
from opendp.extras.numpy._make_np_mean import make_private_np_mean
from opendp.extras.numpy import make_np_clamp
from opendp.extras.sklearn._domains import SklearnFitDomain, sklearn_fit_domain
from opendp.extras.sklearn._estimator import DPTransformer
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
    n_samples: int
    n_components: int | float | str | None = None


def make_private_pca(
    input_domain: Domain,
    input_metric: Metric,
    output_measure,
    d_in,
    d_out,
    *,
    clip_norm: float | None = None,
    center: str | None = None,
    num_components: int | float | str | None = None,
) -> Measurement:
    """Construct pure-DP PCA of data in a public L2 ball.

    ``clip_norm`` clips around the domain origin (zero by default), retaining
    any tighter certified L2 radius. Without it, the domain must supply a bound.
    ``center=None`` estimates and projects a private mean, then reclips around
    that mean before eigendecomposition. ``center='origin'`` instead subtracts
    the public ball origin; it makes no assertion about the empirical mean.

    Private centering gets one third of the budget, eigendecomposition gets the
    rest. Eigenvalues and Bingham samples split the eigen budget equally, with
    equal budgets per sample. With no samples, eigenvalues get the entire share.
    Use ``d_in=2`` to protect one replacement in the exact-size input domain.

    :param input_domain: Domain of the input NumPy array, with public size and width.
    :param input_metric: Symmetric distance on input rows.
    :param output_measure: Max divergence (pure epsilon-DP).
    :param d_in: Input distance to protect, either one or two symmetric edits.
    :param d_out: Positive epsilon privacy allowance at ``d_in``.
    :param clip_norm: Optional positive finite L2 clipping radius.
    :param center: Privately estimate the mean (None), or use the public 'origin'.
    :param num_components: Number of components, variance fraction, 'mle', or None.
    :return: Measurement releasing PCA parameters in a :class:`PCARelease`.
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")

    if input_metric != dp.symmetric_distance():
        raise ValueError("PCA currently supports symmetric_distance() only")
    if output_measure != dp.max_divergence():
        raise ValueError("PCA currently supports max_divergence() only")
    if d_in <= 0 or d_out <= 0:
        raise ValueError("d_in and d_out must be positive")
    if d_in not in (1, 2):
        raise ValueError("PCA currently supports d_in=1 or d_in=2")
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
    num_samples = min(
        num_components if isinstance(num_components, int) else n_features,
        n_features - 1,
    )

    origin = np.array(
        0.0 if desc.origin is None else desc.origin, dtype=float, copy=True
    )
    if origin.ndim == 0:
        origin = np.full(n_features, origin.item())
    if origin.shape != (n_features,):
        raise ValueError("domain origin must be scalar or have shape (n_features,)")
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

    # A certified tighter ball needs no clipping. Domains permitting NaNs
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
        # Low-level mechanisms are calibrated at two symmetric edits.
        budget_at_two = 2 * eigen_budget / float(d_in)
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
        while True:
            eigvals_epsilon = budget_at_two / 2 if num_samples else budget_at_two
            eigvecs_epsilons = (
                [_smaller(budget_at_two / 2 / num_samples)] * num_samples
                if num_samples
                else []
            )
            measurement = t_center >> then_private_np_eigendecomposition(
                eigvals_epsilon,
                eigvecs_epsilons,
            )
            spent = measurement.map(d_in)
            if spent <= eigen_budget:
                break
            # Directed-rounding error can accumulate across many samples.
            # Reduce all allocations together, preserving the fixed split.
            budget_at_two = _smaller(budget_at_two * (eigen_budget / spent))

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
        # Certify ||x-c||_1 <= sqrt(d) R without changing the L2-clipped rows.
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
            # Fixed-size sums have zero sensitivity at one symmetric edit.
            d_in=2,
            d_out=2 * mean_budget / float(d_in),
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
            compositor.map,
        )

    return t_clip >> measurement if t_clip is not None else measurement


then_private_pca = to_then(make_private_pca)
register(make_private_pca)


class PCA(SklearnPCA, DPTransformer):
    """Differentially private PCA.

    DP wrapper for `sklearn's PCA <https://scikit-learn.org/stable/modules/generated/sklearn.decomposition.PCA.html>`_.
    This implementation is based on `Differentially Private Covariance Estimation <https://papers.nips.cc/paper_files/paper/2019/hash/4158f6d19559955bae372bb00f6204e4-Abstract.html>`_ by Kareem Amin, et al.

    Trying to create an instance without sklearn installed will raise an ``ImportError``.

    See the :ref:`tutorial on diffentially private PCA <dp-pca>` for details.

    :param n_components: fraction of released variance in (0, 1), ``'mle'``, or None for all available components.
    :param whiten: Whether to scale projections using the released variances.
    :param clip_norm: Public L2 clipping radius around the domain origin; None uses the domain bound.
    :param center: None estimates a private mean; ``'origin'`` uses the public ball origin.
    """

    def __init__(self, n_components=None, whiten=False, clip_norm=None, center=None):
        super().__init__(n_components=n_components, whiten=whiten)
        self.clip_norm = clip_norm
        self.center = center

    def fit(self, X, y=None, *, privacy_unit, privacy_loss, domain=None):
        """Fit privately through the shared estimator API, never sklearn's fitter."""
        return DPTransformer.fit(
            self,
            X,
            y,
            privacy_unit=privacy_unit,
            privacy_loss=privacy_loss,
            domain=domain,
        )

    def fit_transform(self, X, y=None, **kwargs):
        """Reject releasing transformed private training rows without accounting.

        Use ``fit`` to release parameters and ``make_transform`` to preprocess
        private records within a subsequent accounted computation.
        """
        raise NotImplementedError(
            "fit_transform would release nonprivate training rows; use make_transform "
            "within an accounted private computation"
        )

    def make(
        self,
        input_domain,
        input_metric,
        output_measure,
        d_in,
        d_out,
    ) -> Measurement:
        fit_desc = input_domain.cast(SklearnFitDomain)
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

        if not isinstance(release, PCARelease):
            raise TypeError("PCA expected a PCARelease")

        n_samples = release.n_samples
        if n_samples <= 1:
            raise ValueError("PCA requires at least two samples")

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

        n_components = release.n_components
        if n_components is None:
            n_components = min(n_samples, n_features)
        elif n_components == "mle":
            if n_samples < n_features:
                raise ValueError("n_components='mle' requires n_samples >= n_features")
            # sklearn may select zero for one feature or a zero spectrum.
            # This API retains at least one component, even in those cases.
            n_components = max(1, _infer_dimension(explained_variance, n_samples))
        elif isinstance(n_components, float):
            if not 0 < n_components < 1:
                raise ValueError("n_components must be in (0, 1) when fractional")
            # With no released variance, one component is sufficient for any
            # requested fraction; searching an all-zero cumulative sum is not.
            n_components = (
                1
                if total_variance == 0
                else np.searchsorted(
                    np.cumsum(explained_variance_ratio), n_components, side="right"
                )
                + 1
            )
        elif isinstance(n_components, int) and not isinstance(n_components, bool):
            if not 1 <= n_components <= min(n_samples, n_features):
                raise ValueError(
                    "n_components must be between 1 and min(n_samples, n_features)"
                )
        else:
            raise ValueError(
                "n_components must be None, an integer, a fraction, or 'mle'"
            )

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

    def make_transform(self, input_domain, input_metric):
        """Construct a transformation from the released PCA state.

        A SklearnFit input domain retains targets unchanged.
        Each output row depends only on that input X row and released parameters,
        so the complete sklearn rows have identity symmetric-distance stability.
        """
        import opendp.prelude as dp

        np = import_optional_dependency("numpy")
        check_is_fitted(self)
        if input_metric != dp.symmetric_distance():
            raise ValueError("PCA transform requires symmetric_distance()")
        fit_desc = input_domain.cast(SklearnFitDomain)
        desc = fit_desc.X.cast(dp.numpy.NPArray2Domain)
        if desc.num_columns != self.n_features_in_:
            raise ValueError("input_domain must match the fitted number of features")

        # Snapshot only released parameters, never the private training data.
        mean = self.mean_.copy()
        components = self.components_.copy()
        scale = None
        if self.whiten:
            scale = np.sqrt(self.explained_variance_)
            scale = np.maximum(scale, np.finfo(scale.dtype).eps)

        def transform(data):
            result = (data - mean) @ components.T
            if scale is not None:
                result /= scale
            return np.asarray(result, dtype=float)

        output_domain = dp.numpy.array2_domain(
            T=float,
            size=desc.size,
            num_columns=self.n_components_,
        )
        output_domain = sklearn_fit_domain(output_domain, fit_desc.y)
        function = lambda data: replace(data, X=transform(data.X))
        return _make_transformation(
            input_domain,
            input_metric,
            output_domain,
            input_metric,
            function,
            lambda distance: distance,
        )


def _smaller(v):
    """Return the next non-negative float closer to zero."""
    np = import_optional_dependency("numpy")
    if v < 0:
        raise ValueError("expected non-negative value")  # pragma: no cover
    return v if v == 0 else np.nextafter(v, -1)
