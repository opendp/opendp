from opendp.v1.trans import *
from opendp.v1.meas import *

from opendp.v1.typing import HammingDistance, L1Distance


def main():

    ### HELLO WORLD
    identity = make_identity(M=HammingDistance, T=str)
    arg = "hello, world!"
    res = identity(arg)
    print(res)

    ### SUMMARY STATS
    # Parse dataframe
    parse_dataframe = (
        make_split_dataframe(separator=",", col_names=[0, 1, 2], M=HammingDistance) >>
        make_parse_column(key=1, impute=True, M=HammingDistance, T=int) >>
        make_parse_column(key=2, impute=True, M=HammingDistance, T=float)
    )

    # Noisy sum, col 1
    noisy_sum_1 = (
            make_select_column(key=1, M=HammingDistance, T=int) >>
            make_clamp(lower=0, upper=10, M=HammingDistance) >>
            make_bounded_sum(lower=0, upper=10, MI=HammingDistance, MO=L1Distance[int]) >>
            make_base_geometric(scale=1.0)
    )

    # Count, col 1
    noisy_count_2 = (
            make_select_column(key=2, M=HammingDistance, T=float) >>
            make_count(MI=HammingDistance, MO=L1Distance[int], TI=float) >>
            make_base_geometric(scale=1.0)
    )

    arg = "ant, 1, 1.1\nbat, 2, 2.2\ncat, 3, 3.3"

    # Compose & chain
    # everything = parse_dataframe >> make_basic_composition(noisy_sum_1, noisy_count_2)
    # res = everything(arg)

    # TODO: temporary until composition is worked out
    res = ((parse_dataframe >> noisy_sum_1)(arg), (parse_dataframe >> noisy_count_2)(arg))
    print(res)


if __name__ == "__main__":
    main()
