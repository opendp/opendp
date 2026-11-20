from opendp.extras._utilities import to_then, with_privacy
from opendp.extras.numpy import _nuclear_norm_distance
from opendp.mod import Domain, Metric, Transformation
from opendp._lib import import_optional_dependency
from opendp._internal import _make_transformation

# planning to make this public, but may make more API changes


def make_eigenvalues(input_domain: Domain, input_metric: Metric) -> Transformation:
    """Construct a Transformation that computes the eigenvalues of a covariance matrix.

    :param input_domain: instance of `_symmetric_matrix_domain(num_features=_)`
    :param input_metric: nuclear norm distance on symmetric matrices
    """
    np = import_optional_dependency("numpy")
    import opendp.prelude as dp

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")

    if input_metric != _nuclear_norm_distance():
        raise ValueError("input metric must be nuclear norm distance")

    input_desc = input_domain.descriptor

    return _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=input_desc.T, nan=False)),
        dp.l1_distance(T=input_desc.T),
        np.linalg.eigvalsh,
        # Lidskii-Mirsky: ||eigval(C) - eigval(C')||_1 <= ||C - C'||_*.
        lambda d_in: d_in,
    )


# generate then and private variants of the constructor
then_eigenvalues = to_then(make_eigenvalues)
make_private_eigenvalues = with_privacy(make_eigenvalues)
then_private_eigenvalues = to_then(make_private_eigenvalues)
