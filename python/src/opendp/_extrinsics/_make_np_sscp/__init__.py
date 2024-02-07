from opendp._extrinsics._utilities import to_then
from opendp._extrinsics.domains import _np_sscp_domain
from opendp.mod import Domain, Metric, Transformation

# planning to make this public, but may make more API changes


def make_np_sscp(
    input_domain: Domain, input_metric: Metric, output_metric: Metric
) -> Transformation:
    """Construct a Transformation that computes a covariance matrix from the input data.

    :param input_domain: instance of `np_array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param output_metric: either `symmetric_distance()` or `l2_distance()`

    :returns a Measurement that computes the DP sum
    """
    import opendp.prelude as dp
    import numpy as np  # type: ignore[import]

    dp.assert_features("contrib", "floating-point")

    if not str(input_domain).startswith("NPArray2Domain"):
        raise ValueError("input_domain must be a 2d-numpy array domain")

    input_desc = input_domain.descriptor
    if input_desc.num_columns is None:
        raise ValueError("num_columns must be known in input_domain")

    if output_metric.type == "SymmetricDistance":
        stability = lambda d_in: d_in
    elif output_metric.type.origin == "L2Distance":
        norm = input_desc.norm
        if input_desc.p != 2:
            raise ValueError("rows in input_domain must have bounded L2 norm")

        if input_desc.size is None:
            origin = np.atleast_1d(input_desc.origin)
            norm += np.linalg.norm(origin, ord=2)
            stability = lambda d_in: d_in * norm**2
        else:
            stability = lambda d_in: d_in // 2 * 2 * norm**2
    else:
        raise ValueError(
            "expected an output metric of either type SymmetricDistance or L2Distance<_>"
        )

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        _np_sscp_domain(
            num_features=input_desc.num_columns,
            norm=input_desc.norm,
            p=input_desc.p,
            size=input_desc.size,
            T=input_desc.T,
        ),
        output_metric,
        lambda arg: arg.T @ arg,
        stability,
    )


# generate then variant of the constructor
then_np_sscp = to_then(make_np_sscp)
