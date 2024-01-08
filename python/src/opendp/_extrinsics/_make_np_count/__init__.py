import opendp.prelude as dp
from opendp.extrinsics._utilities import to_then, with_privacy

# planning to make this public, but may make more API changes

def make_np_count(input_domain, input_metric):
    """Construct a new Transformation that returns the length of axis 0 of a 2-dimensional array."""
    dp.assert_features("contrib")
    size = input_domain.size

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.atom_domain(T=int),
        dp.absolute_distance(T=int),
        (lambda x: x.shape[0]) if size is None else (lambda _: size),
        lambda d_in: d_in if size is None else 0,
    )


# generate then and private variants of the constructor
then_np_count = to_then(make_np_count)
make_private_np_count = with_privacy(make_np_count)
then_private_np_count = to_then(make_private_np_count)