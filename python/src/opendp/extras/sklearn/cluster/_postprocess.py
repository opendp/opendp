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
