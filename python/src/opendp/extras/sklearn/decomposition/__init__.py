'''
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.sklearn.decomposition``.    

See also our :ref:`tutorial on diffentially private PCA <dp-pca>`.
'''

from __future__ import annotations
from typing import NamedTuple, Optional, TYPE_CHECKING
from opendp.extras.numpy import then_np_clamp
from opendp.extras._utilities import register_measurement, to_then
from opendp.extras.numpy._make_np_mean import make_private_np_mean
from opendp.extras.sklearn._make_eigendecomposition import then_private_np_eigendecomposition
from opendp.mod import Domain, Measurement, Metric
from opendp._lib import import_optional_dependency
from opendp._internal import _make_measurement, _make_transformation

if TYPE_CHECKING: # pragma: no cover
    import numpy # type: ignore[import-not-found]


_decomposition = import_optional_dependency('sklearn.decomposition', False)
if _decomposition is not None:
    class _SKLPCA(_decomposition.PCA): # type: ignore[name-defined]
        '''
        :meta private:
        '''
        pass
else: # pragma: no cover
    class _SKLPCA(object): # type: ignore[no-redef]
        '''
        :meta private:
        '''
        def __init__(*args, **kwargs):
            raise ImportError(
                "The optional install scikit-learn is required for this functionality"
            )


class PCAEpsilons(NamedTuple):
    eigvals: float
    eigvecs: list[float]
    mean: Optional[float]


def make_private_pca(
    input_domain: Domain,
    input_metric: Metric,
    unit_epsilon: float | PCAEpsilons,
    norm: float | None = None,
    num_components=None,
) -> Measurement:
    """Construct a Measurement that returns the data mean, singular values and right singular vectors.

    :param input_domain: instance of `array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param unit_epsilon: ε-expenditure per changed record in the input data
    :param norm: clamp each row to this norm bound
    :param num_components: optional, number of eigenvectors to release. defaults to num_columns from input_domain

    :returns: a Measurement that computes a tuple of (mean, S, Vt)
    """
    import opendp.prelude as dp
    np = import_optional_dependency('numpy')

    dp.assert_features("contrib", "floating-point")

    class PCAResult(NamedTuple):
        mean: numpy.ndarray
        S: numpy.ndarray
        Vt: numpy.ndarray

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

    if isinstance(unit_epsilon, float):
        num_eigvec_releases = min(num_components, input_desc.num_columns - 1)
        unit_epsilon = _split_pca_epsilon_evenly(
            unit_epsilon, num_eigvec_releases, estimate_mean=input_desc.origin is None
        )

    if not isinstance(unit_epsilon, PCAEpsilons):
        raise ValueError("epsilon must be a float or instance of PCAEpsilons")  # pragma: no cover

    eigvals_epsilon, eigvecs_epsilons, mean_epsilon = unit_epsilon

    def eig_to_SVt(decomp):
        eigvals, eigvecs = decomp
        return np.sqrt(np.maximum(eigvals, 0))[::-1], eigvecs.T

    def make_eigdecomp(norm, origin):
        return (
            (input_domain, input_metric)
            >> then_np_clamp(norm, p=2, origin=origin)
            >> then_center()
            >> then_private_np_eigendecomposition(eigvals_epsilon, eigvecs_epsilons)
            >> (lambda out: PCAResult(origin, *eig_to_SVt(out)))
        )

    if input_desc.norm is not None:
        if mean_epsilon is not None:
            raise ValueError("mean_epsilon should be zero because origin is known")  # pragma: no cover
        norm = input_desc.norm if norm is None else norm
        norm = min(input_desc.norm, norm)
        return make_eigdecomp(norm, input_desc.origin)
    elif norm is None:
        raise ValueError("must have either bounded `input_domain` or specify `norm`")  # pragma: no cover


    # make releases under the assumption that d_in is 2.
    unit_d_in = 2

    compositor = dp.c.make_sequential_composition(
        input_domain,
        input_metric,
        dp.max_divergence(),
        d_in=unit_d_in,
        d_mids=[mean_epsilon, make_eigdecomp(norm, 0).map(unit_d_in)],
    )

    def function(data):
        nonlocal input_domain
        qbl = compositor(data)

        # find origin
        m_mean = dp.binary_search_chain(  # type: ignore[misc]
            lambda s: make_private_np_mean(
                input_domain, input_metric, s, norm=norm, p=1
            ),
            d_in=unit_d_in,
            d_out=mean_epsilon,
            T=float,
        )
        origin = qbl(m_mean)
        # make full release
        return qbl(make_eigdecomp(norm, origin))

    return _make_measurement(
        input_domain,
        input_metric,
        compositor.output_measure,
        function,
        compositor.map,
    )


# generate then variant of the constructor
then_private_pca = register_measurement(make_private_pca)


class PCA(_SKLPCA):
    # TODO: If I add a docstring here, it also tries to locate _SKLPCA, and fails
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
            raise ValueError("DP-PCA model has already been fitted")  # pragma: no cover

        input_domain = dp.numpy.array2_domain(
            num_columns=self.n_features_in_, size=self.n_samples, T=float
        )
        input_metric = dp.symmetric_distance()

        n_estimated_components = (
            self.n_components
            if isinstance(self.n_components, int)
            else self.n_features_in_
        )

        return make_private_pca(
            input_domain,
            input_metric,
            self.epsilon / self.n_changes * 2,
            norm=self.row_norm,
            num_components=n_estimated_components,
        ) >> self._postprocess

    def _postprocess(self, values):
        """A function that applies a release of the mean and eigendecomposition to self"""
        np = import_optional_dependency('numpy')
        from sklearn.utils.extmath import stable_cumsum, svd_flip # type: ignore[import]
        from sklearn.decomposition._pca import _infer_dimension # type: ignore[import]

        self.mean_, S, Vt = values
        U = Vt.T
        n_samples, n_features = self.n_samples, self.n_features_in_
        n_components = self.n_components

        # CODE BELOW THIS POINT IS FROM SKLEARN
        # flip eigenvectors' sign to enforce deterministic output
        U, Vt = svd_flip(U, Vt)

        components_ = Vt

        # Get variance explained by singular values
        explained_variance_ = (S**2) / (n_samples - 1)
        total_var = np.sum(explained_variance_)
        explained_variance_ratio_ = explained_variance_ / total_var
        singular_values_ = S # Store the singular values. 

        # Postprocess the number of components required
        if n_components == "mle":
            n_components = _infer_dimension(explained_variance_, n_samples)
        elif 0 < n_components < 1.0:
            # number of components for which the cumulated explained
            # variance percentage is superior to the desired threshold
            # side='right' ensures that number of features selected
            # their variance is always greater than n_components float
            # passed. More discussion in issue: https://github.com/scikit-learn/scikit-learn/pull/15669
            explained_variance_ratio_np = explained_variance_ratio_
            ratio_cumsum = stable_cumsum(explained_variance_ratio_np)
            n_components = np.searchsorted(ratio_cumsum, n_components, side="right") + 1

        # Compute noise covariance using Probabilistic PCA model
        # The sigma2 maximum likelihood (cf. eq. 12.46)
        if n_components < min(n_features, n_samples):
            self.noise_variance_ = np.mean(explained_variance_[n_components:])
        else:
            self.noise_variance_ = 0.0

        self.n_samples_ = n_samples
        self.components_ = components_[:n_components, :]
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


def _smaller(v):
    """returns the next non-negative float closer to zero"""
    np = import_optional_dependency('numpy')

    if v < 0:
        raise ValueError("expected non-negative value")  # pragma: no cover
    return v if v == 0 else np.nextafter(v, -1)


def _split_pca_epsilon_evenly(unit_epsilon, num_eigvec_releases, estimate_mean=False):
    num_queries = 3 if estimate_mean else 2
    per_query_epsilon = unit_epsilon / num_queries
    per_evec_epsilon = per_query_epsilon / num_eigvec_releases

    # use conservatively smaller budgets to prevent totals from exceeding total epsilon
    return PCAEpsilons(
        eigvals=per_query_epsilon,
        eigvecs=[_smaller(per_evec_epsilon)] * num_eigvec_releases,
        mean=_smaller(per_query_epsilon) if estimate_mean else None,
    )


def _make_center(input_domain, input_metric):
    import opendp.prelude as dp
    np = import_optional_dependency('numpy')

    dp.assert_features("contrib", "floating-point")

    input_desc = input_domain.descriptor

    kwargs = input_desc._asdict() | {"origin": np.zeros(input_desc.num_columns)}
    return _make_transformation(
        input_domain,
        input_metric,
        dp.numpy.array2_domain(**kwargs),
        input_metric,
        lambda arg: arg - input_desc.origin,
        lambda d_in: d_in,
    )


then_center = to_then(_make_center)