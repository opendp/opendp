from __future__ import annotations
from opendp._extrinsics.make_np_clamp import make_np_clamp
from opendp._extrinsics._utilities import to_then
from opendp._extrinsics._make_np_sum import make_private_np_sum
from opendp.mod import Domain, Metric, Measurement

# planning to make this public, but may make more API changes


def make_private_np_mean(
    input_domain: Domain,
    input_metric: Metric,
    scale: float,
    norm: float | None = None,
    p: int | None = 2,
    origin=None,
) -> Measurement:
    """Construct a Measurement that releases the mean of a 2-dimensional array.

    :param input_domain: instance of `np_array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param scale: parameter for `make_laplace` or `make_gaussian`, depending on `p`
    :param norm: clamp each row to this norm. Required if data is not already bounded
    :param p: designates L`p` norm
    :param origin: norm clamping is centered on this point. Defaults to zero

    :returns a Measurement that computes the DP mean
    """
    import opendp.prelude as dp
    import numpy as np  # type: ignore[import]

    dp.assert_features("contrib", "floating-point")

    if norm is not None:
        t_clamp = make_np_clamp(input_domain, input_metric, norm, p, origin)
        input_domain, input_metric = t_clamp.output_space

    input_desc = input_domain.descriptor
    size = input_desc.size
    if size is None:
        raise ValueError("input_domain must consist of sized data")

    privacy_measure = {
        1: dp.max_divergence(T=input_desc.T),
        2: dp.zero_concentrated_divergence(T=input_desc.T),
    }[input_desc.p]

    t_sum = make_private_np_sum(
        input_domain, input_metric, privacy_measure, scale * size
    )
    if norm is not None:
        t_sum = t_clamp >> t_sum

    return t_sum >> (lambda sums: np.array(sums) / size)


# generate then variant of the constructor
then_private_np_mean = to_then(make_private_np_mean)
