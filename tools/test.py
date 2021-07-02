from opendp.transformations import *
from opendp.measurements import *
from opendp.combinators import *

from opendp.typing import ChangeOneDistance, VectorDomain, AllDomain
from opendp.mod import enable_features
enable_features("contrib")

# used for testing a successful release


def main():

    # HELLO WORLD
    identity = make_identity(D=VectorDomain[AllDomain[str]], M=ChangeOneDistance)
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
    everything = parse_dataframe >> make_basic_composition([noisy_sum_1, noisy_count_2])
    print(everything(arg))

    # TODO: update data unloaders to work recursively, to avoid needing the cast to a string


if __name__ == "__main__":
    main()


class SimpleMeasurement:
    def function(self, data):
        return None
class Queryable:
    def eval(self, query):
        return Queryable()
class InteractiveMeasurement:
    def function(self, data):
        return Queryable()

def make_simple_measurement():
    return SimpleMeasurement()
def make_interactive_measurement():
    return InteractiveMeasurement()
def make_sequential_comp():
    return InteractiveMeasurement()


# 0. Make a sequential composition and get the outer queryable
sequential_im = make_sequential_comp()
data = [1, 2, 3]
outer_queryable = sequential_im.function(data)

# 1. Spawn the first inner queryable (from an interactive measurement)
outer_query1 = make_interactive_measurement()
inner_queryable1 = outer_queryable.eval(outer_query1)

# 1.A. Query the first inner queryable (with some simple measurements)
inner_query1_1 = make_simple_measurement()
answer1_1 = inner_queryable1.eval(inner_query1_1)
inner_query1_2 = make_simple_measurement()
answer1_2 = inner_queryable1.eval(inner_query1_2)

# 2. Spawn the second inner queryable
outer_query2 = make_interactive_measurement()
inner_queryable2 = outer_queryable.eval(outer_query2)

# 2.A Query the first inner queryable (with some simple measurements)
inner_query2_1 = make_simple_measurement()
answer2_1 = inner_queryable2.eval(inner_query2_1)
