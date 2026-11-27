from opendp.extras._utilities import to_then, with_privacy
from opendp.mod import Domain, Metric, Transformation
from opendp._lib import import_optional_dependency
from opendp._internal import _make_transformation

# planning to make this public, but may make more API changes


def make_eigenvalues(input_domain: Domain, input_metric: Metric) -> Transformation:
    """Construct a Transformation that computes the eigenvalues of a covariance matrix.

    :param input_domain: instance of `_symmetric_matrix_domain(num_features=_)`
    :param input_metric: nuclear distance on symmetric matrices
    """
    np = import_optional_dependency("numpy")
    import opendp.prelude as dp

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")

    if input_metric != dp.nuclear_distance():
        raise ValueError("input metric must be nuclear distance")

    input_desc = input_domain.descriptor

    return _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=input_desc.T, nan=False)),
        dp.l1_distance(T=input_desc.T),
        np.linalg.eigvalsh,
        # Lidskii–Mirsky: sorted eigenvalues are L1-Lipschitz in nuclear distance.
        lambda d_in: d_in,
    )


# generate then and private variants of the constructor
then_eigenvalues = to_then(make_eigenvalues)
make_private_eigenvalues = with_privacy(make_eigenvalues)
then_private_eigenvalues = to_then(make_private_eigenvalues)
