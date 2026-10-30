import opendp.prelude as dp
import pytest


def test_count_by_categories():
    """Compute histogram with known category set"""
    space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    preprocess = space >> dp.t.then_count_by_categories(
        categories=["a", "b", "c"], MO=dp.L1Distance[int]
    )
    noisy_histogram = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s), d_in=1, d_out=1.0
    )
    assert noisy_histogram.check(1, 1.0)
    print(noisy_histogram(["a"] * 25 + ["b"] * 25 + ["what?"] * 10))


def test_count_by_categories_float():
    """Compute histogram with known category set"""
    data = ["a"] * 5 + ["b"] * 20 + ["c"] * 10 + ["z"] * 5
    cats = ["a", "b", "c"]
    space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    assert (
        space
        >> dp.t.then_count_by_categories(cats, MO=dp.L1Distance[float], TOA=float)
        >> dp.m.then_laplace(0.0)
    )(data) == [5.0, 20.0, 10.0, 5.0]
    assert (
        space
        >> dp.t.then_count_by_categories(cats, MO=dp.L2Distance[float], TOA=float)
        >> dp.m.then_gaussian(0.0)
    )(data) == [5.0, 20.0, 10.0, 5.0]


def test_count_by_threshold():
    """Compute histogram with unknown category set"""
    space = dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance()
    pre = space >> dp.t.then_count_by()
    budget = (1.0, 1e-8)
    scale = dp.binary_search(
        lambda s: (
            (pre >> dp.m.then_laplace_threshold(scale=s, threshold=100_000)).map(1)[0]
            <= budget[0]
        )
    )
    threshold = dp.binary_search(
        lambda t: (
            (pre >> dp.m.then_laplace_threshold(scale=scale, threshold=t)).map(1)[1]
            <= budget[1]
        ),
    )
    laplace_histogram = pre >> dp.m.then_laplace_threshold(
        scale=scale, threshold=threshold
    )
    assert laplace_histogram.check(1, budget)
    data = ["a"] * 500 + ["b"] * 200 + ["other"] * 100
    assert pre(data) == {"a": 500, "b": 200, "other": 100}
    print("laplace_histogram", laplace_histogram(data))
    assert scale == 1.0
    assert threshold == 19
    assert (pre >> dp.m.then_laplace_threshold(scale=0.0, threshold=threshold)).map(
        1
    ) == (float("inf"), 1.0)


def test_b_ary_tree_output_metric():
    domain = dp.vector_domain(dp.atom_domain(T=int), size=4)
    l1 = dp.t.make_b_ary_tree(
        domain, dp.l1_distance(T=int), dp.l1_distance(T=int), 4, 4
    )
    l2 = dp.t.make_b_ary_tree(
        domain,
        dp.l1_distance(T=int),
        dp.l2_distance(T=float),
        leaf_count=4,
        branching_factor=4,
    )
    assert l1.map(1) == 2
    assert 1.414 < l2.map(1) < 1.415
    assert l2([1, 1, 1, 1]) == [4, 1, 1, 1, 1]

    # L2 input is not supported: its stability cannot use the L1 bound.
    with pytest.raises(Exception):
        dp.t.make_b_ary_tree(
            domain, dp.l2_distance(T=int), dp.l2_distance(T=float), 4, 4
        )
