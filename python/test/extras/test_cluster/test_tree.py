from typing import cast

import opendp.prelude as dp
import pytest

from opendp.extras.sklearn.cluster._postprocess import (
    _assigned_leaf_summary,
    estimate_group_sizes,
    estimate_silhouette_score,
    _variance_weighted_corrections,
    postprocess_node,
    solve_tree,
)
from opendp.extras.sklearn.cluster._tree import (
    LSH,
    Node,
    Quadtree,
    _child_splitters,
    _inverse_variance_weight,
    _leaf_penalty,
    _normalize,
    _noise_var,
    _noise_scale_for_budget,
    _privacy_budget_for_noise_scale,
    _split,
    _count_metric,
    make_parallel_composition,
    make_private_tree,
)

np = pytest.importorskip("numpy")


def _node_values(nodes):
    return [node.value for node in nodes]


def test_normalize_preserves_ratios():
    level_allocation = _normalize([2, 3, 5])
    assert np.allclose(level_allocation, [0.2, 0.3, 0.5])
    assert np.isclose(level_allocation.sum(), 1.0)


@pytest.mark.parametrize("bad", [[], [0, 0], [-1, 1]])
def test_normalize_rejects_bad_level_allocation(bad):
    with pytest.raises(ValueError, match="level_allocation"):
        _normalize(bad)


def test_scale_is_global_knob_and_level_allocation_controls_budget():
    total_scale = 1e-6
    total_budget = _privacy_budget_for_noise_scale(dp.max_divergence(), total_scale)
    level_allocation = _normalize([1.0, 3.0])

    local_budget = level_allocation[0] * total_budget
    tail_budget = level_allocation[1] * total_budget

    # Larger weight means more privacy budget and therefore a smaller noise scale.
    local_scale = _noise_scale_for_budget(dp.max_divergence(), local_budget)
    tail_scale = _noise_scale_for_budget(dp.max_divergence(), tail_budget)

    assert np.isclose(local_budget + tail_budget, total_budget)
    assert tail_budget > local_budget
    assert tail_scale < local_scale


def test_inverse_variance_weight_matches_closed_form():
    mean, var = _inverse_variance_weight((10.0, 4.0), (14.0, 1.0))
    assert np.isclose(mean, 13.2)
    assert np.isclose(var, 0.8)


def test_inverse_variance_weight_handles_zero_variance_cases():
    assert _inverse_variance_weight((1.0, 0.0), (2.0, 0.0)) == (1.0, 0.0)
    assert _inverse_variance_weight((1.0, 0.0), (2.0, 4.0)) == (1.0, 0.0)
    assert _inverse_variance_weight((1.0, 4.0), (2.0, 0.0)) == (2.0, 0.0)


def test_inverse_variance_weight_rejects_negative_variance():
    with pytest.raises(ValueError, match="nonnegative"):
        _inverse_variance_weight((1.0, -1.0), (2.0, 1.0))


def test_variance_weighted_corrections_rejects_negative_variance():
    with pytest.raises(ValueError, match="nonnegative"):
        _variance_weighted_corrections([Node(0.0, -1.0)], 1.0)


def test_variance_weighted_corrections_uniform_when_total_variance_zero():
    corrections = _variance_weighted_corrections(
        [Node(1.0, 0.0), Node(2.0, 0.0), Node(3.0, 0.0)],
        6.0,
    )
    assert np.allclose(corrections, [2.0, 2.0, 2.0])


def test_estimate_group_sizes_handles_empty_centers():
    splitter = Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0]))
    sizes = estimate_group_sizes([Node(1.0, 1.0), Node(2.0, 1.0)], splitter, np.zeros((0, 2)))
    assert sizes.shape == (0,)


def test_estimate_silhouette_score_uses_l1_for_kmedians():
    splitter = Quadtree(np.array([0.0, 0.0]), np.array([5.0, 5.0]))
    children = [Node(2.0, 0.5), Node(2.0, 0.5)]
    centers = np.array([[0.0, 0.5], [4.0, 4.5]], dtype=float)
    score = estimate_silhouette_score(children, splitter, centers, objective="kmedians")
    assert np.isclose(score, 1.0)


def test_estimate_silhouette_score_handles_zero_weight_other_cluster():
    splitter = Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0]))
    children = [Node(2.0, 1.0), Node(0.0, 1.0)]
    centers = np.array([[0.25, 0.5], [0.75, 0.5]], dtype=float)
    score = estimate_silhouette_score(children, splitter, centers, objective="kmeans")
    assert score == 0.0


def test_assigned_leaf_summary_rejects_bad_objective():
    splitter = Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0]))
    with pytest.raises(ValueError, match="objective"):
        _assigned_leaf_summary([Node(1.0, 1.0)], splitter, np.array([[0.5, 0.5]]), objective="bad")


def test_assigned_leaf_summary_empty_centers_with_leaf_centers():
    splitter = Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0]))
    labels, weights, centers = _assigned_leaf_summary(
        [Node(1.0, 1.0)],
        splitter,
        np.zeros((0, 2)),
        objective="kmeans",
        return_leaf_centers=True,
    )
    assert labels.shape == (0,)
    assert weights.shape == (0,)
    assert centers.shape == (0, 0)


def test_assigned_leaf_summary_recurses_into_children():
    splitter = Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0]))
    children = [
        Node(0.0, 1.0, children=[Node(1.0, 1.0), Node(2.0, 1.0)]),
        Node(3.0, 1.0),
    ]
    labels, weights, leaf_centers = _assigned_leaf_summary(
        children,
        splitter,
        np.array([[0.25, 0.25], [0.75, 0.75]], dtype=float),
        objective="kmedians",
        return_leaf_centers=True,
    )
    assert labels.shape == (3,)
    assert np.allclose(weights, [1.0, 2.0, 3.0])
    assert leaf_centers.shape == (3, 2)


def test_postprocess_node_enforces_parent_child_consistency():
    root = Node(
        value=10.0,
        var=1.0,
        children=[Node(3.0, 1.0), Node(9.0, 1.0)],
    )
    out = postprocess_node(root)
    assert out.children is not None
    assert np.isclose(sum(child.value for child in out.children), out.value)


def test_quadtree_split_root_level():
    data = np.array([
        [0.1, 0.1],
        [0.4, 0.9],
        [0.7, 0.2],
        [0.9, 0.8],
    ])
    splitter = Quadtree(lower=np.array([0.0, 0.0]), upper=np.array([1.0, 1.0]))

    left, right = _split(data, splitter, depth=0)

    assert np.all(left[:, 0] <= 0.5)
    assert np.all(right[:, 0] > 0.5)
    assert len(left) == 2
    assert len(right) == 2



def test_quadtree_rotates_axes():
    data = np.array([
        [0.2, 0.1],
        [0.2, 0.9],
        [0.8, 0.2],
        [0.8, 0.7],
    ])
    splitter = Quadtree(lower=np.array([0.0, 0.0]), upper=np.array([1.0, 1.0]))

    # depth=1 should split on axis 1
    left, right = _split(data, splitter, depth=1)

    assert np.all(left[:, 1] <= 0.5)
    assert np.all(right[:, 1] > 0.5)



def test_quadtree_child_splitters_update_bounds():
    splitter = Quadtree(lower=np.array([0.0, 0.0]), upper=np.array([4.0, 2.0]))
    left, right = _child_splitters(splitter, depth=0)

    assert np.allclose(left.lower, [0.0, 0.0])
    assert np.allclose(left.upper, [2.0, 2.0])
    assert np.allclose(right.lower, [2.0, 0.0])
    assert np.allclose(right.upper, [4.0, 2.0])



def test_lsh_split_uses_hyperplane_sign():
    data = np.array([
        [-2.0, 0.0],
        [-0.5, 1.0],
        [0.5, -1.0],
        [3.0, 0.0],
    ])
    splitter = LSH(hyperplanes=np.array([[1.0, 0.0]]))

    left, right = _split(data, splitter, depth=0)

    assert np.all((left @ np.array([1.0, 0.0])) <= 0.0)
    assert np.all((right @ np.array([1.0, 0.0])) > 0.0)
    assert len(left) == 2
    assert len(right) == 2



def test_lsh_child_splitters_share_hyperplanes():
    hp = np.array([[1.0, 0.0], [0.0, 1.0]])
    splitter = LSH(hyperplanes=hp)
    left, right = _child_splitters(splitter, depth=0)

    assert left is not right
    assert np.shares_memory(left.hyperplanes, hp)
    assert np.shares_memory(right.hyperplanes, hp)



def test_lsh_split_past_last_hyperplane_puts_everything_left():
    data = np.array([[1.0, 2.0], [3.0, 4.0]])
    splitter = LSH(hyperplanes=np.array([[1.0, 0.0]]))

    left, right = _split(data, splitter, depth=5)

    assert np.array_equal(left, data)
    assert right.shape == (0, data.shape[1])


def test_lsh_split_rejects_bad_hyperplane_shape():
    data = np.ones((3, 2))
    splitter = LSH(hyperplanes=np.array([[1.0, 0.0, 0.0]]))
    with pytest.raises(ValueError, match="hyperplanes"):
        _split(data, splitter, depth=0)


def test_split_rejects_unknown_splitter_type():
    with pytest.raises(TypeError, match="unknown splitter"):
        _split(np.ones((2, 2)), SimpleSplitter(), depth=0)  # type: ignore[arg-type]


def test_child_splitters_reject_unknown_splitter_type():
    with pytest.raises(TypeError, match="unknown splitter"):
        _child_splitters(SimpleSplitter(), depth=0)  # type: ignore[arg-type]



def test_tree_structure_one_level_quadtree():
    data = np.array([
        [0.1, 0.1],
        [0.2, 0.8],
        [0.4, 0.4],
        [0.49, 0.9],
        [0.6, 0.2],
        [0.7, 0.7],
        [0.8, 0.1],
        [0.95, 0.95],
    ])
    splitter = Quadtree(lower=np.array([0.0, 0.0]), upper=np.array([1.0, 1.0]))
    meas = make_private_tree(
        input_domain=dp.numpy.array2_domain(T=float, num_columns=2),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
        splitter=splitter,
        level_allocation=[1.0],
        scale=1e-6,
    )

    children = meas(data)
    assert len(children) == 2
    assert all(child.children is None for child in children)
    assert _node_values(children) == pytest.approx([4.0, 4.0], abs=1e-3)



def test_tree_structure_two_level_quadtree():
    data = np.array([
        [0.1, 0.1],
        [0.2, 0.2],
        [0.3, 0.7],
        [0.4, 0.9],
        [0.6, 0.1],
        [0.7, 0.4],
        [0.8, 0.7],
        [0.9, 0.95],
    ])
    splitter = Quadtree(lower=np.array([0.0, 0.0]), upper=np.array([1.0, 1.0]))
    meas = make_private_tree(
        input_domain=dp.numpy.array2_domain(T=float, num_columns=2),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
        splitter=splitter,
        level_allocation=[1.0, 1.0],
        scale=1e-6,
    )

    children = meas(data)
    assert len(children) == 2
    assert _node_values(children) == pytest.approx([4.0, 4.0], abs=1e-3)
    assert all(child.children is not None for child in children)

    left_grandchildren = children[0].children
    right_grandchildren = children[1].children
    assert len(left_grandchildren) == 2
    assert len(right_grandchildren) == 2
    assert _node_values(left_grandchildren) == pytest.approx([2.0, 2.0], abs=1e-3)
    assert _node_values(right_grandchildren) == pytest.approx([2.0, 2.0], abs=1e-3)



def test_tree_structure_two_level_lsh():
    data = np.array([
        [-2.0, -2.0],
        [-1.0, -0.5],
        [-2.0, 1.0],
        [-0.5, 2.0],
        [0.5, -2.0],
        [2.0, -0.5],
        [1.0, 1.0],
        [2.0, 2.0],
    ])
    splitter = LSH(hyperplanes=np.array([
        [1.0, 0.0],
        [0.0, 1.0],
    ]))
    meas = make_private_tree(
        input_domain=dp.numpy.array2_domain(T=float, num_columns=2),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
        splitter=splitter,
        level_allocation=[1.0, 1.0],
        scale=1e-6,
    )

    children = meas(data)
    assert len(children) == 2
    assert _node_values(children) == pytest.approx([4.0, 4.0], abs=1e-3)
    assert all(child.children is not None for child in children)
    assert _node_values(children[0].children) == pytest.approx([2.0, 2.0], abs=1e-3)
    assert _node_values(children[1].children) == pytest.approx([2.0, 2.0], abs=1e-3)



def test_tree_structure_mixed_recursion_and_leaf_refine():
    data = np.array([
        [0.1, 0.1],
        [0.2, 0.2],
        [0.3, 0.8],
        [0.45, 0.9],
        [0.9, 0.9],
    ])
    splitter = Quadtree(lower=np.array([0.0, 0.0]), upper=np.array([1.0, 1.0]))
    meas = make_private_tree(
        input_domain=dp.numpy.array2_domain(T=float, num_columns=2),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
        splitter=splitter,
        level_allocation=[1.0, 1.0],
        scale=1e-6,
    )

    children = meas(data)
    assert len(children) == 2
    # With the current budget-first recursion rule, both children recurse.
    assert children[0].children is not None
    assert children[1].children is not None
    assert children[0].value == pytest.approx(4.0, abs=1e-3)
    assert children[1].value == pytest.approx(1.0, abs=1e-3)
    assert _node_values(children[0].children) == pytest.approx([2.0, 2.0], abs=1e-3)
    assert _node_values(children[1].children) == pytest.approx([0.0, 1.0], abs=1e-3)


def test_make_parallel_composition_rejects_empty_measurements():
    with pytest.raises(ValueError, match="nonempty"):
        make_parallel_composition([])


def test_make_parallel_composition_requires_matching_domains_metrics_and_measures():
    m1 = make_private_tree(
        input_domain=dp.numpy.array2_domain(T=float, num_columns=2),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
        splitter=Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0])),
        level_allocation=[1.0],
        scale=1.0,
    )
    m2 = make_private_tree(
        input_domain=dp.numpy.array2_domain(T=float, num_columns=3),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
        splitter=Quadtree(np.array([0.0, 0.0, 0.0]), np.array([1.0, 1.0, 1.0])),
        level_allocation=[1.0],
        scale=1.0,
    )
    with pytest.raises(ValueError, match="input domain"):
        make_parallel_composition([m1, m2])

    m3 = make_private_tree(
        input_domain=dp.numpy.array2_domain(T=float, num_columns=2),
        input_metric=dp.change_one_distance(),
        output_measure=dp.max_divergence(),
        splitter=Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0])),
        level_allocation=[1.0],
        scale=1.0,
    )
    with pytest.raises(ValueError, match="input metric"):
        make_parallel_composition([m1, m3])

    m4 = make_private_tree(
        input_domain=dp.numpy.array2_domain(T=float, num_columns=2),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.zero_concentrated_divergence(),
        splitter=Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0])),
        level_allocation=[1.0],
        scale=1.0,
    )
    with pytest.raises(ValueError, match="output measure"):
        make_parallel_composition([m1, m4])


def test_count_metric_and_noise_var_cover_supported_measures():
    assert _count_metric(dp.max_divergence()) == dp.l1_distance(T=int)
    assert _count_metric(dp.zero_concentrated_divergence()) == dp.l2_distance(T=int)
    assert np.isclose(_noise_var(dp.max_divergence(), 3.0), 18.0)
    assert np.isclose(_noise_var(dp.zero_concentrated_divergence(), 3.0), 9.0)


def test_count_metric_and_noise_var_reject_unsupported_measures():
    with pytest.raises(ValueError, match="output_measure"):
        _count_metric(dp.approximate(dp.max_divergence()))
    with pytest.raises(ValueError, match="output_measure"):
        _noise_var(dp.approximate(dp.max_divergence()), 1.0)


def test_budget_scale_conversions_cover_supported_measures():
    budget = 2.0
    max_scale = _noise_scale_for_budget(dp.max_divergence(), budget)
    zcdp_scale = _noise_scale_for_budget(dp.zero_concentrated_divergence(), budget)
    assert max_scale > 0.0
    assert zcdp_scale > 0.0
    assert _privacy_budget_for_noise_scale(dp.max_divergence(), 0.5) == pytest.approx(2.0)
    assert _privacy_budget_for_noise_scale(dp.zero_concentrated_divergence(), 0.5) == pytest.approx(2.0)


@pytest.mark.parametrize("bad_budget", [0.0, -1.0])
def test_noise_scale_for_budget_requires_positive_budget(bad_budget):
    with pytest.raises(ValueError, match="budget must be positive"):
        _noise_scale_for_budget(dp.max_divergence(), bad_budget)


@pytest.mark.parametrize("bad_scale", [0.0, -1.0])
def test_privacy_budget_for_noise_scale_requires_positive_scale(bad_scale):
    with pytest.raises(ValueError, match="scale must be positive"):
        _privacy_budget_for_noise_scale(dp.max_divergence(), bad_scale)


def test_budget_scale_conversions_reject_unsupported_measure():
    approx = dp.approximate(dp.max_divergence())
    with pytest.raises(ValueError, match="output_measure"):
        _noise_scale_for_budget(approx, 1.0)
    with pytest.raises(ValueError, match="output_measure"):
        _privacy_budget_for_noise_scale(approx, 1.0)


def test_leaf_penalty_covers_both_objectives_and_rejects_invalid_objective():
    splitter = Quadtree(np.array([0.0, 0.0]), np.array([3.0, 4.0]))
    assert _leaf_penalty(2.0, splitter, "kmedians") == pytest.approx(10.0)
    assert _leaf_penalty(2.0, splitter, "kmeans") == pytest.approx(50.0)
    with pytest.raises(ValueError, match="objective"):
        _leaf_penalty(1.0, splitter, "bad")


def test_solve_tree_validates_inputs():
    splitter = Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0]))
    with pytest.raises(NotImplementedError, match="Quadtree"):
        solve_tree(
            [Node(1.0, 1.0), Node(1.0, 1.0)],
            cast(Quadtree, LSH(np.array([[1.0, 0.0]]))),
            1,
        )
    with pytest.raises(ValueError, match="nonnegative"):
        solve_tree([Node(1.0, 1.0), Node(1.0, 1.0)], splitter, -1)


class SimpleSplitter:
    pass
