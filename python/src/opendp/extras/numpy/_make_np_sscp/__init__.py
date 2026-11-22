from opendp.extras._utilities import to_then
from opendp.extras.numpy import _nuclear_norm_distance, _symmetric_matrix_domain
from opendp.mod import Domain, Metric, Transformation
from opendp._lib import import_optional_dependency
from opendp._internal import _make_transformation

# planning to make this public, but may make more API changes


def make_np_sscp(
    input_domain: Domain, input_metric: Metric, output_metric: Metric | None = None
) -> Transformation:
    """Construct a Transformation that computes the sum of squares and cross-products, ``X.T @ X``.

    The data is not centered or normalized. A declared norm bound must be centered
    at zero. To use a nonzero public origin, subtract that origin from the data
    and update the input domain to reflect the centered data first.

    :param input_domain: instance of `array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param output_metric: entrywise L1 if input p=1, Frobenius/L2 if p=2
        (default), or explicitly the nuclear-norm metric. Both L1 and L2
        metrics can be requested independently of the input norm.

    :return: a Transformation computing ``X.T @ X``
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
    l1_metric = dp.l1_distance(T=input_desc.T)
    l2_metric = dp.l2_distance(T=input_desc.T)
    nuclear_metric = _nuclear_norm_distance()
    if output_metric is None:
        output_metric = {1: l1_metric, 2: l2_metric}.get(input_desc.p)
    if output_metric not in (l1_metric, l2_metric, nuclear_metric):
        raise ValueError("output_metric must be L1, L2, or nuclear norm")

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

    # ||xx^T||_1 = ||x||_1^2 and ||xx^T||_F = ||x||_2^2.
    # Nuclear norm of a rank-one PSD matrix is also ||x||_2^2.
    # L2-bounded rows have ||x||_1^2 <= d * ||x||_2^2.
    row_bound = (
        row_norm_sq * input_desc.num_columns
        if output_metric == l1_metric and input_desc.p == 2
        else row_norm_sq
    )
    if not np.isfinite(row_bound):
        raise ValueError("SSCP sensitivity overflows")
    if input_desc.size is not None and input_desc.num_columns == 1:
        # A 1x1 Gram matrix is a scalar: |x^2 - y^2| <= R^2
        # for fixed-size row replacement, under all three output metrics.
        stability = lambda d_in: d_in // 2 * row_norm_sq
    elif output_metric == l2_metric and input_desc.size is not None:
        # Under fixed-size substitution, two symmetric edits replace a row;
        # ||xx^T - yy^T||_F <= sqrt(2) R^2.
        replacement_bound = np.nextafter(np.sqrt(2.0) * row_norm_sq, np.inf)
        if not np.isfinite(replacement_bound):
            raise ValueError("Frobenius replacement sensitivity overflows")
        stability = lambda d_in: d_in // 2 * replacement_bound
    else:
        stability = lambda d_in: d_in * row_bound

    return _make_transformation(
        input_domain,
        input_metric,
        _symmetric_matrix_domain(num_features=input_desc.num_columns, T=input_desc.T),
        output_metric,
        lambda arg: arg.T @ arg,
        stability,
    )


# generate then variant of the constructor
then_np_sscp = to_then(make_np_sscp)
