from opendp.mod import Measure, Domain, Metric, Measurement
import opendp.prelude as dp
from opendp._lib import import_optional_dependency
from opendp._internal import _new_pure_function
from opendp.extras.numpy._make_np_sum import make_private_np_sum
from opendp.extras.numpy import make_np_clamp


def per_example_gradients(data, theta):
    np = import_optional_dependency("numpy")
    special = import_optional_dependency("scipy.special")
    y = data[:, -1]
    Xf = data[:, :-1]
    X = np.hstack([Xf, np.ones((data.shape[0], 1))])
    z = X @ theta
    p = special.expit(z)
    return (p - y)[:, None] * X


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
            dp.numpy.array2_domain(num_columns=d + 1, size=n, T=float),
            dp.symmetric_distance(),
            function=lambda data, th=theta: per_example_gradients(data, th),
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
        theta = theta - learning_rate * (noised_sum / n + penalty_grad)
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
    penalty: str = "l2",
    C: float = 1.0,
    l1_ratio: float | None = None ,
) -> dp.Measurement:

    import opendp.prelude as dp

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPArray2Domain"):  # |\label{domain-check}|
        raise ValueError(
            f"input_domain ({input_domain}) must be NPArray2Domain"
        )  # pragma: no cover

    if input_domain.descriptor.nan:
        raise ValueError(
            f"input_domain ({input_domain}) must not permit NaN elements"
        )  # pragma: no cover

    if input_metric != dp.symmetric_distance():  # |\label{metric-check}|
        raise ValueError(
            "input_metric must be the symmetric distance"
        )  # pragma: no cover

    if output_measure != dp.zero_concentrated_divergence():
        raise ValueError(
            "output_measure must be zero-concentrated divergence (zCDP)"
        )  # pragma: no cover

    if penalty not in ("l1", "l2", "elasticnet", None):
        raise ValueError(
            f"penalty must be 'l1', 'l2', 'elasticnet', or None, got {penalty}"
        )  # pragma: no cover

    if C <= 0:
        raise ValueError(f"C must be > 0, got {C}")  # pragma: no cover
    if penalty == "elasticnet" and l1_ratio is None:
        raise ValueError(
            "l1_ratio required when penalty='elasticnet'"
        )  # pragma: no cover
    if l1_ratio is not None and not (0 <= l1_ratio <= 1):
        raise ValueError(
            f"l1_ratio must be in [0, 1], got {l1_ratio}"
        )  # pragma: no cover

    desc = input_domain.descriptor
    n, d = desc.size, desc.num_columns - 1

    reg = 1.0 / (C * n)

    if penalty is None:
        l1_coef, l2_coef = 0.0, 0.0
    elif penalty == "l2":
        l1_coef, l2_coef = 0.0, reg
    elif penalty == "l1":
        l1_coef, l2_coef = reg, 0.0
    elif penalty == "elasticnet":
        l1_coef, l2_coef = l1_ratio * reg, (1.0 - l1_ratio) * reg

    if n is None:
        raise ValueError(
            "input_domain must have known size (sized data required)"
        )  # pragma: no cover
    if n_iters < 1:
        raise ValueError(f"n_iters must be >= 1, got {n_iters}")  # pragma: no cover
    if learning_rate <= 0:
        raise ValueError(
            f"learning_rate must be > 0, got {learning_rate}"
        )  # pragma: no cover
    if clip_norm <= 0:
        raise ValueError(f"clip_norm must be > 0, got {clip_norm}")  # pragma: no cover
    if penalty not in ("l1", "l2", "elasticnet", None):
        raise ValueError(...)  # pragma: no cover
    if d_in % 2 != 0:
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
            clip_norm=clip_norm,
            learning_rate=learning_rate,
            l1_coef=l1_coef,
            l2_coef=l2_coef,
        )
    )
