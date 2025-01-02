import opendp.prelude as dp

def test_sized_bounded_variance():
    """known-n bounded float sum (assuming n is public)"""

    size = 200
    bounds = (0., 20.)

    preprocess = (
        # Convert csv string into a dataframe of String columns
        dp.t.make_split_dataframe(",", ['A', 'B']) >>
        # Selects a column of df, Vec<str>
        dp.t.make_select_column("A", TOA=str) >>
        # Cast the column as Vec<Optional<Float>>
        dp.t.then_cast(TOA=float) >>
        # Impute missing values to 0, emit Vec<Float>
        dp.t.then_impute_constant(constant=0.) >>
        # Clamp values
        dp.t.then_clamp(bounds=bounds) >>
        # Resize dataset length
        dp.t.then_resize(size=size, constant=0.) >>
        # Aggregate with variance
        dp.t.then_variance()
    )

    noisy_known_n_variance_from_dataframe = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_laplace(s),
        d_in=1, d_out=1.)

    assert noisy_known_n_variance_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * size)

    print("noisy_known_n_variance_from_dataframe(data)", noisy_known_n_variance_from_dataframe(data))
