import opendp.prelude as dp
from opendp.extrinsics._utilities import to_then, with_privacy


def make_np_sum(input_domain, input_metric):
    """Construct a new Transformation that computes a sum over the row axis of a 2-dimensional array."""
    import numpy as np

    dp.assert_features("contrib", "floating-point")
    descriptor = input_domain.descriptor

    norm = descriptor.get("norm")
    if norm is None:
        raise ValueError("input_domain must have bounds. See make_np_clamp")

    order = input_domain.descriptor["ord"]
    output_metric = {1: dp.l1_distance, 2: dp.l2_distance}[order]

    size = descriptor.get("size")
    if size is None:
        origin = np.atleast_1d(descriptor.get("origin", 0.0))
        norm += np.linalg.norm(origin, ord=order)
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


then_np_sum = to_then(make_np_sum)
make_private_np_sum = with_privacy(make_np_sum)
then_private_np_sum = to_then(make_private_np_sum)