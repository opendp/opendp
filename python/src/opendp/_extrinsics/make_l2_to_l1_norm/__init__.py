from opendp.extrinsics._utilities import register_transformation
from opendp.mod import Domain, Metric


def make_l2_to_l1_norm(input_domain: Domain, input_metric: Metric):
    import opendp.prelude as dp
    dp.assert_features("contrib")
    if input_metric.type.origin != "L2Distance":
        raise ValueError("expected input_metric to be L2Distance")

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        input_domain,
        dp.l1_distance(T=input_metric.distance_type),
        lambda arg: arg,
        lambda d_in: d_in,
    )

# generate then variant of the constructor
then_l2_to_l1_norm = register_transformation(make_l2_to_l1_norm)
