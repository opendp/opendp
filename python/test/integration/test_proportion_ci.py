from opendp.trans import *
from opendp.meas import *
from opendp.comb import *
from opendp.mod import enable_features

enable_features("floating-point", "contrib")


def test_dp_proportion_cis():
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

    scale_mean = 0.05
    scale_var = 0.1

    mean_var_meas = (
        make_split_dataframe(separator=",", col_names=["values", "idents"]) >>
        make_df_is_equal("values", "1") >>
        make_filter_by("values", keep_columns=["idents"]) >>
        make_select_column("idents", TOA=str) >>
        make_count_by_categories(partition_idents) >>
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
    print("rho:", mean_var_meas.map(1))
    print("eps:", make_zCDP_to_approxDP(mean_var_meas).map(1).epsilon(1e-7))
