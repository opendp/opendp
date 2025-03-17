from opendp.extras._utilities import to_then
from opendp.extras.numpy import _sscp_domain
from opendp.mod import Domain, Metric, Transformation
from opendp._internal import _make_transformation

# planning to make this public, but may make more API changes


def make_np_sscp(input_domain: Domain, input_metric: Metric) -> Transformation:
    """Construct a Transformation that computes a covariance matrix from the input data.

    :param input_domain: instance of `array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`

    :return: a Measurement that computes the DP sum
    """
    import opendp.prelude as dp

    dp.assert_features("contrib", "floating-point")

    if not str(input_domain).startswith("NPArray2Domain"):
        raise ValueError(
            f"input_domain ({input_domain}) must be a 2d-numpy array domain"
        )  # pragma: no cover

    if input_metric != dp.symmetric_distance():
        raise ValueError(f"input_metric ({input_metric}) must be symmetric distance")  # pragma: no cover
    
    input_desc = input_domain.descriptor

    if input_desc.nan:
        raise ValueError(f"input_domain ({input_domain}) must not contain NaN values")  # pragma: no cover

    if input_desc.num_columns is None:
        raise ValueError(f"input_domain ({input_domain}) must have known num_columns")  # pragma: no cover

    return _make_transformation(
        input_domain,
        input_metric,
        _sscp_domain(
            num_features=input_desc.num_columns,
            norm=input_desc.norm,
            p=input_desc.p,
            size=input_desc.size,
            T=input_desc.T,
        ),
        input_metric,
        lambda arg: arg.T @ arg,
        lambda d_in: d_in,
    )


# generate then variant of the constructor
then_np_sscp = to_then(make_np_sscp)
