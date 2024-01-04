import opendp.prelude as dp
from opendp.extrinsics._utilities import to_then
from opendp.extrinsics.domains import _np_xTx_domain


def make_np_xTx(input_domain, input_metric, output_metric):
    """Construct a new Transformation that computes a covariance matrix from the input data."""
    import numpy as np

    dp.assert_features("contrib", "floating-point")

    if not str(input_domain).startswith("NPArray2Domain"):
        raise ValueError("input_domain must be NPArray2Domain")

    descriptor = input_domain.descriptor
    
    if "num_columns" not in descriptor:
        raise ValueError("num_columns must be known in input_domain")
    
    Q = input_metric.distance_type
    if output_metric == dp.symmetric_distance():
        stability = lambda d_in: d_in
    elif output_metric == dp.l2_distance(T=Q):
        norm, order, size = map(descriptor.get, ("norm", "order", "size"))
        if norm is None or order != 2:
            raise ValueError("rows in input_domain must have bounded L2 norm")
        
        if size is None:
            origin = np.atleast_1d(descriptor.get("origin", 0.0))
            norm += np.linalg.norm(origin, ord=2)
            stability = lambda d_in: d_in * norm**2
        else:
            stability = lambda d_in: d_in // 2 * 2 * norm**2
    else:
        raise ValueError(f"expected an output metric of either type SymmetricDistance or L2Distance<{Q}>")

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        _np_xTx_domain(
            num_features=descriptor["num_columns"],
            norm=descriptor.get("norm"),
            ord=descriptor.get("ord"),
            size=descriptor.get("size"),
            T=descriptor["T"],
        ),
        output_metric,
        lambda arg: arg.T @ arg,
        stability,
    )


then_np_xTx = to_then(make_np_xTx)
