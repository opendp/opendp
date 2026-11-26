from opendp.extras._utilities import to_then
from opendp.mod import Domain, Measure, Measurement, Metric, Transformation
from opendp._lib import import_optional_dependency
from opendp._internal import _make_transformation, _new_pure_function

# planning to make this public, but may make more API changes


def make_eigenvalues(
    input_domain: Domain, input_metric: Metric, p: int = 1
) -> Transformation:
    """Construct a Transformation that computes the eigenvalues of a covariance matrix.

    :param input_domain: instance of `_symmetric_matrix_domain(num_features=_)`
    :param input_metric: joint nuclear/Frobenius distance on symmetric matrices
    :param p: L1 or L2 distance on the eigenvalues (1 or 2).
    """
    np = import_optional_dependency("numpy")
    import opendp.prelude as dp

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")

    if input_metric != dp.schatten12_distance():
        raise ValueError("input metric must be joint nuclear/Frobenius distance")

    input_desc = input_domain.descriptor
    if p == 1:
        output_metric = dp.l1_distance(T=input_desc.T)
        # Lidskii-Mirsky and Hoffman-Wielandt plus L2-to-L1 conversion.
        stability = lambda d_in: min(d_in[0], np.sqrt(input_desc.num_features) * d_in[1])
    elif p == 2:
        output_metric = dp.l2_distance(T=input_desc.T)
        # Hoffman-Wielandt and the nuclear norm bound on the Frobenius norm.
        stability = lambda d_in: min(d_in)
    else:
        raise ValueError("p must be 1 or 2")

    return _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=input_desc.T, nan=False)),
        output_metric,
        np.linalg.eigvalsh,
        stability,
    )


then_eigenvalues = to_then(make_eigenvalues)


def make_private_eigenvalues(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    scale: float,
) -> Measurement:
    """Release all eigenvalues with Laplace or Gaussian noise, sorted ascending."""
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    if output_measure == dp.max_divergence():
        p, noise = 1, dp.m.then_laplace
    elif output_measure == dp.zero_concentrated_divergence():
        p, noise = 2, dp.m.then_gaussian
    else:
        raise ValueError(
            "eigenvalues support max_divergence() or zero_concentrated_divergence()"
        )
    return (
        make_eigenvalues(input_domain, input_metric, p)
        >> noise(scale)
        >> dp.as_array()
        >> _new_pure_function(np.sort)
    )


then_private_eigenvalues = to_then(make_private_eigenvalues)
