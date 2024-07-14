import pytest

import opendp.prelude as dp

from ..helpers import optional_dependency

dp.enable_features("contrib", "floating-point")


def test_microdataa_faithfulness():
    from opendp._extrinsics.microdata.faithfulness import make_faithfulness

    # TODO: add optional_dependency("igraph")
    with optional_dependency("numpy"), optional_dependency("scipy"), optional_dependency("sklearn"):

        pl = pytest.importorskip("polars")
        # private
        dataset = pl.LazyFrame({"A": pl.Series([100, 1, 300], dtype=pl.Int32),
                                "B": pl.Series([4, 4, 6], dtype=pl.Int32)})

        # public
        reference_dataset = pl.DataFrame({"A": pl.Series([1, 2, 3], dtype=pl.Int32),
                                          "B": pl.Series([4, 5, 6], dtype=pl.Int32)})

        lf_domain = dp.lazyframe_domain(
            [dp.series_domain("A", dp.option_domain(dp.atom_domain(T=dp.i32))),
             dp.series_domain("B", dp.atom_domain(T=dp.i32))
             ])

        # exact match
        def similarity_record_fn(x, y):
            return 2*(x != y).any()

        trans1 = make_faithfulness(lf_domain,
                                   dp.symmetric_distance(),
                                   reference_dataset=reference_dataset,
                                   similarity=similarity_record_fn)

        assert trans1.map(1) == 1
        assert trans1(dataset) == 1

        trans2 = make_faithfulness(lf_domain,
                                   dp.symmetric_distance(),
                                   reference_dataset=reference_dataset,
                                   similarity={"A": 1/100, "B": 1/2})

        assert trans2.map(1) == 1
        assert trans2(dataset) == 2
