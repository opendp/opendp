from dataclasses import replace
from typing import Iterable
from opendp.extras.sklearn.cluster._tree import Node, Quadtree, _child_splitters, _inverse_variance_weight


# Based on the standard two-pass hierarchical consistency pattern:
# 1. bottom-up inverse-variance averaging of each internal node estimate
#    with the sum of its children,
# 2. top-down correction so each parent equals the sum of its children.
#
# This generalizes the equal-split tree algorithm by distributing the
# top-down correction in proportion to child variances. When child variances
# are equal, this reduces to the usual uniform split.


def postprocess_node(root: Node) -> Node:
    z = _bottom_up(root)
    return _top_down(z, z.value)


def postprocess_children(children: list[Node]) -> list[Node]:
    return [postprocess_node(child) for child in children]


def estimate_group_sizes(
    children: list[Node],
    splitter: Quadtree,
    cluster_centers,
    *,
    objective: str = "kmedians",
):
    import numpy as np

    leaves, leaf_weights = _assigned_leaf_summary(
        children,
        splitter,
        cluster_centers,
        objective=objective,
    )
    if len(cluster_centers) == 0:
        return np.zeros(0, dtype=float)
    return np.bincount(leaves, weights=leaf_weights, minlength=len(cluster_centers))


def estimate_silhouette_score(
    children: list[Node],
    splitter: Quadtree,
    cluster_centers,
    *,
    objective: str = "kmedians",
):
    import numpy as np

    leaf_labels, leaf_weights, leaf_centers = _assigned_leaf_summary(
        children,
        splitter,
        cluster_centers,
        objective=objective,
        return_leaf_centers=True,
    )
    if len(leaf_weights) == 0:
        return float("nan")

    if objective == "kmedians":
        pairwise = np.abs(leaf_centers[:, None, :] - leaf_centers[None, :, :]).sum(axis=2)
    else:
        diff = leaf_centers[:, None, :] - leaf_centers[None, :, :]
        pairwise = np.sqrt(np.sum(diff * diff, axis=2))

    unique_labels = np.unique(leaf_labels)
    if len(unique_labels) < 2:
        return 0.0

    silhouettes = np.zeros(len(leaf_weights), dtype=float)
    for i, label in enumerate(leaf_labels):
        same_mask = leaf_labels == label
        same_weights = leaf_weights[same_mask].copy()
        same_indices = np.flatnonzero(same_mask)
        same_weights[np.where(same_indices == i)[0]] = 0.0
        same_denom = same_weights.sum()
        a_i = (
            float(np.dot(pairwise[i, same_mask], same_weights) / same_denom)
            if same_denom > 0
            else 0.0
        )

        b_i = float("inf")
        for other in unique_labels:
            if other == label:
                continue
            other_mask = leaf_labels == other
            other_weights = leaf_weights[other_mask]
            other_denom = other_weights.sum()
            if other_denom == 0:
                continue
            candidate = float(np.dot(pairwise[i, other_mask], other_weights) / other_denom)
            if candidate < b_i:
                b_i = candidate

        if not np.isfinite(b_i):
            silhouettes[i] = 0.0
            continue
        scale = max(a_i, b_i)
        silhouettes[i] = 0.0 if scale == 0.0 else (b_i - a_i) / scale

    return float(np.average(silhouettes, weights=leaf_weights))


def _assigned_leaf_summary(
    children: list[Node],
    splitter: Quadtree,
    cluster_centers,
    *,
    objective: str,
    return_leaf_centers: bool = False,
):
    import numpy as np

    if objective not in {"kmeans", "kmedians"}:
        raise ValueError("objective must be 'kmeans' or 'kmedians'")

    cluster_centers = np.asarray(cluster_centers, dtype=float)
    if len(cluster_centers) == 0:
        if return_leaf_centers:
            return (
                np.zeros(0, dtype=int),
                np.zeros(0, dtype=float),
                np.zeros((0, 0), dtype=float),
            )
        return np.zeros(0, dtype=int), np.zeros(0, dtype=float)

    labels: list[int] = []
    weights: list[float] = []
    leaf_centers: list[np.ndarray] = []

    def accumulate(node: Node, node_splitter: Quadtree, depth: int) -> None:
        if node.children is None:
            center = _cell_center(node_splitter)
            if objective == "kmedians":
                dists = np.abs(cluster_centers - center).sum(axis=1)
            else:
                diff = cluster_centers - center
                dists = np.sum(diff * diff, axis=1)
            labels.append(int(np.argmin(dists)))
            weights.append(max(0.0, float(node.value)))
            if return_leaf_centers:
                leaf_centers.append(center)
            return

        for child, child_splitter in zip(
            node.children,
            _child_splitters(node_splitter, depth),
        ):
            accumulate(child, child_splitter, depth + 1)

    for child, child_splitter in zip(children, _child_splitters(splitter, 0)):
        accumulate(child, child_splitter, 1)

    out_labels = np.asarray(labels, dtype=int)
    out_weights = np.asarray(weights, dtype=float)
    if return_leaf_centers:
        return out_labels, out_weights, np.asarray(leaf_centers, dtype=float)
    return out_labels, out_weights


def _bottom_up(node: Node) -> Node:
    if node.children is None:
        return replace(node)

    children = [_bottom_up(child) for child in node.children]
    child_total = sum(child.value for child in children)
    child_var = sum(child.var for child in children)

    value, var = _inverse_variance_weight(
        (node.value, node.var),
        (child_total, child_var),
    )
    return Node(value=value, var=var, children=children)



def _top_down(node: Node, target: float) -> Node:
    if node.children is None:
        return Node(value=target, var=node.var)

    children = node.children
    child_total = sum(child.value for child in children)
    correction = target - child_total
    corrections = _variance_weighted_corrections(children, correction)

    adjusted_children = [
        _top_down(child, child.value + delta)
        for child, delta in zip(children, corrections)
    ]
    return Node(value=target, var=node.var, children=adjusted_children)



def _variance_weighted_corrections(children: Iterable[Node], total_correction: float):
    import numpy as np
    vars_ = np.asarray([child.var for child in children], dtype=float)
    if np.any(vars_ < 0):
        raise ValueError("node variances must be nonnegative")

    total_var = vars_.sum()
    if total_var == 0:
        return np.full(len(vars_), total_correction / len(vars_))
    return total_correction * vars_ / total_var



def solve_tree(children, splitter: Quadtree, k: int, *, objective: str = "kmedians"):
    if not isinstance(splitter, Quadtree):
        raise NotImplementedError("tree postprocessor is implemented for Quadtree only")
    if k < 0:
        raise ValueError("k must be nonnegative")

    child_splitters = _child_splitters(splitter, 0)
    tables = [
        _solve_node(node, child_splitter, depth=1, k=k, objective=objective)
        for node, child_splitter in zip(children, child_splitters)
    ]

    cost = [float("inf")] * (k + 1)
    centers = [None] * (k + 1)
    for k0 in range(k + 1):
        best = float("inf")
        best_centers = None
        for k1 in range(k0 + 1):
            c = tables[0][0][k1] + tables[1][0][k0 - k1]
            if c < best:
                best = c
                best_centers = tables[0][1][k1] + tables[1][1][k0 - k1]
        cost[k0] = best
        centers[k0] = best_centers
    return cost, centers


def _solve_node(node: Node, splitter: Quadtree, *, depth: int, k: int, objective: str):
    if node.children is None:
        return _solve_leaf(node, splitter, k=k, objective=objective)

    child_splitters = _child_splitters(splitter, depth)
    left_table = _solve_node(
        node.children[0], child_splitters[0], depth=depth + 1, k=k, objective=objective
    )
    right_table = _solve_node(
        node.children[1], child_splitters[1], depth=depth + 1, k=k, objective=objective
    )

    cost = [float("inf")] * (k + 1)
    centers = [None] * (k + 1)
    for k0 in range(k + 1):
        best = float("inf")
        best_centers = None
        for k1 in range(k0 + 1):
            c = left_table[0][k1] + right_table[0][k0 - k1]
            if c < best:
                best = c
                best_centers = left_table[1][k1] + right_table[1][k0 - k1]
        cost[k0] = best
        centers[k0] = best_centers
    return cost, centers


def _solve_leaf(node: Node, splitter: Quadtree, *, k: int, objective: str):
    center = _cell_center(splitter)
    diam = _cell_diameter(splitter)
    penalty = node.value * diam if objective == "kmedians" else node.value * diam * diam

    cost = [penalty] + [0.0] * k
    centers = [[]] + [[center.copy() for _ in range(k0)] for k0 in range(1, k + 1)]
    return cost, centers


def _cell_center(splitter: Quadtree):
    import numpy as np

    return 0.5 * (
        np.asarray(splitter.lower, dtype=float)
        + np.asarray(splitter.upper, dtype=float)
    )


def _cell_diameter(splitter: Quadtree):
    import numpy as np

    side_lengths = np.asarray(splitter.upper, dtype=float) - np.asarray(
        splitter.lower, dtype=float
    )
    return float(np.linalg.norm(side_lengths))
