from __future__ import annotations

from collections.abc import Sequence

from opendp.context import register
from opendp.extras._utilities import to_then
from opendp.extras.sklearn._make_eigenvalues import make_private_eigenvalues
from opendp.extras.sklearn._make_eigenvector import make_private_eigenvectors
from opendp.mod import Domain, Measure, Metric, Measurement


def make_private_eigendecomposition(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    eigvals_scale: float,
    eigvecs_scale: float,
    weights: Sequence[float],
) -> Measurement:
    """Privately estimate the eigendecomposition of a symmetric matrix.

    :param input_domain: Symmetric matrix domain with a known dimension.
    :param input_metric: Joint nuclear and Frobenius distance.
    :param output_measure: Max divergence (pure epsilon-DP).
    :param eigvals_scale: Laplace noise scale for the eigenvalues.
    :param eigvecs_scale: Inverse aggregate Bingham concentration.
    :param weights: Relative Bingham concentrations, one per requested eigenvector.
        The final weight of a full basis is ignored. All eigenvalues are returned.

    :return: A tuple of eigenvalues (ascending) and eigenvectors (columns,
        estimated in leading-component order).
    """
    import opendp.prelude as dp

    return dp.c.make_composition(
        [
            make_private_eigenvalues(
                input_domain, input_metric, output_measure, eigvals_scale
            ),
            make_private_eigenvectors(
                input_domain, input_metric, output_measure, eigvecs_scale, weights
            ),
        ]
    )


then_private_eigendecomposition = to_then(make_private_eigendecomposition)
register(make_private_eigendecomposition)
