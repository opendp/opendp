"""Floating-point score evaluation and sensitivity for MBI selection."""

from fractions import Fraction
from math import isfinite, prod, ulp
from typing import Sequence

from opendp._lib import import_optional_dependency
from opendp.extras._numerics import _pairwise_sum, _pairwise_sum_depth, _round_up
from opendp.metrics import linf_distance
from opendp.domains import atom_domain, vector_domain
from opendp._internal import _float_sum_sensitivity, _l01inf_to_lp, _make_transformation
from opendp.extras.mbi._utilities import Count, TypedDictDomain, TypedDictDistance
from opendp.mod import ExtrinsicDomain, Metric, Transformation


def _make_l1_score(domain, synthetic, *, penalty=0.0, weight=1.0):
    """Return a vectorized score and its conservative stability map."""
    np = import_optional_dependency("numpy")
    if str(domain.T) != "i32":
        raise ValueError("MBI selection scores require i32 marginal domains")

    synthetic = np.array(synthetic, dtype=np.float64, copy=True)
    if synthetic.shape != domain.shape:
        raise ValueError("model projection shape differs from marginal domain")
    synthetic = synthetic.ravel()
    if not np.isfinite(synthetic).all():
        raise ValueError("model projection must be finite")
    penalty, weight = float(penalty), float(weight)
    if not isfinite(penalty) or not isfinite(weight) or weight <= 0:
        raise ValueError(
            "selection penalty and weight must be finite and weight positive"
        )

    # Depth includes residual subtraction and penalty subtraction.
    # Absolute value only changes the signs of the two residual summands.
    m = synthetic.size
    depth = _pairwise_sum_depth(m) + 2
    magnitude = _round_up(
        m * (2**31 + Fraction(float(np.max(np.abs(synthetic)))))
        + abs(Fraction(penalty))
    )
    # Also verify that every intermediate and the final score is finite.
    computed_bound = _float_sum_sensitivity(
        magnitude, depth, magnitude, magnitude, T="f64"
    )
    w = Fraction(weight)
    weighted_bound = _round_up(Fraction(computed_bound) * w)
    # Multiplication is not an addition in the tree. Two final
    # roundings can differ by at most one ULP of the public bound.
    rounding = Fraction(ulp(weighted_bound)) if weight != 1.0 else Fraction(0)

    def score(exact):
        values = np.abs(exact.ravel().astype(np.float64) - synthetic)
        return (float(_pairwise_sum(values)) - penalty) * weight

    def stability(distance):
        sensitivity = _l01inf_to_lp(distance, p=1)
        if sensitivity == 0:
            return 0.0
        bound = _float_sum_sensitivity(
            sensitivity, depth, magnitude, magnitude, T="f64"
        )
        return _round_up(Fraction(bound) * w + rounding)

    return score, stability


def make_l1_scores(
    input_domain: ExtrinsicDomain,
    input_metric: Metric,
    queries: Sequence[Count],
    model,
    expectations: dict[tuple[str, ...], float] | None = None,
) -> Transformation:
    """Score marginal discrepancies, with optional AIM penalties and weights."""
    from opendp.extras.numpy import NPArrayDDomain

    domains = input_domain.cast(TypedDictDomain)
    input_metric.cast(TypedDictDistance)
    scores = []
    for query in queries:
        by = tuple(query.by)
        desc = domains[by].cast(NPArrayDDomain)
        penalty = 0.0 if expectations is None else expectations[by] * prod(desc.shape)
        score, stability = _make_l1_score(
            desc, model.project(by).values, penalty=penalty, weight=query.weight
        )
        scores.append((by, score, stability))

    return _make_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=vector_domain(atom_domain(T="f64", nan=False)),
        output_metric=linf_distance(T="f64", monotonic=False),
        function=lambda exact: [score(exact[by]) for by, score, _ in scores],
        stability_map=lambda d: max(stability(d[by]) for by, _, stability in scores),
    )
