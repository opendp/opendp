from opendp.extrinsics._utilities import to_then, with_privacy
import opendp.prelude as dp


def make_np_eigenvalues(input_domain, input_metric):
    """Construct a new Transformation that computes the eigenvalues of a covariance matrix."""
    import numpy as np
    dp.assert_features("contrib", "floating-point")

    if not str(input_domain).startswith("NPCovDomain"):
        raise ValueError("input_domain must be NPCovDomain")

    if input_metric != dp.symmetric_distance():
        raise ValueError("expected symmetric distance input metric")

    descriptor = input_domain.descriptor
    if "size" not in descriptor:
        raise ValueError("expected sized data")

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=descriptor["T"])),
        dp.l2_distance(T=descriptor["T"]),
        lambda arg: np.linalg.eigvalsh(arg),
        lambda d_in: d_in * descriptor["norm"] ** 2,
    )


then_np_eigenvalues = to_then(make_np_eigenvalues)
make_private_np_eigenvalues = with_privacy(make_np_eigenvalues)
then_private_np_eigenvalues = to_then(make_private_np_eigenvalues)