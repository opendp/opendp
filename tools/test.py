import opendp.prelude as dp

dp.enable_features("contrib", "honest-but-curious")

# used for testing a successful release


def main():

    # HELLO WORLD
    identity = dp.t.make_identity(domain=dp.vector_domain(dp.atom_domain(T=str)), metric=dp.symmetric_distance())
    arg = ["hello, world!"]
    res = identity(arg)
    print(res)

    # SUMMARY STATS
    # Parse dataframe
    parse_dataframe = dp.t.make_split_dataframe(separator=",", col_names=["A", "B", "C"])

    # Noisy sum, col 1
    noisy_sum_1 = (
        dp.t.make_select_column(key="B", TOA=str) >>
        dp.t.then_cast_default(TOA=int) >>
        dp.t.then_clamp(bounds=(0, 10)) >>
        dp.t.then_sum() >>
        dp.m.then_laplace(scale=1.0)
    )

    # Count, col 2
    noisy_count_2 = (
        dp.t.make_select_column(key="C", TOA=str) >>
        dp.t.then_cast_default(TOA=float) >>
        dp.t.then_count() >>
        dp.m.then_laplace(scale=1.0)
    )

    arg = "ant, 1, 1.1\nbat, 2, 2.2\ncat, 3, 3.3"

    # Compose & chain
    everything = parse_dataframe >> dp.c.make_composition([noisy_sum_1, noisy_count_2])
    print(everything(arg))


if __name__ == "__main__":
    main()
