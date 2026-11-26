from opendp.extras._utilities import to_then
from opendp.mod import Domain, Measure, Measurement, Metric, Transformation
from opendp._lib import import_optional_dependency
from opendp._internal import _make_transformation, _new_pure_function

# planning to make this public, but may make more API changes


def make_eigenvalues(input_domain: Domain, input_metric: Metric) -> Transformation:
    """Construct a Transformation that computes the eigenvalues of a covariance matrix.

    :param input_domain: instance of `_symmetric_matrix_domain(num_features=_)`
    :param input_metric: joint nuclear/Frobenius distance on symmetric matrices
    """
    np = import_optional_dependency("numpy")
    import opendp.prelude as dp

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")

    if input_metric != dp.schatten12_distance():
        raise ValueError("input metric must be joint nuclear/Frobenius distance")

    input_desc = input_domain.descriptor

    return _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=input_desc.T, nan=False)),
        dp.l1_distance(T=input_desc.T),
        np.linalg.eigvalsh,
        # Lidskii-Mirsky and Hoffman-Wielandt plus L2-to-L1 conversion.
        lambda d_in: min(d_in[0], np.sqrt(input_desc.num_features) * d_in[1]),
    )


then_eigenvalues = to_then(make_eigenvalues)


def make_private_eigenvalues(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    scale: float,
) -> Measurement:
    """Release all eigenvalues with independent Laplace noise, sorted ascending."""
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    if output_measure != dp.max_divergence():
        raise ValueError("eigenvalues currently support max_divergence() only")
    return (
        make_eigenvalues(input_domain, input_metric)
        >> dp.m.then_laplace(scale)
        >> dp.as_array()
        >> _new_pure_function(np.sort)
    )


then_private_eigenvalues = to_then(make_private_eigenvalues)
