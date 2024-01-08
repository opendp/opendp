from __future__ import annotations

from opendp.extrinsics.make_l2_to_l1_norm import then_l2_to_l1_norm
from opendp.extrinsics._utilities import register_measurement
from opendp.extrinsics._make_np_eigenvector import then_private_np_eigenvectors
from opendp.extrinsics._make_np_eigenvalues import then_np_eigenvalues
from opendp.extrinsics._make_np_xTx import make_np_xTx

from opendp.mod import Measurement


def make_private_np_eigendecomposition(
    input_domain, input_metric, eigvals_epsilon, eigvecs_epsilons, num_components=None
) -> Measurement:
    """
    :param input_domain: instance of `np_array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param eigvals_epsilon: eigvals ε-expenditure per changed record in the input data
    :param eigvecs_epsilons: eigvecs ε-expenditures per changed record in the input data
    :param num_components: optional, number of eigenvectors to release. defaults to num_columns from input_domain

    :returns a Measurement that computes a tuple of (eigvals, eigvecs)
    """
    import opendp.prelude as dp

    dp.assert_features("contrib", "floating-point")

    if input_domain.size is None:
        raise ValueError("input_domain's size must be known")

    if input_domain.num_columns is None:
        raise ValueError("input_domain's num_columns must be known")

    if input_domain.p != 2:
        raise ValueError("input_domain's norm must be an L2 norm")

    if input_domain.num_columns < 1:
        raise ValueError("input_domain's num_columns must be >= 1")

    # if number of components is not specified, default to num_columns
    num_components = num_components or input_domain.num_columns


    t_cov = make_np_xTx(input_domain, input_metric, dp.symmetric_distance())

    t_eigvals = t_cov.output_space >> then_np_eigenvalues() >> then_l2_to_l1_norm()
    m_eigvals = dp.binary_search_chain(  # type: ignore[misc]
        lambda s: t_eigvals >> dp.m.then_laplace(s),
        d_in=2, # the unit d_in: one change = 1 addition + 1 removal
        d_out=eigvals_epsilon,
    )
    m_eigvecs = t_cov.output_space >> then_private_np_eigenvectors(
        eigvecs_epsilons,
    )
    return t_cov >> dp.c.make_basic_composition([m_eigvals, m_eigvecs])


# generate then variant of the constructor
then_private_np_eigendecomposition = register_measurement(
    make_private_np_eigendecomposition
)
