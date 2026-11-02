import math
from numbers import Integral

from opendp.mod import Measure, Domain, Metric, Measurement
import opendp.prelude as dp
from opendp._lib import import_optional_dependency
from opendp._internal import _new_pure_function
from opendp.extras.numpy._make_np_sum import make_private_np_sum
from opendp.extras.numpy import NPArray1Domain, NPArray2Domain, make_np_clamp
from opendp.extras.sklearn._domains import SklearnFitDomain
from opendp.typing import _NUMERIC_TYPES


def per_example_gradients(data, theta):
    """Compute finite gradients independently for each aligned SklearnFit record."""
    np = import_optional_dependency("numpy")
    special = import_optional_dependency("scipy.special")
    y = data.y
    X = np.nan_to_num(data.X, nan=0.0, posinf=0.0, neginf=0.0)
    X = np.hstack([X, np.ones((X.shape[0], 1))])
    with np.errstate(all="ignore"):
        z = np.nan_to_num(X @ theta, nan=0.0)
        gradients = (special.expit(z) - y)[:, None] * X
    # Missing/nonfinite targets contribute nothing, including to the intercept.
    gradients[~np.isfinite(y)] = 0.0
    # Saturate overflow and remove NaNs before entering the clipping domain.
    return np.nan_to_num(gradients, copy=False)


def log_regression_train(
    queryable,
    *,
    input_domain,
    input_metric,
    output_measure,
    d_in,
    d_mids,
    n,
    d,
    normalizer,
    clip_norm,
    learning_rate,
    l1_coef,
    l2_coef,
):
    dp.assert_features("contrib", "honest-but-curious", "idealized-numerics")

    np = import_optional_dependency("numpy")
    theta = np.zeros(d + 1)
    for d_mid in d_mids:
        gradient = dp.t.make_user_transformation(
            input_domain,
            input_metric,
            dp.numpy.array2_domain(num_columns=d + 1, size=n, nan=False, T=float),
            dp.symmetric_distance(),
            function=lambda data, th=theta: per_example_gradients(data, th),
            # One aligned (X[i], y[i]) record produces one gradient row.
            stability_map=lambda b_in: b_in,
        )
        clamp = make_np_clamp(
            gradient.output_domain, gradient.output_metric, clip_norm, 2
        )
        clamp_domain, clamp_metric = clamp.output_space
        step = dp.binary_search_chain(
            lambda s: (
                gradient
                >> clamp
                >> make_private_np_sum(clamp_domain, clamp_metric, output_measure, s)
            ),
            d_in=d_in,
            d_out=d_mid,
            T=float,
            bounds=(0.0, 1e6),
        )
        noised_sum = np.asarray(queryable(step))
        penalty_grad = l1_coef * np.sign(theta) + l2_coef * theta
        penalty_grad[-1] = 0.0
        theta = theta - learning_rate / normalizer * (noised_sum + penalty_grad)
    return theta


def make_private_logistic_regression(
    input_domain: "Domain",
    input_metric: "Metric",
    output_measure: "Measure",
    d_in,
    d_out,
    *,
    n_iters: int,
    learning_rate: float,
    clip_norm: float,
    alpha: float = 1.0,
    l1_ratio: float = 0.0,
    reference_size: int | None = None,
) -> Measurement:
    """Release binary logistic-regression weights using full-batch noisy gradient descent.

    Each iteration computes per-example gradients over the entire dataset, clips
    their norms, privately sums them with Gaussian noise, and updates the weights.
    Regularization is postprocessing, with parameter alpha and L1 fraction
    l1_ratio; the intercept is not regularized. Each update scales the noisy
    gradient sum plus the regularization gradient by learning_rate / normalizer.
    The normalizer is the public reference_size if provided, otherwise the domain's
    known size. It must be a finite positive integer; unsized domains require an
    explicit reference_size. It need not equal the private dataset size and changes
    only the step size, not the regularized objective. Privacy loss is composed
    under zCDP, independently of reference_size and the actual number of records.

    The input Domain must be described by SklearnFitDomain with numeric
    NPArray2Domain features and numeric or boolean NPArray1Domain targets.
    Ordinary NumPy SklearnFit inputs can use domain_of(data, infer=True), which
    assumes public dtypes and feature count but infers neither size nor finiteness.
    Binary labels are expected to be encoded as 0 and 1; the target domain need
    not restrict their values. Nonfinite features are replaced with zero, and
    nonfinite targets produce zero gradient rows. Extreme arithmetic is sanitized
    to finite values before clipping. All preprocessing is row-local, preserving
    identity stability under symmetric_distance(). Feature-domain constraints are
    retained. Fixed-size domains use substitution adjacency (even d_in); unsized
    domains use add/remove adjacency, including d_in=1.
    """
    import opendp.prelude as dp

    dp.assert_features("contrib", "idealized-numerics")

    fit_desc = input_domain.cast(SklearnFitDomain)
    if fit_desc.y is None:
        raise ValueError("LogisticRegression requires a target domain y")
    x_desc = fit_desc.X.cast(NPArray2Domain)
    y_desc = fit_desc.y.cast(NPArray1Domain)
    target_types = _NUMERIC_TYPES | {"bool"}
    if x_desc.T not in _NUMERIC_TYPES or y_desc.T not in target_types:
        raise ValueError("X and y must have numeric element types (y may be boolean)")
    if (
        x_desc.size is not None
        and y_desc.size is not None
        and x_desc.size != y_desc.size
    ):
        raise ValueError("X and y domains must have matching row counts")
    # Alignment makes the record count public if either field has a fixed size.
    n = x_desc.size if x_desc.size is not None else y_desc.size
    d = x_desc.num_columns
    if d is None:
        raise ValueError("X_domain must specify num_columns")
    normalizer = reference_size if reference_size is not None else n
    if normalizer is None:
        raise ValueError("reference_size is required for unsized domains")
    if (
        isinstance(normalizer, bool)
        or not isinstance(normalizer, Integral)
        or normalizer <= 0
    ):
        raise ValueError("reference_size must be a finite positive integer")
    try:
        finite = math.isfinite(normalizer)
    except OverflowError:
        finite = False
    if not finite:
        raise ValueError("reference_size must be a finite positive integer")

    if input_metric != dp.symmetric_distance():  # |\label{metric-check}|
        raise ValueError(
            "input_metric must be the symmetric distance"
        )  # pragma: no cover

    if output_measure != dp.zero_concentrated_divergence():
        raise ValueError(
            "output_measure must be zero-concentrated divergence (zCDP)"
        )  # pragma: no cover

    if not math.isfinite(alpha) or alpha < 0:
        raise ValueError("alpha must be finite and nonnegative")
    if not math.isfinite(l1_ratio) or not 0 <= l1_ratio <= 1:
        raise ValueError("l1_ratio must be finite and in [0, 1]")

    l1_coef, l2_coef = l1_ratio * alpha, (1.0 - l1_ratio) * alpha

    if n_iters < 1:
        raise ValueError(f"n_iters must be >= 1, got {n_iters}")  # pragma: no cover
    if learning_rate <= 0:
        raise ValueError(
            f"learning_rate must be > 0, got {learning_rate}"
        )  # pragma: no cover
    if clip_norm <= 0:
        raise ValueError(f"clip_norm must be > 0, got {clip_norm}")  # pragma: no cover
    if n is not None and d_in % 2 != 0:
        raise ValueError(
            f"For sized data, d_in must be even: one change is a substitution "
            f"affecting 2 rows. Got d_in={d_in}."
        )  # pragma: no cover

    rho_per_step = dp.binary_search_param(
        lambda r: dp.c.make_adaptive_composition(
            input_domain, input_metric, output_measure, d_in, d_mids=[r] * n_iters
        ),
        d_in=d_in,
        d_out=d_out,
        T=float,
    )  # figure out budget per step

    d_mids = [rho_per_step] * n_iters

    return dp.c.make_adaptive_composition(
        input_domain, input_metric, output_measure, d_in, d_mids=d_mids
    ) >> _new_pure_function(
        lambda queryable: log_regression_train(
            queryable,
            input_domain=input_domain,
            input_metric=input_metric,
            output_measure=output_measure,
            d_in=d_in,
            d_mids=d_mids,
            n=n,
            d=d,
            normalizer=normalizer,
            clip_norm=clip_norm,
            learning_rate=learning_rate,
            l1_coef=l1_coef,
            l2_coef=l2_coef,
        )
    )
