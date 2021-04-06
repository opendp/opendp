import opendp

def main():
    odp = opendp.OpenDP()

    ### HELLO WORLD
    identity = odp.trans.make_identity(b"<String>")
    arg = odp.data.from_string(b"hello, world!")
    ret = odp.core.transformation_invoke(identity, arg)
    print(odp.to_str(ret))
    odp.data.data_free(arg)
    odp.data.data_free(ret)
    odp.core.transformation_free(identity)

    ### SUMMARY STATS
    # Parse dataframe
    split_dataframe = odp.trans.make_split_dataframe(b"<HammingDistance>", b",", 3)
    parse_column_1 = odp.trans.make_parse_column(b"<HammingDistance, i32, i32>", opendp.i32_p(1), True)
    parse_column_2 = odp.trans.make_parse_column(b"<HammingDistance, i32, f64>", opendp.i32_p(2), True)
    parse_dataframe = odp.make_chain_tt_multi(parse_column_2, parse_column_1, split_dataframe)

    # Noisy sum, col 1
    select_1 = odp.trans.make_select_column(b"<HammingDistance, i32, i32>", opendp.i32_p(1))
    clamp_1 = odp.trans.make_clamp(b"<HammingDistance, i32>", opendp.i32_p(0), opendp.i32_p(10))
    bounded_sum_1 = odp.trans.make_bounded_sum(b"<HammingDistance, L1Sensitivity<i32>, i32>", opendp.i32_p(0), opendp.i32_p(10))
    base_geometric_1 = odp.meas.make_base_simple_geometric(b"<i32, f64>", opendp.f64_p(1.0), opendp.u32_p(0), opendp.u32_p(1000))
    # base_laplace_1 = odp.meas.make_base_laplace(b"<i32>", opendp.f64_p(1.0))
    noisy_sum_1 = odp.core.make_chain_mt(base_geometric_1, odp.make_chain_tt_multi(bounded_sum_1, clamp_1, select_1))

    # Count, col 1
    select_2 = odp.trans.make_select_column(b"<HammingDistance, i32, f64>", opendp.i32_p(2))
    count_2 = odp.trans.make_count(b"<HammingDistance, L1Sensitivity<u32>, f64>")
    base_geometric_2 = odp.meas.make_base_simple_geometric(b"<u32, f64>", opendp.f64_p(1.0), opendp.u32_p(0), opendp.u32_p(1000))
    noisy_count_2 = odp.core.make_chain_mt(base_geometric_2, odp.make_chain_tt_multi(count_2, select_2))

    # Compose & chain
    composition = odp.core.make_composition(noisy_sum_1, noisy_count_2)
    everything = odp.core.make_chain_mt(composition, parse_dataframe)

    # Do it!!!
    arg = odp.data.from_string(b"ant, 1, 1.1\nbat, 2, 2.2\ncat, 3, 3.3")
    ret = odp.core.measurement_invoke(everything, arg)
    print(odp.to_str(ret))

    # Clean up
    odp.data.data_free(arg)
    odp.data.data_free(ret)
    odp.core.measurement_free(everything)

if __name__ == "__main__":
    main()
