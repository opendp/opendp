'''
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.decomposition``.    

See also our :ref:`tutorial on diffentially private PCA <dp-pca>`.
'''

from __future__ import annotations
from typing import NamedTuple, Optional, TYPE_CHECKING, Sequence
from opendp.extras.numpy import then_np_clamp
from opendp.context import register
from opendp.extras._utilities import to_then
from opendp.extras.numpy._make_np_mean import make_private_np_mean
from opendp.extras.sklearn._make_eigendecomposition import then_private_np_eigendecomposition
from opendp.mod import Domain, Measurement, Metric
from opendp._lib import import_optional_dependency
from opendp._internal import _make_measurement, _make_transformation

if TYPE_CHECKING: # pragma: no cover
    import numpy # type: ignore[import-not-found]


class PCAEpsilons(NamedTuple):
    '''
    Tuple used to describe the ε-expenditure per changed record in the input data
    '''
    eigvals: float
    eigvecs: Sequence[float]
    mean: Optional[float]


PCAEpsilons.eigvals.__doc__ = 'ε-expenditure to estimate the eigenvalues'
PCAEpsilons.eigvecs.__doc__ = 'ε-expenditure to estimate the eigenvectors'
PCAEpsilons.mean.__doc__ = ''  """ε-expenditure to estimate the mean.

A portion of the budget is used to estimate the mean because the OpenDP PCA algorithm 
releases an eigendecomposition of the sum of squares and cross-products matrix (SSCP), 
not of the covariance matrix. 
If the data is centered beforehand (either by a prior from the user or by privately estimating the mean and then centering), 
then PCA will correspond to the covariance matrix, as expected, 
because the SSCP matrix of centered data is equivalent to a scaled covariance matrix.

If the data is not centered (or the mean is poorly estimated), then the first eigenvector will be dominated by the true mean.
"""


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

    :return: a Measurement that computes a tuple of (mean, S, Vt)
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

    def _eig_to_SVt(decomp):
        eigvals, eigvecs = decomp
        return np.sqrt(np.maximum(eigvals, 0))[::-1], eigvecs.T

    def _make_eigdecomp(norm, origin):
        return (
            (input_domain, input_metric)
            >> then_np_clamp(norm, p=2, origin=origin)
            >> then_center()
            >> then_private_np_eigendecomposition(eigvals_epsilon, eigvecs_epsilons)
            >> (lambda out: PCAResult(origin, *_eig_to_SVt(out)))
        )

    if input_desc.norm is not None:
        if mean_epsilon is not None:
            raise ValueError("mean_epsilon should be zero because origin is known")  # pragma: no cover
        norm = input_desc.norm if norm is None else norm
        norm = min(input_desc.norm, norm)
        return _make_eigdecomp(norm, input_desc.origin)
    elif norm is None:
        raise ValueError("must have either bounded `input_domain` or specify `norm`")  # pragma: no cover


    # make releases under the assumption that d_in is 2.
    unit_d_in = 2

    compositor = dp.c.make_adaptive_composition(
        input_domain,
        input_metric,
        dp.max_divergence(),
        d_in=unit_d_in,
        d_mids=[mean_epsilon, _make_eigdecomp(norm, 0).map(unit_d_in)],
    )

    def _function(data):
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
        return qbl(_make_eigdecomp(norm, origin))

    return _make_measurement(
        input_domain,
        input_metric,
        compositor.output_measure,
        _function,
        compositor.map,
    )


# generate then variant of the constructor
then_private_pca = to_then(make_private_pca)
register(make_private_pca)


class PCA:
    '''
    DP wrapper for `sklearn's PCA <https://scikit-learn.org/stable/modules/generated/sklearn.decomposition.PCA.html>`_.
    This implementation is based on `Differentially Private Covariance Estimation <https://papers.nips.cc/paper_files/paper/2019/hash/4158f6d19559955bae372bb00f6204e4-Abstract.html>`_ by Kareem Amin, et al.

    Trying to create an instance without sklearn installed will raise an ``ImportError``.
    
    See the :ref:`tutorial on diffentially private PCA <dp-pca>` for details.

    :param whiten: Mirrors the corresponding sklearn parameter:
        When ``True`` (``False`` by default) the ``components_`` vectors are multiplied
        by the square root of n_samples and then divided by the singular values
        to ensure uncorrelated outputs with unit component-wise variances.
    '''
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
    ) -> None:  # pragma: no cover
        # Error if constructor called without dependency:
        import_optional_dependency('sklearn.decomposition')
        # used for mypy typing
        self.n_samples_ = None
        self.components_ = None
        self.n_components_ = None
        self.explained_variance_ = None
        self.explained_variance_ratio_ = None
        self.singular_values_ = None

    @property
    def n_features(self):
        '''
        Number of features
        '''
        ...

    def fit(self, X, y=None):
        '''
        Fit the model with X.

        :param X: Training data, where ``n_samples`` is the number of samples and ``n_features`` is the number of features.
        :param y: Ignored
        '''
        ...

    # this overrides the scikit-learn method to instead use the opendp-core constructor
    def _fit(self, X):
        ...

    def _prepare_fitter(self) -> Measurement:  # type: ignore[empty-body]
        """Returns a measurement that computes the mean and eigendecomposition,
        and then apply those releases to self."""
        ...

    def _postprocess(self, values):
        """A function that applies a release of the mean and eigendecomposition to self"""
        ...

    def measurement(self) -> Measurement:  # type: ignore[empty-body]
        """Return a measurement that releases a fitted model."""
        ...

    def _validate_params(*args, **kwargs):
        ...


_decomposition = import_optional_dependency('sklearn.decomposition', False)
if _decomposition is not None:
    class PCA(_decomposition.PCA):  # type: ignore  # noqa: F811
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
            '''
            Number of features
            '''
            return self.n_features_in_

        # This isn't strictly necessary, since we just call the superclass method,
        # but this lets us document a frequently used method,
        # and avoids a number of mypy warnings.
        def fit(self, X, y=None):
            '''
            Fit the model with X.

            :param X: Training data, where ``n_samples`` is the number of samples and ``n_features`` is the number of features.
            :param y: Ignored
            '''
            return super().fit(X)

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