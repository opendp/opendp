from opendp.trans import *
from opendp.meas import *
from opendp.comb import *
from opendp.mod import enable_features
from opendp.typing import InsertDeleteDistance

enable_features("floating-point", "contrib")


def test_dp_mean():
    partitions = 2
    sample_size = 1000

    strat_sizes = [sample_size * 2] * partitions
    sample_sizes = [sample_size] * partitions

    from string import ascii_uppercase
    partition_idents = [i for i in ascii_uppercase[:partitions]]

    from random import choice
    from itertools import chain
    values = [choice((0, 1)) for _ in range(sample_size * partitions)]
    idents = chain(*([ident] * sample_size for ident in partition_idents))

    data = "\n".join(f"{v},{i}" for v, i in zip(values, idents))

    bounds = (0., 1.)

    scale_mean = 1.
    scale_var = 1.

    mean_var_meas = (
        # Convert data into Vec<Vec<String>>
        make_split_dataframe(separator=",", col_names=["values", "idents"]) >>
        make_partition_by("idents", partition_idents, keep_columns=["values"]) >>
        make_partition_map_trans([
                make_select_column(key="values", TOA=str) >>
                make_cast_default(TIA=str, TOA=float) >>
                make_clamp(bounds) >>
                make_bounded_resize(sample_size_i, bounds, 0., MO=InsertDeleteDistance) >>
                make_sized_bounded_sum(sample_size_i, bounds)   
                for sample_size_i in sample_sizes
        ]) >> 
        make_basic_composition([
            (
                make_lipschitz_sized_proportion_ci_mean(strat_sizes, sample_sizes) >> 
                make_base_gaussian(scale_mean)
            ),
            (
                make_lipschitz_sized_proportion_ci_variance(strat_sizes, sample_sizes, scale_mean) >> 
                make_base_gaussian(scale_var)
            )
        ])
    )
    mean, var = mean_var_meas(data)
    print("mean:", mean)
    print("var:", var)
