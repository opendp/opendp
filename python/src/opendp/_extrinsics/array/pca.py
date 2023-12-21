from __future__ import annotations
import opendp.prelude as dp

from opendp.extrinsics.array.clamp import make_np_clamp
from opendp.extrinsics.norm import then_l2_to_l1_norm
from opendp.extrinsics.register import register_measurement
from opendp.extrinsics.array.private_eigenvector import make_private_eigenvectors
from opendp.extrinsics.array.eigenvalues import make_eigenvalues
from opendp.extrinsics.array.covariance import make_np_cov
from opendp.extrinsics.array.stats import make_private_np_mean
from opendp.extrinsics.composition import make_stateful_sequential_composition


def make_pca(input_domain, input_metric, unit_epsilon, num_components=None, norm=None):
    """
    :param input_domain: instance of `np_array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param unit_epsilon: ε-expenditure assuming one changed record in the input data
    :param num_components: optional, number of eigenvectors to release
    :param norm: clamp each row to this norm bound
    """
    import numpy as np
    dp.assert_features("contrib", "floating-point")

    descriptor = input_domain.descriptor
    privacy_measure = dp.max_divergence(T=descriptor["T"])

    if "size" not in descriptor:
        raise ValueError("input_domain's size must be known")

    if "num_columns" not in descriptor:
        raise ValueError("input_domain's num_columns must be known")
    
    if "norm" in descriptor and descriptor["ord"] != 2:
        raise ValueError("input_domain's norm must be an L2 norm")

    num_columns = descriptor["num_columns"]
    if num_columns < 1:
        raise ValueError("input_domain's num_columns must be gte 1")

    # if number of components is not specified, default to num_columns
    num_components = num_components or num_columns
    # the last eigvec is implicit if all other eigvecs are released
    num_eigvec_releases = min(num_components, num_columns - 1)

    # split budget evenly three ways if origin unknown, else 2
    origin = descriptor.get("origin")
    weights = np.array([1.0] * (3 if origin is None else 2))
    epsilons = [unit_epsilon * w_i / sum(weights) for w_i in weights]

    # make releases under the assumption that d_in is 2. For any other d_in,
    unit_d_in = 2
    compositor = make_stateful_sequential_composition(
        input_domain, input_metric, privacy_measure, d_in=unit_d_in, d_mids=epsilons
    )

    def function(data):
        nonlocal origin, input_domain
        epsilon_state = list(reversed(epsilons))

        qbl = compositor(data)

        # shift the data
        if "origin" not in descriptor:
            m_mean = dp.binary_search_chain(
                lambda s: make_private_np_mean(
                    input_domain, input_metric, s, norm=norm, ord=1.0
                ),
                d_in=unit_d_in,
                d_out=epsilon_state.pop(),
                T=float,
            )
            origin = qbl(m_mean)

        new_desc = {**descriptor, "origin": np.zeros_like(origin)}
        prior_input_domain, input_domain = input_domain, dp.np_array2_domain(**new_desc)

        t_offset = dp.t.make_user_transformation(
            prior_input_domain,
            input_metric,
            dp.np_array2_domain(**new_desc),
            input_metric,
            lambda arg: arg - origin,
            lambda d_in: d_in,
        )
        qbl(t_offset)

        # scale the data
        if "norm" not in descriptor or descriptor["norm"] > norm:
            t_clamp = make_np_clamp(input_domain, input_metric, norm, ord=2)
            input_domain = t_clamp.output_domain
            qbl(t_clamp)

        t_cov = make_np_cov(input_domain, input_metric)
        qbl(t_cov)

        t_eigvals = make_eigenvalues(*t_cov.output_space) >> then_l2_to_l1_norm()
        m_eigvals = dp.binary_search_chain(
            lambda s: t_eigvals >> dp.m.then_laplace(s),
            unit_d_in,
            epsilon_state.pop(),
        )
        m_eigvecs = make_private_eigenvectors(
            *t_cov.output_space,
            [epsilon_state.pop() / num_eigvec_releases] * num_eigvec_releases,
        )

        return origin, qbl(m_eigvals), qbl(m_eigvecs)

    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        privacy_measure,
        function,
        lambda d_in: d_in // unit_d_in * unit_epsilon,
        TO="ExtrinsicObject",
    )


then_pca = register_measurement(make_pca)

try:
    from sklearn.decomposition._pca import PCA as SKLPCA
except ImportError as e:
    class SKLPCA(object):
        def __init__(*args, **kwargs):
            raise e

class PCA(SKLPCA):
    def __init__(
        self,
        *,
        epsilon: int,
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

    def get_measurement(self):
        input_domain = dp.np_array2_domain(
            num_columns=self.n_features_in_, size=self.n_samples, T=float
        )
        input_metric = dp.symmetric_distance()
        # from opendp.extrinsics.array.pca_consolidated import make_pca

        return make_pca(
            input_domain,
            input_metric,
            self.epsilon / self.n_changes * 2,
            num_components=self.n_components,
            norm=self.row_norm,
        )

    @property
    def n_features(self):
        return self.n_features_in_

    def _fit(self, X):
        import numpy as np
        from sklearn.utils.extmath import stable_cumsum, svd_flip
        from sklearn.decomposition._pca import _infer_dimension
        meas = self.get_measurement()
        self.mean_, eigvals, eigvecs = meas(X)

        S = np.sqrt(eigvals)
        U = eigvecs

        # flip eigenvectors' sign to enforce deterministic output
        U, Vt = svd_flip(U, U.T)

        components_ = Vt

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
            # passed. More discussion in issue: #15669
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

    def _validate_params(*args, **kwargs):
        pass
