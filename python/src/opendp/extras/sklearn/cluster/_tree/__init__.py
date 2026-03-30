from __future__ import annotations

from dataclasses import dataclass
from typing import Literal, TYPE_CHECKING
import opendp.prelude as dp
from opendp._internal import _new_pure_function, _extrinsic_distance, _make_measurement, _make_transformation
from opendp._lib import import_optional_dependency
from opendp.extras.numpy._make_np_count import make_np_count

if TYPE_CHECKING:  # pragma: no cover
    import numpy


def _np():
    return import_optional_dependency("numpy")


@dataclass
class Node:
    value: float
    var: float
    children: list["Node"] | None = None


@dataclass
class Quadtree:
    lower: "numpy.ndarray"
    upper: "numpy.ndarray"


@dataclass
class LSH:
    hyperplanes: "numpy.ndarray"


# Recursively release the two children of the current cell.
# `level_allocation` is a per-level privacy-budget schedule: after normalization,
# `level_allocation[i]` is the fraction of the remaining budget allocated to
# depth `i`. `scale` is the equivalent one-shot noise scale for the total budget
# available at the current node, and per-level noise scales are derived from
# budget shares.


def make_private_tree(
    input_domain,
    input_metric,
    output_measure,
    *,
    splitter: Quadtree | LSH,
    level_allocation,
    scale: float,
    objective: Literal["kmeans", "kmedians"] = "kmedians",
    depth: int = 0,
):
    """Release a private binary tree over a recursive splitter.

    :param input_domain: Domain of the input dataset.
    :param input_metric: Metric on the input dataset.
    :param output_measure: Privacy measure used by the tree release.
    :param splitter: Recursive binary splitter that defines the tree topology.
    :param level_allocation: Relative privacy-budget allocation across the remaining tree levels.
    :param scale: Global, measure-agnostic privacy knob for noise scale.
    :param objective: Clustering objective used by the downstream tree solver. Must be ``"kmeans"`` or ``"kmedians"``.
    :param depth: Current recursion depth.
    """
    np = _np()
    split = _make_split(input_domain, input_metric, splitter=splitter, depth=depth)
    split_domain = split.output_domain
    split_metric = split.output_metric
    level_allocation = _normalize(level_allocation)
    total_budget = _privacy_budget_for_noise_scale(output_measure, scale)
    head_allocation = level_allocation[0]
    tail_allocation = np.asarray(level_allocation[1:], dtype=float)

    local_budget = head_allocation * total_budget
    local_scale = _noise_scale_for_budget(output_measure, local_budget)
    m_child_counts = _make_private_child_counts(
        split_domain,
        split_metric,
        output_measure,
        scale=local_scale,
    )

    # At the final level, just release the two child counts and stop.
    if len(level_allocation) == 1:
        return split >> m_child_counts

    tail_budget = tail_allocation.sum() * total_budget
    tail_scale = _noise_scale_for_budget(output_measure, tail_budget)
    child_allocation = _normalize(tail_allocation)
    child_splitters = _child_splitters(splitter, depth)
    child_local_scale = _noise_scale_for_budget(
        output_measure,
        child_allocation[0] * tail_budget,
    )

    m_recurse = [
        make_private_tree(
            input_domain,
            input_metric,
            output_measure,
            splitter=child_splitter,
            level_allocation=child_allocation,
            scale=tail_scale,
            objective=objective,
            depth=depth + 1,
        )
        for child_splitter in child_splitters
    ]
    m_recursive_followup = make_parallel_composition(m_recurse)
    common_followup_budget = _budget_with_slack(m_recursive_followup.map((1, 1)))

    # When a branch stops early, spend the same remaining privacy budget on a
    # direct refined count as we would have spent on opening the subtree.
    m_leaf = make_private_leaf_node(
        input_domain,
        input_metric,
        output_measure,
        scale=_noise_scale_for_budget(output_measure, common_followup_budget),
    )

    comp = dp.c.make_adaptive_composition(
        input_domain=split_domain,
        input_metric=split_metric,
        output_measure=output_measure,
        d_in=(1, 1),
        d_mids=[_budget_with_slack(m_child_counts.map((1, 1))), common_followup_budget],
    )

    def postprocess(qbl):
        count_nodes = qbl(m_child_counts)
        recurse_flags = [
            _should_recurse(
                count=node.value,
                splitter=child_splitter,
                output_measure=output_measure,
                next_local_scale=child_local_scale,
                objective=objective,
                depth=depth,
            )
            for node, child_splitter in zip(count_nodes, child_splitters)
        ]

        # Build the actual second query after observing the current child counts.
        followups = make_parallel_composition([
            m_recurse[i] if recurse else m_leaf
            for i, recurse in enumerate(recurse_flags)
        ])
        followup_outputs = qbl(followups)

        out = []
        for count_node, recurse, followup in zip(count_nodes, recurse_flags, followup_outputs):
            if recurse:
                out.append(Node(count_node.value, count_node.var, followup))
            else:
                # Below the split criterion, spend the child budget on a second
                # estimate for the same child and fuse the two estimates.
                value, var = _inverse_variance_weight(
                    (count_node.value, count_node.var),
                    (followup.value, followup.var),
                )
                out.append(Node(value, var))
        return out

    children = comp >> _new_pure_function(postprocess, TO="ExtrinsicObject")
    return split >> children



def make_private_leaf_node(input_domain, input_metric, output_measure, *, scale: float):
    return (
        make_np_count(input_domain, input_metric)
        >> dp.m.then_noise(output_measure, scale)
        >> _new_pure_function(
            lambda value: Node(value, _noise_var(output_measure, scale)),
            TO="ExtrinsicObject",
        )
    )



def _make_private_child_counts(input_domain, input_metric, output_measure, *, scale: float):
    var = _noise_var(output_measure, scale)
    trans = _make_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=dp.vector_domain(dp.atom_domain(T=int), size=2),
        output_metric=_count_metric(output_measure),
        function=lambda children: [len(child) for child in children],
        stability_map=lambda d_in: d_in[0] * d_in[1],
    )
    return (trans >> dp.m.then_noise(output_measure, scale)) >> _new_pure_function(
        lambda counts: [Node(c, var) for c in counts],
        TO="ExtrinsicObject",
    )



def _make_split(input_domain, input_metric, *, splitter: Quadtree | LSH, depth: int):
    np = _np()
    return _make_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=dp.vector_domain(input_domain, size=2),
        output_metric=parallel_distance(input_metric),
        function=lambda data: _split(np.asarray(data, dtype=float), splitter, depth),
        stability_map=lambda d_in: (1, d_in),
    )



def _split_axis_mid(splitter: Quadtree, depth: int):
    np = _np()
    axis = depth % len(splitter.lower)
    lo = np.asarray(splitter.lower, dtype=float)
    hi = np.asarray(splitter.upper, dtype=float)
    mid = 0.5 * (lo[axis] + hi[axis])
    return axis, mid, lo, hi



def _split(data: "numpy.ndarray", splitter: Quadtree | LSH, depth: int):
    np = _np()
    if isinstance(splitter, Quadtree):
        axis, mid, _, _ = _split_axis_mid(splitter, depth)
        left_mask = data[:, axis] <= mid
        right_mask = ~left_mask
        return [data[left_mask], data[right_mask]]

    if isinstance(splitter, LSH):
        hp = np.asarray(splitter.hyperplanes, dtype=float)
        if hp.ndim != 2 or hp.shape[1] != data.shape[1]:
            raise ValueError("hyperplanes must have shape (L, d)")
        if depth >= len(hp):
            return [data, data[:0]]
        dots = data @ hp[depth]
        left_mask = dots <= 0.0
        right_mask = ~left_mask
        return [data[left_mask], data[right_mask]]

    raise TypeError("unknown splitter type")



def _child_splitters(splitter: Quadtree | LSH, depth: int):
    if isinstance(splitter, Quadtree):
        axis, mid, lo, hi = _split_axis_mid(splitter, depth)
        left_lo, left_hi = lo.copy(), hi.copy()
        right_lo, right_hi = lo.copy(), hi.copy()
        left_hi[axis] = mid
        right_lo[axis] = mid
        return [Quadtree(left_lo, left_hi), Quadtree(right_lo, right_hi)]

    if isinstance(splitter, LSH):
        return [LSH(splitter.hyperplanes), LSH(splitter.hyperplanes)]

    raise TypeError("unknown splitter type")



def parallel_distance(inner_metric):
    identifier = f"ParallelDistance({inner_metric})"
    return _extrinsic_distance(identifier, descriptor=inner_metric)



def make_parallel_composition(measurements):
    measurements = list(measurements)
    if not measurements:
        raise ValueError("measurements must be nonempty")

    m0 = measurements[0]
    common_domain = m0.input_domain
    common_metric = m0.input_metric
    output_measure = m0.output_measure

    if not all(m.input_domain == common_domain for m in measurements):
        raise ValueError("all measurements must share the same input domain")
    if not all(m.input_metric == common_metric for m in measurements):
        raise ValueError("all measurements must share the same input metric")
    if not all(m.output_measure == output_measure for m in measurements):
        raise ValueError("all measurements must share the same output measure")

    return _make_measurement(
        input_domain=dp.vector_domain(common_domain, size=len(measurements)),
        input_metric=parallel_distance(common_metric),
        output_measure=output_measure,
        function=lambda data: [m(x) for m, x in zip(measurements, data)],
        privacy_map=lambda d_in: d_in[0] * max(m.map(d_in[1]) for m in measurements),
        TO="ExtrinsicObject",
    )



def _count_metric(output_measure):
    if output_measure == dp.zero_concentrated_divergence():
        return dp.l2_distance(T=int)
    if output_measure == dp.max_divergence():
        return dp.l1_distance(T=int)
    raise ValueError("output_measure must be MaxDivergence or ZeroConcentratedDivergence")



def _noise_var(output_measure, scale: float) -> float:
    if output_measure == dp.zero_concentrated_divergence():
        return scale * scale
    if output_measure == dp.max_divergence():
        return 2.0 * scale * scale
    raise ValueError("output_measure must be MaxDivergence or ZeroConcentratedDivergence")


def _noise_scale_for_budget(output_measure, budget: float) -> float:
    np = _np()
    if budget <= 0:
        raise ValueError("budget must be positive")
    if output_measure == dp.max_divergence():
        return _scale_with_slack(1.0 / budget)
    if output_measure == dp.zero_concentrated_divergence():
        return _scale_with_slack(np.sqrt(1.0 / (2.0 * budget)))
    raise ValueError("output_measure must be MaxDivergence or ZeroConcentratedDivergence")


def _privacy_budget_for_noise_scale(output_measure, scale: float) -> float:
    if scale <= 0:
        raise ValueError("scale must be positive")
    if output_measure == dp.max_divergence():
        return 1.0 / scale
    if output_measure == dp.zero_concentrated_divergence():
        return 1.0 / (2.0 * scale * scale)
    raise ValueError("output_measure must be MaxDivergence or ZeroConcentratedDivergence")


def _budget_with_slack(budget: float, *, factor: float = 64.0) -> float:
    np = _np()
    eps = np.finfo(float).eps
    return float(np.nextafter(float(budget) * (1.0 + factor * eps), np.inf))


def _scale_with_slack(scale: float) -> float:
    np = _np()
    return float(np.nextafter(float(scale), np.inf))


def _cell_diameter(splitter: Quadtree) -> float:
    np = _np()
    side_lengths = np.asarray(splitter.upper, dtype=float) - np.asarray(splitter.lower, dtype=float)
    return float(np.linalg.norm(side_lengths))



def _leaf_penalty(count: float, splitter: Quadtree, objective: str) -> float:
    count = max(0.0, float(count))
    diam = _cell_diameter(splitter)
    if objective == "kmedians":
        return count * diam
    if objective == "kmeans":
        return count * diam * diam
    raise ValueError("objective must be 'kmeans' or 'kmedians'")



def _should_recurse(
    count: float,
    splitter: Quadtree | LSH,
    *,
    output_measure,
    next_local_scale: float,
    objective: str,
    depth: int,
) -> bool:
    np = _np()
    # For non-quadtree splitters, we don't have a clean diameter-based leaf
    # penalty, so default to opening the next level whenever budget remains.
    if not isinstance(splitter, Quadtree):
        return True

    child_splitters = _child_splitters(splitter, depth)
    current_penalty = _leaf_penalty(count, splitter, objective)
    next_penalty = 0.5 * sum(_leaf_penalty(count, child, objective) for child in child_splitters)
    benefit = current_penalty - next_penalty

    # Translate the next level's count-noise scale into the same units as the
    # solver's leaf penalty and only split when the expected benefit exceeds it.
    next_noise_std = np.sqrt(_noise_var(output_measure, next_local_scale))
    next_noise_floor = 0.5 * sum(_leaf_penalty(next_noise_std, child, objective) for child in child_splitters)
    return benefit > next_noise_floor



def _normalize(level_allocation):
    np = _np()
    level_allocation = np.asarray(level_allocation, dtype=float)
    if level_allocation.ndim != 1 or len(level_allocation) == 0:
        raise ValueError("level_allocation must be a nonempty 1D vector")
    if np.any(level_allocation < 0) or not np.any(level_allocation > 0):
        raise ValueError("level_allocation must be nonnegative with at least one positive entry")
    return level_allocation / level_allocation.sum()


def _inverse_variance_weight(
    a: tuple[float, float], b: tuple[float, float]
) -> tuple[float, float]:
    (mean_a, var_a), (mean_b, var_b) = a, b
    if var_a < 0 or var_b < 0:
        raise ValueError("variances must be nonnegative")

    if var_a == 0 and var_b == 0:
        return mean_a, 0.0
    if var_a == 0:
        return mean_a, 0.0
    if var_b == 0:
        return mean_b, 0.0

    w_a = 1.0 / var_a
    w_b = 1.0 / var_b
    return (mean_a * w_a + mean_b * w_b) / (w_a + w_b), 1.0 / (w_a + w_b)
