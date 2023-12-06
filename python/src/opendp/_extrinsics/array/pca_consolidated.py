import opendp.prelude as dp

from opendp.extrinsics.array.clamp import then_np_clamp
from opendp.extrinsics.norm import then_l2_to_l1_norm
from opendp.extrinsics.register import register_measurement
from opendp.extrinsics.array.private_eigenvector import make_private_eigenvectors
from opendp.extrinsics.array.eigenvalues import make_eigenvalues
from opendp.extrinsics.array.covariance import then_np_cov
from opendp.extrinsics.array.stats import make_private_np_mean
import numpy as np


def make_pca(input_domain, input_metric, unit_epsilon, components=None, norm=None):
    dp.assert_features("contrib")
    components = components or (input_domain.descriptor["num_columns"] - 1)

    descriptor = input_domain.descriptor
    privacy_measure = dp.max_divergence(T=descriptor["T"])

    if "size" not in descriptor:
        raise ValueError("dataset size must be known")

    origin = descriptor.get("origin")

    weights = [1 / 3, 2 / 3] if origin is None else [1]
    epsilons = [unit_epsilon * w_i / sum(weights) for w_i in weights]

    compositor = dp.c.make_sequential_composition(
        input_domain, input_metric, privacy_measure, 2, epsilons
    )

    def function(data):
        nonlocal origin, input_domain
        epsilon_state = list(reversed(epsilons))

        qbl = compositor(data)

        # shift the data
        if origin is None:
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

        t_pre = dp.t.make_user_transformation(
            prior_input_domain,
            input_metric,
            dp.np_array2_domain(**new_desc),
            input_metric,
            lambda arg: arg - origin,
            lambda d_in: d_in,
        )

        # scale the data
        if "norm" not in descriptor:
            t_pre = t_pre >> then_np_clamp(norm)

        t_pre = t_pre >> then_np_cov()

        unit_epsilon_eig = epsilon_state.pop() / 2

        m_cov = t_pre >> dp.c.make_sequential_composition(
            t_pre.output_domain,
            t_pre.output_metric,
            privacy_measure,
            2,
            [unit_epsilon_eig] * 2,
        )

        qbl_cov = qbl(m_cov)

        t_eigvals = make_eigenvalues(*t_pre.output_space) >> then_l2_to_l1_norm()
        m_eigvals = dp.binary_search_chain(
            lambda s: t_eigvals >> dp.m.then_laplace(s),
            2,
            unit_epsilon_eig,
        )

        eigvals = qbl_cov(m_eigvals)
        # eigvals = None
        m_eigvecs = make_private_eigenvectors(
            *t_pre.output_space, [unit_epsilon_eig / components] * components
        )
        eigvecs = qbl_cov(m_eigvecs)
        # eigvecs = None
        return origin, eigvals, eigvecs

    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        privacy_measure,
        function,
        lambda d_in: d_in // 2 * unit_epsilon,
        TO="ExtrinsicObject",
    )


then_pca = register_measurement(make_pca)

