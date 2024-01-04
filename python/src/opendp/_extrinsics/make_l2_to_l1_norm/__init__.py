from opendp.extrinsics._utilities import register_transformation
import opendp.prelude as dp


def make_l2_to_l1_norm(input_domain: dp.Domain, input_metric: dp.Metric):
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

then_l2_to_l1_norm = register_transformation(make_l2_to_l1_norm)
