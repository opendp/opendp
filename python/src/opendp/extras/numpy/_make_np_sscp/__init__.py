from opendp.extras._utilities import to_then
from opendp.extras.numpy import _nuclear_norm_distance, _symmetric_matrix_domain
from opendp.mod import Domain, Metric, Transformation
from opendp._lib import import_optional_dependency
from opendp._internal import _make_transformation

# planning to make this public, but may make more API changes


def make_np_sscp(input_domain: Domain, input_metric: Metric) -> Transformation:
    """Construct a Transformation that computes the sum of squares and cross-products, ``X.T @ X``.

    The data is not centered or normalized. A declared norm bound must be centered
    at zero. To use a nonzero public origin, subtract that origin from the data
    and update the input domain to reflect the centered data first.

    :param input_domain: instance of `array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`

    :return: a Transformation with nuclear-norm output metric, computing ``X.T @ X``
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPArray2Domain"):
        raise ValueError(
            f"input_domain ({input_domain}) must be a 2d-numpy array domain"
        )  # pragma: no cover

    if input_metric != dp.symmetric_distance():
        raise ValueError(
            f"input_metric ({input_metric}) must be symmetric distance"
        )  # pragma: no cover

    input_desc = input_domain.descriptor

    if input_desc.nan:
        raise ValueError(
            f"input_domain ({input_domain}) must not contain NaN values"
        )  # pragma: no cover

    if input_desc.num_columns is None:
        raise ValueError(
            f"input_domain ({input_domain}) must have known num_columns"
        )  # pragma: no cover

    if input_desc.norm is None or not np.isfinite(input_desc.norm):
        raise ValueError("input_domain must have a finite row norm bound")
    row_norm_sq = input_desc.norm**2
    if row_norm_sq == 0 and input_desc.norm != 0:
        raise ValueError("row norm squared underflows")
    if not np.isfinite(row_norm_sq):
        raise ValueError("row norm squared overflows")

    if input_desc.origin is not None and np.any(input_desc.origin != 0):
        raise ValueError(
            "input_domain origin must be zero. Center the data and update its domain first."
        )

    return _make_transformation(
        input_domain,
        input_metric,
        _symmetric_matrix_domain(num_features=input_desc.num_columns, T=input_desc.T),
        _nuclear_norm_distance(),
        lambda arg: arg.T @ arg,
        # ||xx^T||_* = ||x||_2^2 <= norm^2, also for L1-bounded rows.
        lambda d_in: d_in * row_norm_sq,
    )


# generate then variant of the constructor
then_np_sscp = to_then(make_np_sscp)
