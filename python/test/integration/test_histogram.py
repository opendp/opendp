import pytest
import opendp.prelude as dp

dp.enable_features("contrib", "floating-point")


def test_count_by_categories():
    """Compute histogram with known category set"""
    preprocess = (
        dp.t.make_split_dataframe(",", ["A", "B"])
        >> dp.t.make_select_column("A", TOA=str)
        >> dp.t.then_count_by_categories(
            categories=["a", "b", "c"], MO=dp.L1Distance[int]
        )
    )

    noisy_histogram_from_dataframe = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_base_discrete_laplace(s), d_in=1, d_out=1.0
    )

    assert noisy_histogram_from_dataframe.check(1, 1.0)

    data = "\n".join(["a"] * 25 + ["b"] * 25 + ["what?"] * 10)

    print(noisy_histogram_from_dataframe(data))


def test_count_by_categories_float():
    """Compute histogram with known category set"""
    noisy_float_histogram = (
        dp.t.make_split_dataframe(",", ["A", "B"])
        >> dp.t.make_select_column("A", TOA=str)
        >> dp.t.then_count_by_categories(
            categories=["a", "b", "c"], MO=dp.L1Distance[float], TOA=float
        )
        >> dp.m.then_base_laplace(scale=1.0)
    )
    print(
        noisy_float_histogram(
            "\n".join(["a"] * 5 + ["b"] * 20 + ["c"] * 10 + ["z"] * 5)
        )
    )

    noisy_float_histogram = (
        dp.t.make_split_dataframe(",", ["A", "B"])
        >> dp.t.make_select_column("A", TOA=str)
        >> dp.t.then_count_by_categories(
            categories=["a", "b", "c"], MO=dp.L2Distance[float], TOA=float
        )
        >> dp.m.then_base_gaussian(scale=1.0)
    )
    print(
        noisy_float_histogram(
            "\n".join(["a"] * 5 + ["b"] * 20 + ["c"] * 10 + ["z"] * 5)
        )
    )


def test_count_by_threshold():
    """Compute histogram with unknown category set"""
    pre = (
        dp.t.make_split_dataframe(",", ["A", "B"])
        >> dp.t.make_select_column("A", TOA=str)
        >> dp.t.then_count_by(MO=dp.L1Distance[float], TV=float)
    )
    budget = (1.0, 1e-8)
    scale = dp.binary_search_param(
        lambda s: pre >> dp.m.then_base_laplace_threshold(scale=s, threshold=1e8),
        d_in=1,
        d_out=budget,
    )
    threshold = dp.binary_search_param(
        lambda t: pre >> dp.m.then_base_laplace_threshold(scale=scale, threshold=t),
        d_in=1,
        d_out=budget,
    )

    laplace_histogram_from_dataframe = pre >> dp.m.then_base_laplace_threshold(
        scale=scale, threshold=threshold
    )

    assert laplace_histogram_from_dataframe.check(1, budget)

    data = "\n".join(["a"] * 500 + ["b"] * 200 + ["what?"] * 100)

    print(laplace_histogram_from_dataframe(data))


    with pytest.raises(dp.OpenDPException):
        dp.m.make_base_laplace_threshold(
            dp.atom_domain(T=int),
            dp.l1_distance(T=float),
            scale=1., threshold=1e8)
