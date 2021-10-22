from opendp.mod import enable_features

enable_features('contrib')


def test_count_by_categories():
    """Compute histogram with known category set"""
    from opendp.trans import make_count_by_categories, make_split_dataframe, make_select_column
    from opendp.meas import make_base_geometric
    from opendp.typing import L1Distance, VectorDomain, AllDomain
    from opendp.mod import binary_search_chain
    preprocess = (
        make_split_dataframe(",", ['A', 'B']) >>
        make_select_column("A", TOA=str) >>
        make_count_by_categories(categories=["a", "b", "c"], MO=L1Distance[int], TIA=str)
    )

    noisy_histogram_from_dataframe = binary_search_chain(
        lambda s: preprocess >> make_base_geometric(s, D=VectorDomain[AllDomain[int]]),
        d_in=1, d_out=1.)

    assert noisy_histogram_from_dataframe.check(1, 1.)

    data = "\n".join(["a"] * 25 + ["b"] * 25 + ["what?"] * 10)

    print(noisy_histogram_from_dataframe(data))


def test_count_by_ptr():
    """Compute histogram with unknown category set"""
    from opendp.trans import make_split_dataframe, make_select_column
    from opendp.meas import make_count_by_ptr
    from opendp.mod import binary_search_param, enable_features
    enable_features("floating-point")

    preprocess = (
        make_split_dataframe(",", ['A', 'B']) >>
        make_select_column("A", TOA=str)
    )
    budget = (1., 1e-8)
    scale = binary_search_param(
        lambda s: preprocess >> make_count_by_ptr(scale=s, threshold=1e8, TIA=str),
        d_in=1, d_out=budget)
    threshold = binary_search_param(
        lambda t: preprocess >> make_count_by_ptr(scale=scale, threshold=t, TIA=str),
        d_in=1, d_out=budget)

    laplace_histogram_from_dataframe = \
        preprocess >> make_count_by_ptr(scale=scale, threshold=threshold, TIA=str)

    assert laplace_histogram_from_dataframe.check(1, budget)

    data = "\n".join(["a"] * 500 + ["b"] * 200 + ["what?"] * 100)

    print(laplace_histogram_from_dataframe(data))
