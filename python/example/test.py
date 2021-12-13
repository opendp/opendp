from opendp.trans import *
from opendp.meas import *
from opendp.comb import *

from opendp.typing import SubstituteDistance, VectorDomain, AllDomain
from opendp.mod import enable_features
enable_features("contrib")

# This is a short demonstration of opendp in a script.
# Includes an example of composition.


def main():

    # HELLO WORLD
    identity = make_identity(D=VectorDomain[AllDomain[str]], M=SubstituteDistance)
    arg = ["hello, world!"]
    res = identity(arg)
    print(res)

    # SUMMARY STATS
    # Parse dataframe
    parse_dataframe = make_split_dataframe(separator=",", col_names=["A", "B", "C"])

    # Noisy sum, col 1
    noisy_sum_1 = (
        make_select_column(key="B", TOA=str) >>
        make_cast_default(TIA=str, TOA=int) >>
        make_clamp(bounds=(0, 10)) >>
        make_bounded_sum(bounds=(0, 10)) >>
        make_base_discrete_laplace(scale=1.0)
    )

    # Count, col 2
    noisy_count_2 = (
        make_select_column(key="C", TOA=str) >>
        make_cast_default(TIA=str, TOA=float) >>
        make_count(TIA=float) >>
        make_base_discrete_laplace(scale=1.0)
    )

    arg = "ant, 1, 1.1\nbat, 2, 2.2\ncat, 3, 3.3"

    # Compose & chain
    everything = parse_dataframe >> make_basic_composition(noisy_sum_1, noisy_count_2)
    print(everything(arg))

    # TODO: update data unloaders to work recursively, to avoid needing the cast to a string


if __name__ == "__main__":
    main()
