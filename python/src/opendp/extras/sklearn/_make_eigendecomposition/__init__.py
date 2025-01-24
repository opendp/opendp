from __future__ import annotations
from typing import Sequence

from opendp.extras._utilities import register_measurement
from opendp.extras.sklearn._make_eigenvector import then_private_eigenvectors
from opendp.extras.sklearn._make_eigenvalues import then_eigenvalues
from opendp.extras.numpy._make_np_sscp import make_np_sscp

from opendp.mod import Domain, Metric, Measurement


def make_private_np_eigendecomposition(
    input_domain: Domain,
    input_metric: Metric,
    eigvals_epsilon: float,
    eigvecs_epsilons: Sequence[float],
    num_components: int | None = None,
) -> Measurement:
    """Construct a Measurement that releases eigenvalues and eigenvectors.

    :param input_domain: instance of `array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param eigvals_epsilon: eigvals ε-expenditure per changed record in the input data
    :param eigvecs_epsilons: eigvecs ε-expenditures per changed record in the input data
    :param num_components: optional, number of eigenvectors to release. defaults to num_columns from input_domain

    :return: a Measurement that computes a tuple of (eigvals, eigvecs)
    """
    import opendp.prelude as dp

    dp.assert_features("contrib", "floating-point")
    input_desc = input_domain.descriptor

    if input_desc.size is None:
        raise ValueError("input_domain's size must be known")  # pragma: no cover

    if input_desc.num_columns is None:
        raise ValueError("input_domain's num_columns must be known")  # pragma: no cover

    if input_desc.p != 2:
        raise ValueError("input_domain's norm must be an L2 norm")  # pragma: no cover

    if input_desc.num_columns < 1:
        raise ValueError("input_domain's num_columns must be >= 1")  # pragma: no cover

    if num_components is not None and num_components < 1:
        raise ValueError("num_components must be least one")  # pragma: no cover

    # if number of components is not specified, default to num_columns
    num_components = num_components or input_desc.num_columns

    t_sscp = make_np_sscp(
        input_domain, input_metric, dp.symmetric_distance()
    )

    t_eigvals = t_sscp.output_space >> then_eigenvalues()
    m_eigvals = dp.binary_search_chain(  # type: ignore[misc]
        lambda s: t_eigvals >> dp.m.then_laplace(s),
        d_in=2,  # the unit d_in: one change = 1 addition + 1 removal
        d_out=eigvals_epsilon,
    )
    m_eigvecs = t_sscp.output_space >> then_private_eigenvectors(
        eigvecs_epsilons,
    )
    return t_sscp >> dp.c.make_basic_composition([m_eigvals, m_eigvecs])


# generate then variant of the constructor
then_private_np_eigendecomposition = register_measurement(
    make_private_np_eigendecomposition
)
