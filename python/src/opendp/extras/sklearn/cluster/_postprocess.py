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


def estimate_tree_inertia(
    children: list[Node],
    splitter: Quadtree,
    cluster_centers,
    *,
    objective: str = "kmeans",
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
        return 0.0

    cluster_centers = np.asarray(cluster_centers, dtype=float)
    assigned = cluster_centers[leaf_labels]
    if objective == "kmedians":
        dists = np.abs(leaf_centers - assigned).sum(axis=1)
    else:
        diff = leaf_centers - assigned
        dists = np.sum(diff * diff, axis=1)
    return float(np.dot(leaf_weights, dists))


def extract_leaf_coreset(children: list[Node], splitter: Quadtree):
    import numpy as np

    leaf_centers: list[np.ndarray] = []
    leaf_weights: list[float] = []

    def accumulate(node: Node, node_splitter: Quadtree, depth: int) -> None:
        if node.children is None:
            leaf_centers.append(_cell_center(node_splitter))
            leaf_weights.append(max(0.0, float(node.value)))
            return

        for child, child_splitter in zip(
            node.children,
            _released_child_splitters(node_splitter, node.children, depth),
        ):
            accumulate(child, child_splitter, depth + 1)

    for child, child_splitter in zip(children, _released_child_splitters(splitter, children, 0)):
        accumulate(child, child_splitter, 1)

    dim = len(np.asarray(splitter.lower, dtype=float))
    if not leaf_centers:
        return np.zeros((0, dim), dtype=float), np.zeros(0, dtype=float)
    return np.asarray(leaf_centers, dtype=float), np.asarray(leaf_weights, dtype=float)


def solve_weighted_coreset_kmeans(
    children: list[Node],
    splitter: Quadtree,
    k: int,
    *,
    lower,
    upper,
    random_state=None,
    max_iter: int = 20,
):
    import numpy as np

    if k < 0:
        raise ValueError("k must be nonnegative")
    if k == 0:
        return 0.0, []

    X_leaf, weights = extract_leaf_coreset(children, splitter)
    mask = weights > 0.0
    X_leaf = X_leaf[mask]
    weights = weights[mask]

    lower = np.asarray(lower, dtype=float)
    upper = np.asarray(upper, dtype=float)
    if len(X_leaf) == 0:
        center = _cell_center(splitter)
        return 0.0, [np.clip(center, lower, upper).copy() for _ in range(k)]

    rng = np.random.default_rng(random_state)
    centers = _weighted_kmeans_plus_plus(X_leaf, weights, k, rng)
    centers = np.clip(centers, lower, upper)

    for _ in range(max_iter):
        labels = _nearest_center_labels(X_leaf, centers)
        updated = centers.copy()
        for idx in range(k):
            cluster_mask = labels == idx
            if not np.any(cluster_mask):
                continue
            updated[idx] = np.average(
                X_leaf[cluster_mask],
                axis=0,
                weights=weights[cluster_mask],
            )
        updated = np.clip(updated, lower, upper)
        if np.allclose(updated, centers):
            centers = updated
            break
        centers = updated

    labels = _nearest_center_labels(X_leaf, centers)
    diff = X_leaf - centers[labels]
    inertia = float(np.dot(weights, np.sum(diff * diff, axis=1)))
    return inertia, [center.copy() for center in centers]


def _weighted_kmeans_plus_plus(X_leaf, weights, k: int, rng):
    import numpy as np

    centers = np.empty((k, X_leaf.shape[1]), dtype=float)
    probs = weights / weights.sum()
    centers[0] = X_leaf[rng.choice(len(X_leaf), p=probs)]

    closest_sq = np.sum((X_leaf - centers[0]) ** 2, axis=1)
    for idx in range(1, k):
        scores = weights * closest_sq
        if float(scores.sum()) <= 0.0:
            centers[idx] = centers[idx - 1]
            continue
        centers[idx] = X_leaf[rng.choice(len(X_leaf), p=scores / scores.sum())]
        dist_sq = np.sum((X_leaf - centers[idx]) ** 2, axis=1)
        closest_sq = np.minimum(closest_sq, dist_sq)
    return centers


def _nearest_center_labels(X_leaf, centers):
    import numpy as np

    diff = X_leaf[:, None, :] - centers[None, :, :]
    return np.argmin(np.sum(diff * diff, axis=2), axis=1)


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
            _released_child_splitters(node_splitter, node.children, depth),
        ):
            accumulate(child, child_splitter, depth + 1)

    for child, child_splitter in zip(children, _released_child_splitters(splitter, children, 0)):
        accumulate(child, child_splitter, 1)

    out_labels = np.asarray(labels, dtype=int)
    out_weights = np.asarray(weights, dtype=float)
    if return_leaf_centers:
        return out_labels, out_weights, np.asarray(leaf_centers, dtype=float)
    return out_labels, out_weights


def _released_child_splitters(splitter: Quadtree, children: list[Node], depth: int):
    split_axis = _infer_released_split_axis(children)
    if split_axis is None:
        return _child_splitters(splitter, depth)
    released_splitter = Quadtree(splitter.lower, splitter.upper, axis=split_axis)
    return _child_splitters(released_splitter, depth)


def _infer_released_split_axis(children: list[Node]) -> int | None:
    axes = {child.split_axis for child in children if child.split_axis is not None}
    if not axes:
        return None
    if len(axes) != 1:
        raise ValueError("inconsistent released split_axis values across siblings")
    return int(next(iter(axes)))


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
    return Node(value=value, var=var, children=children, split_axis=node.split_axis)



def _top_down(node: Node, target: float) -> Node:
    if node.children is None:
        return Node(value=target, var=node.var, split_axis=node.split_axis)

    children = node.children
    child_total = sum(child.value for child in children)
    correction = target - child_total
    corrections = _variance_weighted_corrections(children, correction)

    adjusted_children = [
        _top_down(child, child.value + delta)
        for child, delta in zip(children, corrections)
    ]
    return Node(value=target, var=node.var, children=adjusted_children, split_axis=node.split_axis)



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

    child_splitters = _released_child_splitters(splitter, children, 0)
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

    child_splitters = _released_child_splitters(splitter, node.children, depth)
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
