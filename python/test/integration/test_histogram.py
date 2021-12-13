from opendp.mod import enable_features

enable_features('contrib')


def test_count_by_categories():
    """Compute histogram with known category set"""
    from opendp.trans import make_count_by_categories, make_split_dataframe, make_select_column
    from opendp.meas import make_base_discrete_laplace
    from opendp.typing import L1Distance, VectorDomain, AllDomain
    from opendp.mod import binary_search_chain
    preprocess = (
        make_split_dataframe(",", ['A', 'B']) >>
        make_select_column("A", TOA=str) >>
        make_count_by_categories(categories=["a", "b", "c"], MO=L1Distance[int], TIA=str)
    )

    noisy_histogram_from_dataframe = binary_search_chain(
        lambda s: preprocess >> make_base_discrete_laplace(s, D=VectorDomain[AllDomain[int]]),
        d_in=1, d_out=1.)

    assert noisy_histogram_from_dataframe.check(1, 1.)

    data = "\n".join(["a"] * 25 + ["b"] * 25 + ["what?"] * 10)

    print(noisy_histogram_from_dataframe(data))


def test_count_by_categories_float():
    """Compute histogram with known category set"""
    from opendp.trans import make_count_by_categories, make_split_dataframe, make_select_column
    from opendp.meas import make_base_laplace, make_base_gaussian
    from opendp.typing import L1Distance, L2Distance, VectorDomain, AllDomain
    from opendp.mod import enable_features
    enable_features("floating-point")
    noisy_float_histogram = (
            make_split_dataframe(",", ['A', 'B']) >>
            make_select_column("A", TOA=str) >>
            make_count_by_categories(categories=["a", "b", "c"], MO=L1Distance[float], TIA=str, TOA=float) >>
            make_base_laplace(scale=1., D=VectorDomain[AllDomain[float]])
    )
    print(noisy_float_histogram("\n".join(["a"] * 5 + ["b"] * 20 + ["c"] * 10 + ["z"] * 5)))

    noisy_float_histogram = (
            make_split_dataframe(",", ['A', 'B']) >>
            make_select_column("A", TOA=str) >>
            make_count_by_categories(categories=["a", "b", "c"], MO=L2Distance[float], TIA=str, TOA=float) >>
            make_base_gaussian(scale=1., D=VectorDomain[AllDomain[float]])
    )
    print(noisy_float_histogram("\n".join(["a"] * 5 + ["b"] * 20 + ["c"] * 10 + ["z"] * 5)))


def test_count_by_ptr():
    """Compute histogram with unknown category set"""
    from opendp.trans import make_split_dataframe, make_select_column, make_count_by
    from opendp.meas import make_base_ptr
    from opendp.typing import L1Distance
    from opendp.mod import binary_search_param, enable_features
    enable_features("floating-point")

    preprocess = (
        make_split_dataframe(",", ['A', 'B']) >>
        make_select_column("A", TOA=str) >>
        make_count_by(MO=L1Distance[float], TK=str, TV=float)
    )
    budget = (1., 1e-8)
    scale = binary_search_param(
        lambda s: preprocess >> make_base_ptr(scale=s, threshold=1e8, TK=str),
        d_in=1, d_out=budget)
    threshold = binary_search_param(
        lambda t: preprocess >> make_base_ptr(scale=scale, threshold=t, TK=str),
        d_in=1, d_out=budget)

    laplace_histogram_from_dataframe = \
        preprocess >> make_base_ptr(scale=scale, threshold=threshold, TK=str)

    assert laplace_histogram_from_dataframe.check(1, budget)

    data = "\n".join(["a"] * 500 + ["b"] * 200 + ["what?"] * 100)

    print(laplace_histogram_from_dataframe(data))
