

def test_sized_bounded_variance():
    """known-n bounded float sum (assuming n is public)"""
    from opendp.transformations import make_split_dataframe, make_select_column, \
        make_cast, make_impute_constant, \
        then_clamp, then_resize, make_sized_bounded_variance
    from opendp.domains import atom_domain
    from opendp.measurements import then_base_laplace
    from opendp.mod import binary_search_chain, enable_features
    from opendp.domains import option_domain, atom_domain

    enable_features("floating-point", "contrib")

    size = 200
    bounds = (0., 20.)

    preprocess = (
        # Convert csv string into a dataframe of String columns
        make_split_dataframe(",", ['A', 'B']) >>
        # Selects a column of df, Vec<str>
        make_select_column("A", TOA=str) >>
        # Cast the column as Vec<Optional<Float>>
        make_cast(TIA=str, TOA=float) >>
        # Impute missing values to 0, emit Vec<Float>
        make_impute_constant(option_domain(atom_domain(T=float)), constant=0.) >>
        # Clamp values
        then_clamp(bounds=bounds) >>
        # Resize dataset length
        then_resize(size=size, constant=0.) >>
        # Aggregate with variance
        make_sized_bounded_variance(size=size, bounds=bounds)
    )

    noisy_known_n_variance_from_dataframe = binary_search_chain(
        lambda s: preprocess >> then_base_laplace(s),
        d_in=1, d_out=1.)

    assert noisy_known_n_variance_from_dataframe.check(1, 1.)

    data = "\n".join(["1"] * size)

    print(noisy_known_n_variance_from_dataframe(data))
