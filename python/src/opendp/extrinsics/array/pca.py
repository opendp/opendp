import opendp.prelude as dp

from opendp.extrinsics.array.clamp import make_np_clamp
from opendp.extrinsics.norm import then_l2_to_l1_norm
from opendp.extrinsics.register import register_measurement
from opendp.extrinsics.array.private_eigenvector import make_private_eigenvectors
from opendp.extrinsics.array.eigenvalues import make_eigenvalues
from opendp.extrinsics.array.covariance import make_np_cov
from opendp.extrinsics.array.stats import make_private_np_mean
from opendp.extrinsics.composition import make_stateful_sequential_composition
import numpy as np


def make_pca(input_domain, input_metric, unit_epsilon, components=None, norm=None):
    dp.assert_features("contrib")
    components = components or input_domain.descriptor["num_columns"]

    descriptor = input_domain.descriptor
    privacy_measure = dp.max_divergence(T=descriptor["T"])

    if "size" not in descriptor:
        raise ValueError("dataset size must be known")

    num_columns = descriptor["num_columns"]
    origin = descriptor.get("origin")

    weights = np.array([1.0] * (3 if origin is None else 2))
    epsilons = [unit_epsilon * w_i / sum(weights) for w_i in weights]

    compositor = make_stateful_sequential_composition(
        input_domain, input_metric, privacy_measure, 2, epsilons
    )

    def function(data):
        nonlocal origin, input_domain
        epsilon_state = list(reversed(epsilons))

        qbl = compositor(data)

        # shift the data
        if "origin" not in descriptor:
            m_mean = dp.binary_search_chain(
                lambda s: make_private_np_mean(
                    input_domain, input_metric, s, norm=norm, ord=1, origin=origin
                ),
                d_in=2,
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
        if "norm" not in descriptor:
            t_clamp = make_np_clamp(input_domain, input_metric, norm)
            input_domain = t_clamp.output_domain
            qbl(t_clamp)

        t_cov = make_np_cov(input_domain, input_metric)
        qbl(t_cov)

        t_eigvals = make_eigenvalues(*t_cov.output_space) >> then_l2_to_l1_norm()
        m_eigvals = dp.binary_search_chain(
            lambda s: t_eigvals >> dp.m.then_laplace(s),
            2,
            epsilon_state.pop(),
        )
        m_eigvecs = make_private_eigenvectors(
            *t_cov.output_space, [epsilon_state.pop() / num_columns] * (num_columns - 1)
        )

        return origin, qbl(m_eigvals), qbl(m_eigvecs)

    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        privacy_measure,
        function,
        lambda d_in: d_in // 2 * unit_epsilon,
        TO="ExtrinsicObject",
    )


then_pca = register_measurement(make_pca)


# from sklearn.decomposition._pca import PCA as SKLPCA

# class PCA(SKLPCA):
#     def __init__(
#         self,
#         n_components: int | float | str | None = None,
#         *,
#         copy: bool = True,
#         whiten: bool = False,
#         svd_solver: Literal["auto", "full", "arpack", "randomized"] = "auto",
#         tol: float = 0,
#         iterated_power: Literal["auto"] = "auto",
#         n_oversamples: int = 10,
#         power_iteration_normalizer: Literal["auto", "QR", "LU", "none"] = "auto",
#         random_state: int | RandomState | None = None
#     ) -> None:
#         super().__init__(
#             n_components,
#             copy=copy,
#             whiten=whiten,
#             svd_solver=svd_solver,
#             tol=tol,
#             iterated_power=iterated_power,
#             n_oversamples=n_oversamples,
#             power_iteration_normalizer=power_iteration_normalizer,
#             random_state=random_state,
#         )
