import math
from numbers import Integral, Real

from opendp.mod import AtomDomain, Measure, Domain, Metric, Measurement, VectorDomain
import opendp.prelude as dp
from opendp._lib import import_optional_dependency
from opendp._internal import _make_transformation, _new_pure_function
from opendp.extras.numpy._make_np_sum import make_private_np_sum
from opendp.extras.numpy import NPArray2Domain, make_np_clamp
from opendp.extras.sklearn._domains import SklearnFitDescriptor
from opendp.typing import _ELEMENTARY_TYPES, _NUMERIC_TYPES


def _normalize_classes(classes):
    """Sort and snapshot two distinct, representable public class labels."""
    np = import_optional_dependency("numpy")
    labels = np.asarray(classes)
    # Compare Python scalars, not NumPy's potentially lossy mixed-type equality.
    original = [
        value.item() if isinstance(value, np.generic) else value
        for value in np.asarray(classes, dtype=object).flat
    ]
    if (
        labels.shape != (2,)
        or labels.dtype.type not in _ELEMENTARY_TYPES
        or labels.dtype.kind not in "biufU"
        or original != labels.tolist()
        or (
            labels.dtype.kind == "f"
            and any(
                not value.is_integer() or not -(2**63) <= value < 2**63
                for value in labels.tolist()
            )
        )
        or labels[0] == labels[1]
    ):
        raise ValueError("classes must contain two distinct public discrete labels")
    return np.sort(labels)


def per_example_gradients(
    data, theta, fit_intercept=True, loss="log_loss", *, classes=(0, 1)
):
    """Compute finite gradients independently for each aligned SklearnFit record."""
    np = import_optional_dependency("numpy")
    # Compare labels without a data-dependent common dtype or lossy integer cast.
    y = np.asarray(
        [value.item() if isinstance(value, np.generic) else value for value in data.y],
        dtype=object,
    )
    negative, positive = np.asarray(classes).tolist()
    is_positive = y == positive
    known = (y == negative) | is_positive
    X = np.nan_to_num(data.X, nan=0.0, posinf=0.0, neginf=0.0)
    d = X.shape[1]
    gradients = np.empty((X.shape[0], d + (1 if fit_intercept else 0)))
    with np.errstate(all="ignore"):
        z = np.nan_to_num(
            X @ theta[:d] + (theta[-1] if fit_intercept else 0.0), nan=0.0
        )
        if loss == "hinge":
            signed_y = np.where(is_positive, 1.0, -1.0)
            residual = np.where(signed_y * z < 1.0, -signed_y, 0.0)
        else:
            residual = (
                import_optional_dependency("scipy.special").expit(z) - is_positive
            )
        np.multiply(residual[:, None], X, out=gradients[:, :d])
        if fit_intercept:
            gradients[:, -1] = residual
    # Unknown labels contribute nothing, including to the intercept.
    gradients[~known] = 0.0
    # Saturate overflow and remove NaNs before entering the clipping domain.
    return np.nan_to_num(gradients, copy=False)


def sgd_classifier_train(
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
    fit_intercept,
    loss,
    classes,
):
    dp.assert_features("contrib", "idealized-numerics")

    np = import_optional_dependency("numpy")
    num_coordinates = d + (1 if fit_intercept else 0)
    theta = np.zeros(num_coordinates)
    for d_mid in d_mids:
        gradient = _make_transformation(
            input_domain,
            input_metric,
            dp.numpy.array2_domain(
                num_columns=num_coordinates, size=n, nan=False, T=float
            ),
            dp.symmetric_distance(),
            lambda data, th=theta: per_example_gradients(
                data, th, fit_intercept, loss=loss, classes=classes
            ),
            # One aligned (X[i], y[i]) record produces one gradient row.
            lambda b_in: b_in,
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
        step_size = learning_rate / normalizer
        smooth_grad = noised_sum.copy()
        smooth_grad[:d] += l2_coef * theta[:d]
        next_theta = theta - step_size * smooth_grad
        if l1_coef > 0:
            # Proximal L1 update; the intercept remains unregularized.
            threshold = step_size * l1_coef
            next_theta[:d] = np.sign(next_theta[:d]) * np.maximum(
                np.abs(next_theta[:d]) - threshold, 0.0
            )
        theta = next_theta
    return theta


def make_private_sgd_classifier(
    input_domain: "Domain",
    input_metric: "Metric",
    output_measure: "Measure",
    d_in,
    d_out,
    *,
    max_iter: int,
    eta0: float,
    clip_norm: float,
    loss: str = "hinge",
    penalty: str | None = "l2",
    alpha: float = 0.0001,
    l1_ratio: float = 0.15,
    fit_intercept: bool = True,
    reference_size: int | None = None,
    classes=(0, 1),
) -> Measurement:
    """Release binary linear-classifier weights via full-batch noisy gradients.

    Supports hinge and logistic loss with fixed iterations and public scaling.
    Accepts SklearnFit with numeric features and numeric, boolean or string targets.
    Classes are public; unknown labels contribute zero gradients.
    """
    import opendp.prelude as dp

    dp.assert_features("contrib", "idealized-numerics")
    classes = _normalize_classes(classes)

    fit_desc = input_domain.cast(SklearnFitDescriptor)
    if fit_desc.y is None:
        raise ValueError("SGDClassifier requires a target domain y")
    x_desc = fit_desc.X.cast(NPArray2Domain)
    if not isinstance(fit_desc.y, VectorDomain):
        raise ValueError("y_domain must be a VectorDomain")
    y_atom = fit_desc.y.element_domain
    if not isinstance(y_atom, AtomDomain):
        raise ValueError("y must have numeric, boolean or string elements")
    y_type, y_size = y_atom.carrier_type, fit_desc.y.size
    target_types = _NUMERIC_TYPES | {"bool", "String"}
    if x_desc.T not in _NUMERIC_TYPES or y_type not in target_types:
        raise ValueError("X must have numeric element types and y must be atomic")
    if (y_type == "String") != (classes.dtype.kind == "U"):
        raise ValueError("classes must match the target domain's label type")
    if x_desc.size is not None and y_size is not None and x_desc.size != y_size:
        raise ValueError("X and y domains must have matching row counts")
    # Alignment makes the record count public if either field has a fixed size.
    n = x_desc.size if x_desc.size is not None else y_size
    d = x_desc.num_columns
    if d is None or d < 1:
        raise ValueError("X_domain must specify a positive num_columns")
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
    if l1_ratio is None and penalty != "elasticnet":
        pass  # Ignored unless using elasticnet, as in sklearn.
    elif (
        not isinstance(l1_ratio, Real)
        or not math.isfinite(l1_ratio)
        or not 0 <= l1_ratio <= 1
    ):
        raise ValueError("l1_ratio must be finite and in [0, 1]")

    if loss not in ("hinge", "log_loss"):
        raise ValueError('loss must be "hinge" or "log_loss"')
    if penalty not in ("l2", "l1", "elasticnet", None):
        raise ValueError("penalty must be 'l2', 'l1', 'elasticnet', or None")

    # The training loop sums gradients; sklearn alpha penalizes their mean.
    # This normalization uses public metadata, not the realized row count.
    alpha_sum = alpha * normalizer if penalty is not None else 0.0
    if not math.isfinite(alpha_sum):
        raise ValueError("alpha times reference_size must be finite")
    if penalty == "l2":
        l1_coef, l2_coef = 0.0, alpha_sum
    elif penalty == "l1":
        l1_coef, l2_coef = alpha_sum, 0.0
    elif penalty == "elasticnet":
        l1_coef, l2_coef = l1_ratio * alpha_sum, (1.0 - l1_ratio) * alpha_sum
    else:
        l1_coef, l2_coef = 0.0, 0.0

    if (
        isinstance(max_iter, bool)
        or not isinstance(max_iter, Integral)
        or max_iter <= 0
    ):
        raise ValueError("max_iter must be a positive integer")
    for name, value in (("eta0", eta0), ("clip_norm", clip_norm)):
        try:
            valid = (
                not isinstance(value, bool)
                and isinstance(value, Real)
                and float(value) > 0
                and math.isfinite(value)
            )
        except OverflowError:
            valid = False
        if not valid:
            raise ValueError(f"{name} must be a finite positive real number")
    eta0, clip_norm = float(eta0), float(clip_norm)
    if n is not None and d_in % 2 != 0:
        raise ValueError(
            f"For sized data, d_in must be even: one change is a substitution "
            f"affecting 2 rows. Got d_in={d_in}."
        )  # pragma: no cover

    rho_per_step = dp.binary_search_param(
        lambda r: dp.c.make_adaptive_composition(
            input_domain, input_metric, output_measure, d_in, d_mids=[r] * max_iter
        ),
        d_in=d_in,
        d_out=d_out,
        T=float,
    )  # figure out budget per step

    d_mids = [rho_per_step] * max_iter

    return dp.c.make_adaptive_composition(
        input_domain, input_metric, output_measure, d_in, d_mids=d_mids
    ) >> _new_pure_function(
        lambda queryable: sgd_classifier_train(
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
            learning_rate=eta0,
            l1_coef=l1_coef,
            l2_coef=l2_coef,
            fit_intercept=fit_intercept,
            loss=loss,
            classes=classes,
        )
    )
