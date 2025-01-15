from opendp.extras._utilities import to_then, with_privacy
from opendp.mod import Domain, Metric, Transformation
from opendp._lib import import_optional_dependency
from opendp._internal import _make_transformation


# planning to make this public, but may make more API changes


def make_np_sum(input_domain: Domain, input_metric: Metric) -> Transformation:
    """Construct a Transformation that computes a sum over the row axis of a 2-dimensional array.

    :param input_domain: instance of `array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`

    :returns a Measurement that computes the DP sum
    """
    import opendp.prelude as dp
    np = import_optional_dependency('numpy')

    dp.assert_features("contrib", "floating-point")

    input_desc = input_domain.descriptor
    norm = input_desc.norm
    if norm is None:
        raise ValueError("input_domain must have bounds. See make_np_clamp")  # pragma: no cover

    output_metric = {1: dp.l1_distance, 2: dp.l2_distance}[input_desc.p]

    if input_desc.size is None:
        origin = np.atleast_1d(input_desc.origin)
        norm += np.linalg.norm(origin, ord=input_desc.p)
        stability = lambda d_in: d_in * norm
    else:
        stability = lambda d_in: d_in // 2 * 2 * norm

    return _make_transformation(
        input_domain,
        input_metric,
        dp.vector_domain(dp.atom_domain(T=input_desc.T)),
        output_metric(T=input_desc.T),
        lambda arg: arg.sum(axis=0),
        stability,
    )


# generate then and private variants of the constructor
then_np_sum = to_then(make_np_sum)
make_private_np_sum = with_privacy(make_np_sum)
then_private_np_sum = to_then(make_private_np_sum)
