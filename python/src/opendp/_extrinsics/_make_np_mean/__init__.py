from opendp.extrinsics.make_np_clamp import make_np_clamp
from opendp.extrinsics._utilities import to_then
from opendp.extrinsics._make_np_sum import make_private_np_sum


def make_private_np_mean(
    input_domain, input_metric, scale, norm=None, p=2, origin=None
):
    import opendp.prelude as dp
    import numpy as np # type: ignore[import]

    dp.assert_features("contrib")

    descriptor = input_domain.descriptor

    if norm is not None:
        t_clamp = make_np_clamp(input_domain, input_metric, norm, p, origin)
        input_domain, input_metric = t_clamp.output_space

    size = descriptor.get("size")
    if size is None:
        raise ValueError("input_domain must consist of sized data")

    privacy_measure = {
        1: dp.max_divergence(T=descriptor["T"]),
        2: dp.zero_concentrated_divergence(T=descriptor["T"]),
    }[input_domain.descriptor["p"]]

    t_sum = make_private_np_sum(
        input_domain, input_metric, privacy_measure, scale * size
    )
    if norm is not None:
        t_sum = t_clamp >> t_sum

    return t_sum >> dp.new_function(
        lambda sums: np.array(sums) / size, TO="ExtrinsicObject"
    )


then_private_np_mean = to_then(make_private_np_mean)
