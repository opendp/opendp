from opendp.transformations import *
from opendp.measurements import *
from opendp.combinators import *

from opendp.typing import SymmetricDistance, VectorDomain, AtomDomain
from opendp.mod import enable_features
enable_features("contrib")

# used for testing a successful release


def main():

    # HELLO WORLD
    identity = make_identity(D=VectorDomain[AtomDomain[str]], M=SymmetricDistance)
    arg = ["hello, world!"]
    res = identity(arg)
    print(res)

    # SUMMARY STATS
    # Parse dataframe
    parse_dataframe = make_split_dataframe(separator=",", col_names=["A", "B", "C"])

    # Noisy sum, col 1
    noisy_sum_1 = (
        make_select_column(key="B", TOA=str) >>
        then_cast_default(TOA=int) >>
        then_clamp(bounds=(0, 10)) >>
        then_sum() >>
        make_base_discrete_laplace(scale=1.0)
    )

    # Count, col 2
    noisy_count_2 = (
        make_select_column(key="C", TOA=str) >>
        then_cast_default(TOA=float) >>
        then_count() >>
        make_base_discrete_laplace(scale=1.0)
    )

    arg = "ant, 1, 1.1\nbat, 2, 2.2\ncat, 3, 3.3"

    # Compose & chain
    everything = parse_dataframe >> make_basic_composition([noisy_sum_1, noisy_count_2])
    print(everything(arg))


if __name__ == "__main__":
    main()
