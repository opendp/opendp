from opendp._extrinsics._utilities import to_then, with_privacy
from opendp.mod import Domain, Metric, Transformation

# planning to make this public, but may make more API changes


def make_np_eigenvalues(input_domain: Domain, input_metric: Metric) -> Transformation:
    """Construct a Transformation that computes the eigenvalues of a covariance matrix.

    :param input_domain: instance of `_np_sscp_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    """
    import numpy as np  # type: ignore[import]
    import opendp.prelude as dp

    dp.assert_features("contrib", "floating-point")

    if not str(input_domain).startswith("NPSSCPDomain"):
        raise ValueError("input_domain must be NPSSCPDomain")

    if input_metric != dp.symmetric_distance():
        raise ValueError("input metric must be symmetric distance")
    
    input_desc = input_domain.descriptor

    if input_desc.size is None:
        # implementation assumes dataset size is known
        # to loosen this limitation, when size is unknown, do: 
        # norm += np.linalg.norm(input_domain.origin, p=2)
        # this is because the addition of one row shifted by the origin 
        #     will not be offset by the removal of another row shifted by the origin
        raise ValueError("expected sized data")

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=input_desc.T)),
        dp.l1_distance(T=input_desc.T),
        np.linalg.eigvalsh,
        # http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=12
        # ‖Λ(XTX) − Λ(X'TX)‖_1 = tr(XTX) − tr(X'TX) = tr(xxT) = \sum x_i^2 = ‖x‖_2^2 ≤ norm^2
        lambda d_in: d_in * input_desc.norm**2,
    )


# generate then and private variants of the constructor
then_np_eigenvalues = to_then(make_np_eigenvalues)
make_private_np_eigenvalues = with_privacy(make_np_eigenvalues)
then_private_np_eigenvalues = to_then(make_private_np_eigenvalues)
