import opendp.prelude as dp
from opendp.extrinsics.register import register_transformation


def make_np_cov(input_domain, input_metric):
    """Construct a new Transformation that computes a covariance matrix from the input data."""

    dp.assert_features("contrib", "floating-point")
    descriptor = input_domain.descriptor

    if "num_columns" not in descriptor:
        raise ValueError("num_columns must be known in input_domain")

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.np_cov_domain(
            num_features=descriptor["num_columns"],
            norm=descriptor.get("norm"),
            ord=descriptor.get("ord"),
            size=descriptor.get("size"),
            T=descriptor["T"],
        ),
        input_metric,
        lambda arg: arg.T @ arg,
        lambda d_in: d_in,
    )


then_np_cov = register_transformation(make_np_cov)
