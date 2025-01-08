from opendp.mod import Domain, Metric, Transformation
import opendp.prelude as dp
from opendp.extras._utilities import to_then, with_privacy
from opendp._internal import _make_transformation

# planning to make this public, but may make more API changes


def make_np_count(input_domain: Domain, input_metric: Metric) -> Transformation:
    """Construct a Transformation that returns the length of axis 0 of a 2-dimensional array.

    :param input_domain: instance of `array2_domain(...)`
    :param input_metric: instance of `symmetric_distance()`
    """
    dp.assert_features("contrib")
    size = input_domain.descriptor.size

    if input_metric != dp.symmetric_distance():
        raise ValueError("input metric must be symmetric distance")  # pragma: no cover

    return _make_transformation(
        input_domain,
        input_metric,
        dp.atom_domain(T=int),
        dp.absolute_distance(T=int),
        lambda x: (x.shape[0] if size is None else size),
        lambda d_in: (d_in if size is None else 0),
    )


# generate then and private variants of the constructor
then_np_count = to_then(make_np_count)
make_private_np_count = with_privacy(make_np_count)
then_private_np_count = to_then(make_private_np_count)
