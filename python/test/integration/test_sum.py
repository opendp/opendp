from opendp.mod import enable_features
enable_features("floating-point", "contrib")


def test_sized_bounded_float_sum():
    """known-n bounded float sum (assuming n is public)"""
    from opendp.transformations import make_split_dataframe, make_select_column, \
        make_cast, make_impute_constant, \
        then_clamp, then_resize, then_sum
    from opendp.measurements import then_base_laplace, then_base_gaussian
    from opendp.combinators import make_fix_delta, make_zCDP_to_approxDP
    from opendp.mod import binary_search_chain
    from opendp.domains import atom_domain, option_domain

    size = 200
    bounds = (0., 20.)

    preprocess = (
        # Convert csv string into a dataframe of String columns
        make_split_dataframe(",", ['A', 'B']) >>
        # Selects a column of df, Vec<str>
        make_select_column("A", TOA=str) >>
        # Cast the column as Vec<Option<Float>>
        make_cast(TIA=str, TOA=float) >>
        # Impute missing values to 0, emit Vec<Float>
        make_impute_constant(option_domain(atom_domain(T=float)), constant=0.) >>
        # Clamp values
        then_clamp(bounds=bounds) >>
        # Resize dataset length
        then_resize(size=size, constant=0.) >>
        # Aggregate with sum
        then_sum()
    )

    # Add noise such that when d_in=1, the result is 1 epsilon DP
    laplace_known_n_sum_from_dataframe = binary_search_chain(
        lambda s: preprocess >> then_base_laplace(s),
        d_in=1, d_out=1.)

    gaussian_known_n_sum_from_dataframe = binary_search_chain(
        lambda s: make_fix_delta(make_zCDP_to_approxDP(preprocess >> then_base_gaussian(s)), 1e-5),
        d_in=1, d_out=(1., 1e-5))

    assert laplace_known_n_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * size)

    print(laplace_known_n_sum_from_dataframe(data))
    print(gaussian_known_n_sum_from_dataframe(data))


def test_sized_bounded_int_sum():
    """known-n bounded int sum (assuming n is public)"""
    from opendp.transformations import make_split_dataframe, make_select_column, \
        make_cast, make_impute_constant, \
        then_clamp, then_resize, then_sum
    from opendp.measurements import then_base_discrete_laplace
    from opendp.mod import binary_search_chain
    from opendp.domains import atom_domain, option_domain

    size = 200
    bounds = (0, 20)

    preprocess = (
        # Convert csv string into a dataframe of String columns
        make_split_dataframe(",", ['A', 'B']) >>
        # Selects a column of df, Vec<str>
        make_select_column("A", TOA=str) >>
        # Cast the column as Vec<Optional<int>>
        make_cast(TIA=str, TOA=int) >>
        # Impute missing values to 0, emit Vec<int>
        make_impute_constant(option_domain(atom_domain(T=int)), constant=0) >>
        # Clamp values
        then_clamp(bounds=bounds) >>
        # Resize dataset length
        then_resize(size=size, constant=0) >>
        # Aggregate with sum
        then_sum()
    )

    noisy_known_n_sum_from_dataframe = binary_search_chain(
        lambda s: preprocess >> then_base_discrete_laplace(s),
        d_in=1, d_out=1.)

    assert noisy_known_n_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * size)

    print(noisy_known_n_sum_from_dataframe(data))


def test_bounded_float_sum():
    """bounded float sum (assuming n is unknown)"""
    from opendp.transformations import make_split_dataframe, make_select_column, \
        make_cast, make_impute_constant, \
        then_clamp, then_sum
    from opendp.measurements import then_base_laplace, then_base_gaussian
    from opendp.combinators import make_fix_delta, make_zCDP_to_approxDP
    from opendp.mod import binary_search_chain
    from opendp.domains import option_domain, atom_domain
    
    bounds = (0., 20.)

    preprocess = (
        # Convert csv string into a dataframe of String columns
        make_split_dataframe(",", ['A', 'B']) >>
        # Selects a column of df, Vec<str>
        make_select_column("A", TOA=str) >>
        # Cast the column as Vec<Option<float>>
        make_cast(TIA=str, TOA=float) >>
        # Impute missing values to 0, emit Vec<float>
        make_impute_constant(option_domain(atom_domain(T=float)), constant=0.) >>
        # Clamp values
        then_clamp(bounds=bounds) >>
        # Aggregate with sum.
        then_sum()
    )

    laplace_sum_from_dataframe = binary_search_chain(
        lambda s: preprocess >> then_base_laplace(s),
        d_in=1, d_out=1.)

    gaussian_sum_from_dataframe = binary_search_chain(
        lambda s: make_fix_delta(make_zCDP_to_approxDP(preprocess >> then_base_gaussian(s)), 1e-5),
        d_in=1, d_out=(1., 1e-5))

    assert laplace_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * 100)

    print(laplace_sum_from_dataframe(data))
    print(gaussian_sum_from_dataframe(data))


def test_bounded_int_sum():
    """bounded int sum (assuming n is unknown)"""
    from opendp.transformations import make_split_dataframe, make_select_column, \
        make_cast, make_impute_constant, \
        then_clamp, then_sum
    from opendp.measurements import then_base_discrete_laplace
    from opendp.mod import binary_search_chain
    from opendp.domains import option_domain, atom_domain

    bounds = (0, 20)

    preprocess = (
        make_split_dataframe(",", ['A', 'B']) >>
        make_select_column("A", TOA=str) >>
        make_cast(TIA=str, TOA=int) >>
        make_impute_constant(option_domain(atom_domain(T=int)), constant=0) >>
        then_clamp(bounds=bounds) >>
        then_sum()
    )

    noisy_sum_from_dataframe = binary_search_chain(
        lambda s: preprocess >> then_base_discrete_laplace(s),
        d_in=1, d_out=1.)

    assert noisy_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * 100)

    print(noisy_sum_from_dataframe(data))

