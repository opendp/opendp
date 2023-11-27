from typing import Callable
import opendp.prelude as dp
from opendp.extrinsics.register import register_transformation, register_measurement
from opendp.extrinsics.array.clamp import make_np_clamp


def make_np_count(input_domain, input_metric):
    """Construct a new Transformation that computes a count over the row axis of a 2-dimensional array."""
    dp.assert_features("contrib")
    size = input_domain.descriptor.get("size")

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.atom_domain(T=int),
        dp.absolute_distance(T=int),
        (lambda x: x.shape[0]) if size is None else (lambda _: size),
        lambda d_in: d_in if size is None else 0,
    )


register_transformation(make_np_count)


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
        norm += np.linalg.norm(descriptor.get("origin", 0.0), ord=order)
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


register_transformation(make_np_sum)


def make_np_mean(input_domain, input_metric):
    """Construct a new Transformation that computes a mean over the row axis of a 2-dimensional array."""
    dp.assert_features("contrib", "floating-point")
    descriptor = input_domain.descriptor

    size = descriptor.get("size")
    if size is None:
        raise ValueError("input_domain must consist of sized data")

    t_sum = make_np_sum(input_domain, input_metric)

    return t_sum >> dp.t.make_user_transformation(
        t_sum.output_domain,
        t_sum.output_metric,
        t_sum.output_domain,
        t_sum.output_metric,
        lambda x: [x_i / size for x_i in x],
        lambda d_in: d_in / size,
    )


then_np_mean = register_transformation(make_np_mean)


def make_private_np_mean(
    input_domain, input_metric, scale, norm=None, ord=2, origin=None
):
    dp.assert_features("contrib")

    if norm is not None:
        t_clamp = make_np_clamp(input_domain, input_metric, norm, ord, origin)
        input_domain, input_metric = t_clamp.output_space

    t_mean = make_np_mean(input_domain, input_metric)
    if norm is not None:
        t_mean = t_clamp >> t_mean

    m_constructor = {
        1: dp.m.then_laplace,
        2: dp.m.then_gaussian,
    }[input_domain.descriptor["ord"]]

    return t_mean >> m_constructor(scale)

then_private_np_mean = register_measurement(make_private_np_mean)


def with_privacy(t_constructor: Callable):
    def private_constructor(input_domain, input_metric, privacy_measure, scale):
        dp.assert_features("contrib")
        m_constructor = {
            "ZeroConcentratedDivergence": dp.m.then_gaussian,
            "MaxDivergence": dp.m.then_laplace,
        }[privacy_measure.type.origin]

        return t_constructor(input_domain, input_metric) >> m_constructor(scale)

    private_constructor.__name__ = t_constructor.__name__.replace(
        "make_", "make_private_"
    )
    private_constructor.__doc__ = t_constructor.__doc__.replace(
        "Transformation", "Measurement"
    )
    return private_constructor


make_private_np_count = with_privacy(make_np_count)
then_private_np_count = register_measurement(make_private_np_count)
make_private_np_sum = with_privacy(make_np_sum)
then_private_np_sum = register_measurement(make_private_np_sum)
# make_private_np_mean = with_privacy(make_np_mean)

