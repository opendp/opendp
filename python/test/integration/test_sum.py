

def test_sized_bounded_float_sum():
    """known-n bounded float sum (assuming n is public)"""
    from opendp.trans import make_split_dataframe, make_select_column, \
        make_cast, make_impute_constant, \
        make_clamp, make_bounded_resize, make_sized_bounded_sum
    from opendp.meas import make_base_laplace
    from opendp.mod import binary_search_chain, enable_features

    enable_features("floating-point")

    size = 200
    bounds = (0., 20.)

    preprocess = (
        make_split_dataframe(",", ['A', 'B']) >>
        make_select_column("A", TOA=str) >>
        make_cast(TIA=str, TOA=float) >>
        make_impute_constant(constant=0.) >>
        make_clamp(bounds=bounds) >>
        make_bounded_resize(size=size, bounds=bounds, constant=0.) >>
        make_sized_bounded_sum(size=size, bounds=bounds)
    )

    noisy_known_n_sum_from_dataframe = binary_search_chain(
        lambda s: preprocess >> make_base_laplace(s),
        d_in=1, d_out=1.)

    assert noisy_known_n_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * size)

    print(noisy_known_n_sum_from_dataframe(data))


def test_sized_bounded_int_sum():
    """known-n bounded int sum (assuming n is public)"""
    from opendp.trans import make_split_dataframe, make_select_column, \
        make_cast, make_impute_constant, \
        make_clamp, make_bounded_resize, make_sized_bounded_sum
    from opendp.meas import make_base_geometric
    from opendp.mod import binary_search_chain, enable_features

    enable_features("floating-point")

    size = 200
    bounds = (0, 20)

    preprocess = (
        make_split_dataframe(",", ['A', 'B']) >>
        make_select_column("A", TOA=str) >>
        make_cast(TIA=str, TOA=int) >>
        make_impute_constant(constant=0) >>
        make_clamp(bounds=bounds) >>
        make_bounded_resize(size=size, bounds=bounds, constant=0) >>
        make_sized_bounded_sum(size=size, bounds=bounds)
    )

    noisy_known_n_sum_from_dataframe = binary_search_chain(
        lambda s: preprocess >> make_base_geometric(s),
        d_in=1, d_out=1.)

    assert noisy_known_n_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * size)

    print(noisy_known_n_sum_from_dataframe(data))


def test_bounded_float_sum():
    """bounded float sum (assuming n is unknown)"""
    from opendp.trans import make_split_dataframe, make_select_column, \
        make_cast, make_impute_constant, \
        make_clamp, make_bounded_sum
    from opendp.meas import make_base_laplace
    from opendp.mod import binary_search_chain, enable_features

    enable_features("floating-point")
    bounds = (0., 20.)

    preprocess = (
        make_split_dataframe(",", ['A', 'B']) >>
        make_select_column("A", TOA=str) >>
        make_cast(TIA=str, TOA=float) >>
        make_impute_constant(constant=0.) >>
        make_clamp(bounds=bounds) >>
        make_bounded_sum(bounds=bounds)
    )

    noisy_sum_from_dataframe = binary_search_chain(
        lambda s: preprocess >> make_base_laplace(s),
        d_in=1, d_out=1.)

    assert noisy_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * 100)

    print(noisy_sum_from_dataframe(data))


def test_bounded_int_sum():
    """bounded int sum (assuming n is unknown)"""
    from opendp.trans import make_split_dataframe, make_select_column, \
        make_cast, make_impute_constant, \
        make_clamp, make_bounded_sum
    from opendp.meas import make_base_geometric
    from opendp.mod import binary_search_chain

    bounds = (0, 20)

    preprocess = (
        make_split_dataframe(",", ['A', 'B']) >>
        make_select_column("A", TOA=str) >>
        make_cast(TIA=str, TOA=int) >>
        make_impute_constant(constant=0) >>
        make_clamp(bounds=bounds) >>
        make_bounded_sum(bounds=bounds)
    )

    noisy_sum_from_dataframe = binary_search_chain(
        lambda s: preprocess >> make_base_geometric(s),
        d_in=1, d_out=1.)

    assert noisy_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * 100)

    print(noisy_sum_from_dataframe(data))

