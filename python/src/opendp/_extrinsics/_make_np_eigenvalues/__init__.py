from opendp.extrinsics._utilities import to_then, with_privacy

# planning to make this public, but may make more API changes


def make_np_eigenvalues(input_domain, input_metric):
    """Construct a new Transformation that computes the eigenvalues of a covariance matrix."""
    import numpy as np # type: ignore[import]
    import opendp.prelude as dp
    dp.assert_features("contrib", "floating-point")

    if not str(input_domain).startswith("NPSSCPDomain"):
        raise ValueError("input_domain must be NPSSCPDomain")

    if input_metric != dp.symmetric_distance():
        raise ValueError("expected symmetric distance input metric")

    if input_domain.size is None:
        # proof assumes dataset size is known
        raise ValueError("expected sized data")

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=input_domain.T)),
        dp.l2_distance(T=input_domain.T),
        np.linalg.eigvalsh,
        lambda d_in: d_in * input_domain.norm ** 2,
    )


# generate then and private variants of the constructor
then_np_eigenvalues = to_then(make_np_eigenvalues)
make_private_np_eigenvalues = with_privacy(make_np_eigenvalues)
then_private_np_eigenvalues = to_then(make_private_np_eigenvalues)
