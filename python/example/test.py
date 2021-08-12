from opendp.trans import *
from opendp.meas import *
from opendp.comb import *

from opendp.typing import SubstituteDistance


def main():

    ### HELLO WORLD
    identity = make_identity(M=SubstituteDistance, T=str)
    arg = "hello, world!"
    res = identity(arg)
    print(res)

    ### SUMMARY STATS
    # Parse dataframe
    parse_dataframe = (
        make_split_dataframe(separator=",", col_names=["A", "B", "C"]) >>
        make_parse_column(key="B", T=int) >>
        make_parse_column(key="C", T=float)
    )

    # Noisy sum, col 1
    noisy_sum_1 = (
        make_select_column(key="B", T=int) >>
        make_clamp(lower=0, upper=10) >>
        make_bounded_sum(lower=0, upper=10) >>
        make_base_geometric(scale=1.0)
    )

    # Count, col 2
    noisy_count_2 = (
        make_select_column(key="C", T=float) >>
        make_count(TIA=float) >>
        make_base_geometric(scale=1.0)
    )

    arg = "ant, 1, 1.1\nbat, 2, 2.2\ncat, 3, 3.3"

    # Compose & chain
    everything = parse_dataframe >> make_basic_composition(noisy_sum_1, noisy_count_2)
    print(everything(arg))


if __name__ == "__main__":
    main()
