import pytest

import opendp.prelude as dp
from opendp.extras.synth.mwem import Schema, MWEMSynthesizerTrainer


dp.enable_features("contrib", "floating-point", "honest-but-curious")


def test_mwem_learning():

    num_records = 1000
    cols = list("abcd")
    lower, upper = 1, 9

    np = pytest.importorskip("numpy")
    pl = pytest.importorskip("polars")
    make_blobs = pytest.importorskip("sklearn.datasets.make_blobs")

    data_array = (make_blobs(num_records, 4, random_state=42)[0]
                  .round()
                  .clip(lower, upper)
                  .astype(np.int32))
    real_df = pl.LazyFrame(data_array, schema={col: pl.Int32 for col in cols})

    lf_domain = dp.lazyframe_domain(
        [dp.series_domain(col, dp.atom_domain(T=dp.i32, bounds=(lower, upper)))
         for col in cols])

    lf_domain_with_margin = dp.with_margin(lf_domain,
                                           by=cols,
                                           public_info="keys")

    real_schema = Schema(bounds={col: (lower, upper) for col in cols},
                         size=num_records)

    mwem_meas = MWEMSynthesizerTrainer.make(lf_domain_with_margin,
                                            dp.symmetric_distance(),
                                            epsilon=10,
                                            schema=real_schema,
                                            epsilon_split=0.5,
                                            num_queries=1000,
                                            num_iterations=10,
                                            num_mult_weights_iterations=25,
                                            verbose=False)

    mwem_synth = mwem_meas(real_df)

    synth_df = mwem_synth.sample(num_records)

    max_error_in_mean = np.max(np.abs(real_df.mean().collect() - synth_df.mean()))

    assert max_error_in_mean < 2
