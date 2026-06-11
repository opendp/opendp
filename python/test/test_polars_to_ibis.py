import pytest

import opendp.prelude as dp


dp.enable_features("contrib")

def test_split_lazyframe():
    pl = pytest.importorskip("polars")
    context = dp.Context.compositor(
        data=pl.scan_csv(
            dp.examples.get_france_lfs_path(),
            ignore_errors=True,
        ),
        privacy_unit=dp.unit_of(contributions=36),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=5,
        margins=[
            dp.polars.Margin(max_length=150_000 * 36),
        ],
    )
    query_work_hours = (
        # 99 represents "Not applicable"
        context.query().filter(pl.col("HWUSUAL") != 99.0)
        # compute the DP sum
        .select(
            pl.col.HWUSUAL.cast(int)
            .fill_null(35)
            .dp.sum(bounds=(0, 80))
        )
    )
    breakpoint()