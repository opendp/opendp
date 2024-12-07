import opendp.prelude as dp


def test_sized_bounded_float_sum():
    """known-n bounded float sum (assuming n is public)"""

    size = 200
    bounds = (0., 20.)

    preprocess = (
        # Convert csv string into a dataframe of String columns
        dp.t.make_split_dataframe(",", ['A', 'B']) >>
        # Selects a column of df, Vec<str>
        dp.t.make_select_column("A", TOA=str) >>
        # Cast the column as Vec<Option<Float>>
        dp.t.then_cast(TOA=float) >>
        # Impute missing values to 0, emit Vec<Float>
        dp.t.then_impute_constant(constant=0.) >>
        # Clamp values
        dp.t.then_clamp(bounds=bounds) >>
        # Resize dataset length
        dp.t.then_resize(size=size, constant=0.) >>
        # Aggregate with sum
        dp.t.then_sum()
    )

    # Add noise such that when d_in=1, the result is 1 epsilon DP
    laplace_known_n_sum_from_dataframe = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s),
        d_in=1, d_out=1.)

    gaussian_known_n_sum_from_dataframe = dp.binary_search_chain(
        lambda s: dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(preprocess >> dp.m.then_gaussian(s)), 1e-5),
        d_in=1, d_out=(1., 1e-5))

    assert laplace_known_n_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * size)

    print("TODO: explain", laplace_known_n_sum_from_dataframe(data))
    print("TODO: explain", gaussian_known_n_sum_from_dataframe(data))


def test_sized_bounded_int_sum():
    """known-n bounded int sum (assuming n is public)"""
    
    size = 200
    bounds = (0, 20)

    preprocess = (
        # Convert csv string into a dataframe of String columns
        dp.t.make_split_dataframe(",", ['A', 'B']) >>
        # Selects a column of df, Vec<str>
        dp.t.make_select_column("A", TOA=str) >>
        # Cast the column as Vec<Optional<int>>
        dp.t.then_cast(TOA=int) >>
        # Impute missing values to 0, emit Vec<int>
        dp.t.then_impute_constant(constant=0) >>
        # Clamp values
        dp.t.then_clamp(bounds=bounds) >>
        # Resize dataset length
        dp.t.then_resize(size=size, constant=0) >>
        # Aggregate with sum
        dp.t.then_sum()
    )

    noisy_known_n_sum_from_dataframe = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s),
        d_in=1, d_out=1.)

    assert noisy_known_n_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * size)

    print("TODO: explain", noisy_known_n_sum_from_dataframe(data))


def test_bounded_float_sum():
    """bounded float sum (assuming n is unknown)"""
    bounds = (0., 20.)

    preprocess = (
        # Convert csv string into a dataframe of String columns
        dp.t.make_split_dataframe(",", ['A', 'B']) >>
        # Selects a column of df, Vec<str>
        dp.t.make_select_column("A", TOA=str) >>
        # Cast the column as Vec<Option<float>>
        dp.t.then_cast(TOA=float) >>
        # Impute missing values to 0, emit Vec<float>
        dp.t.then_impute_constant(constant=0.) >>
        # Clamp values
        dp.t.then_clamp(bounds=bounds) >>
        # Aggregate with sum.
        dp.t.then_sum()
    )

    laplace_sum_from_dataframe = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s),
        d_in=1, d_out=1.)

    gaussian_sum_from_dataframe = dp.binary_search_chain(
        lambda s: dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(preprocess >> dp.m.then_gaussian(s)), 1e-5),
        d_in=1, d_out=(1., 1e-5))

    assert laplace_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * 100)

    print("TODO: explain", laplace_sum_from_dataframe(data))
    print("TODO: explain", gaussian_sum_from_dataframe(data))


def test_bounded_int_sum():
    """bounded int sum (assuming n is unknown)"""

    bounds = (0, 20)

    preprocess = (
        dp.t.make_split_dataframe(",", ['A', 'B']) >>
        dp.t.make_select_column("A", TOA=str) >>
        dp.t.then_cast(TOA=int) >>
        dp.t.then_impute_constant(constant=0) >>
        dp.t.then_clamp(bounds=bounds) >>
        dp.t.then_sum()
    )

    noisy_sum_from_dataframe = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s),
        d_in=1, d_out=1.)

    assert noisy_sum_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * 100)

    print("TODO: explain", noisy_sum_from_dataframe(data))

