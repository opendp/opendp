"""Symmetric matrix coordinates for additive noise mechanisms."""

import math

from opendp._internal import _make_transformation
from opendp._lib import import_optional_dependency


def make_np_svec(input_domain, input_metric, p=2):
    """Encode the upper triangle, scaling off-diagonal entries by sqrt(2).

    This is an isometry from Frobenius distance to L2 distance. For L1
    output distance, use the usual sqrt(number of coordinates) bound.
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")
    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("Expected a symmetric matrix domain")
    desc = input_domain.descriptor
    if input_metric != dp.l2_distance(T=desc.T):
        raise ValueError("Expected Frobenius distance")
    if p not in (1, 2):
        raise ValueError("p must be 1 or 2")

    upper = np.triu_indices(desc.num_features)
    off_diagonal = upper[0] != upper[1]
    num_entries = len(upper[0])

    def svec(matrix):
        values = np.asarray(matrix[upper], dtype=float)
        values[off_diagonal] *= math.sqrt(2.0)
        return values

    return _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=float, nan=False), size=num_entries),
        (dp.l2_distance if p == 2 else dp.l1_distance)(T=float),
        svec,
        (lambda distance: distance)
        if p == 2
        else (
            lambda distance: distance * math.nextafter(math.sqrt(num_entries), math.inf)
        ),
    )


def unsvec(values, dimension):
    """Reconstruct a symmetric matrix from its scaled upper triangle."""
    np = import_optional_dependency("numpy")
    upper = np.triu_indices(dimension)
    values = np.array(values, dtype=float, copy=True)
    values[upper[0] != upper[1]] /= math.sqrt(2.0)
    matrix = np.zeros((dimension, dimension))
    matrix[upper] = values
    matrix[(upper[1], upper[0])] = values
    return matrix
