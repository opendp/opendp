import itertools as it
import functools as ft

import opendp.prelude as dp
from opendp.mod import Domain, Metric, Transformation
from opendp._lib import import_optional_dependency
from opendp._extrinsics._utilities import with_privacy

dp.enable_features("contrib", "floating-point")

np = import_optional_dependency("numpy")
pl = import_optional_dependency("polars")


def test_microdataa_faithfulness():

    from opendp._extrinsics.microdata.universal_microdata_scheme import make_private_universal_microdata_scheme

    COLS = list("ABC")
    NUM_RECORDS = 5

    real_df = pl.LazyFrame({
        "A": [0, 1, 0, 1, 0],
        "B": [1, 0, 1, 0, 1],
        "C": [0, 0, 1, 1, 1]
    })

    lf_domain = dp.lazyframe_domain(
        [dp.series_domain(col, dp.atom_domain(T=dp.i32))
         for col in COLS])

    lf_domain_with_margin = dp.with_margin(lf_domain,
                                           by=COLS, public_info="keys")

    def similarity_record_fn(x, y):
        return np.sum(np.abs((x - y)))

    ums_meas = make_private_universal_microdata_scheme(
        input_domain=lf_domain_with_margin,
        input_metric=dp.symmetric_distance(),
        synthesizer_epsilon=1.,
        configuration_candidates=[{"synthesizer": {"name": "identity"}}],
        quality_acceptance_criteria=[(_make_private_max_marginal_abs_error, 0.05 * NUM_RECORDS),
                                     (ft.partial(_make_private_max_univariate_rel_error, clipping_upper_bound=3), 2)],
        quality_epsilons=[1., 1.],
        faithfulness_simiarlity=similarity_record_fn,
        faithfulness_threshold=0.05,
        faithfulness_epsilon=1.,
        with_selection=True,
        selection_stop_probability=0,
        selection_epsilon=0,
        selection_steps=None
        )

    assert ums_meas.map(1) == 8

    score, output = ums_meas(real_df)

    assert score
    assert len(output) == 4


def _make_max_marginal_abs_error(
        input_domain: Domain,
        input_metric: Metric,
        *,
        reference_dataset: pl.LazyFrame) -> Transformation:
    """Construct a Transformation that returns the maximal absoulte error over all k-way marginals."""

    dp.assert_features("contrib", "floating-point")

    if "LazyFrameDomain" not in str(input_domain.type):
        raise ValueError("input_domain must be a LazyFrame domain")

    # TODO: assert margin with public_info "keys"
    # TODO: extract cols (by) from the magin
    cols = reference_dataset.columns

    # assert input_domain.member(reference_dataset.lazy())

    if input_metric != dp.symmetric_distance():
        raise ValueError("input metric must be symmetric distance")

    def function(dataset):

        abs_diffs = []
        for k_way in range(1, len(cols) + 1):
            for subset in it.combinations(cols, k_way):

                merged_df = (dataset.group_by(subset).len().collect().join(
                    reference_dataset.group_by(subset).len(),
                    how="outer",
                    on=subset,)
                    .select([
                        pl.col("len").alias("len_dataset").cast(pl.Int32),
                        pl.col("len_right").alias("len_reference_dataset").cast(pl.Int32)
                    ])
                    .fill_null(0)
                )

                abs_diffs.append((merged_df["len_dataset"] - merged_df["len_reference_dataset"])
                                 .abs()
                                 .to_numpy())

        return np.concatenate(abs_diffs).max()

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.atom_domain(T=int),
        dp.absolute_distance(T=int),
        function,
        lambda d_in: d_in,
    )


def _make_max_univariate_rel_error(
        input_domain: Domain,
        input_metric: Metric,
        *,
        reference_dataset: pl.LazyFrame,
        clipping_upper_bound: float) -> Transformation:
    """Construct a Transformation that returns the maximal relative error over all 1-way marginals."""

    dp.assert_features("contrib", "floating-point")

    if "LazyFrameDomain" not in str(input_domain.type):
        raise ValueError("input_domain must be a LazyFrame domain")

    # TODO: assert margin with public_info "keys"
    # TODO: extract cols (by) from the magin
    cols = reference_dataset.columns

    # assert input_domain.member(reference_dataset.lazy())

    if input_metric != dp.symmetric_distance():
        raise ValueError("input metric must be symmetric distance")

    def function(dataset):

        ratios = []
        for col in cols:

            merged_df = (dataset.group_by(col).len().collect().join(
                reference_dataset.group_by(col).len(),
                how="outer",
                on=col,)
                .select([
                    pl.col("len").alias("len_dataset").cast(pl.Int32),
                    pl.col("len_right").alias("len_reference_dataset").cast(pl.Int32)
                ])
                .fill_null(0)
            )

            len_dataset_plus_one = merged_df["len_dataset"] + 1
            len_reference_dataset_plus_one = merged_df["len_reference_dataset"] + 1

            ratios.append(len_dataset_plus_one / len_reference_dataset_plus_one)
            ratios.append(len_reference_dataset_plus_one / len_dataset_plus_one)

        clipped_ratios = np.clip(np.concatenate(ratios), 1, clipping_upper_bound)

        # TODO: seperate with max composition
        return np.max(clipped_ratios)

    # Sensitiivy https://arxiv.org/pdf/2405.00267#page=24
    reference_min_count = min([reference_dataset.group_by(col).len()["len"].min() for col in cols])
    reference_min_count_plus_one = reference_min_count + 1
    sensitiivy_factor = max(1 / reference_min_count_plus_one,
                            clipping_upper_bound - 1 / (1 / clipping_upper_bound + 1 / reference_min_count_plus_one))

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.atom_domain(T=float),
        dp.absolute_distance(T=float),
        function,
        lambda d_in: d_in * sensitiivy_factor,
    )


_make_private_max_marginal_abs_error = with_privacy(_make_max_marginal_abs_error)
_make_private_max_univariate_rel_error = with_privacy(_make_max_univariate_rel_error)
