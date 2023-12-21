import opendp.prelude as dp
from opendp.extrinsics.register import register_measurement


def make_eigenvalues(input_domain, input_metric):
    """Construct a new Transformation that computes the eigenvalues of a covariance matrix."""
    import numpy as np

    dp.assert_features("contrib", "floating-point")

    descriptor = input_domain.descriptor

    if input_metric != dp.symmetric_distance():
        raise ValueError("expected symmetric distance input metric")

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


then_eigenvalues = register_measurement(make_eigenvalues)
