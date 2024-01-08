from opendp.extrinsics._utilities import to_then, with_privacy


# planning to make this public, but may make more API changes


def make_np_sum(input_domain, input_metric):
    """Construct a new Transformation that computes a sum over the row axis of a 2-dimensional array."""
    import opendp.prelude as dp
    import numpy as np # type: ignore[import]

    dp.assert_features("contrib", "floating-point")
    descriptor = input_domain.descriptor

    norm = descriptor.get("norm")
    if norm is None:
        raise ValueError("input_domain must have bounds. See make_np_clamp")

    p = input_domain.descriptor["p"]
    output_metric = {1: dp.l1_distance, 2: dp.l2_distance}[p]

    size = descriptor.get("size")
    if size is None:
        origin = np.atleast_1d(descriptor.get("origin", 0.0))
        norm += np.linalg.norm(origin, ord=p)
        stability = lambda d_in: d_in * norm
    else:
        stability = lambda d_in: d_in // 2 * 2 * norm

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=input_domain.descriptor["T"])),
        output_metric(T=input_domain.descriptor["T"]),
        lambda arg: arg.sum(axis=0),
        stability,
    )


# generate then and private variants of the constructor
then_np_sum = to_then(make_np_sum)
make_private_np_sum = with_privacy(make_np_sum)
then_private_np_sum = to_then(make_private_np_sum)