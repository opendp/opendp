from typing import List
import opendp.prelude as dp

dp.enable_features("contrib")


def test_analysis_init():
    analysis = dp.Analysis.sequential_composition(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=3),
        privacy_loss=dp.loss_of(epsilon=3.0),
        split_evenly_over=3,
        domain=dp.domain_of(List[int]),
    )

    dp_sum = analysis.query().clamp((1, 10)).sum().laplace(100.0)
    print(dp_sum.release())

    # this time the scale parameter is omitted, but it is resolved from the analysis
    print(analysis.query().clamp((1, 10)).sum().laplace().release())
    # where we're headed:
    # print(analysis.query().dp_sum((1, 10)).release())


def test_analysis_zCDP():
    analysis = dp.Analysis.sequential_composition(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=3.0, delta=1e-6),
        split_evenly_over=2,
    )

    dp_sum = (
        analysis.query()
        .clamp((1, 10))
        .zCDP_to_approxDP(lambda x: x.sum().gaussian(100.0))
    )
    print(dp_sum.release())

    dp_sum = (
        analysis.query().clamp((1, 10)).zCDP_to_approxDP(lambda x: x.sum().gaussian())
    )
    print(dp_sum.release())


def test_sc_query():
    analysis = dp.Analysis.sequential_composition(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=3.0, delta=1e-6),
        split_evenly_over=2,
        domain=dp.vector_domain(dp.atom_domain(T=int)),
    )

    # build a child sequential compositor, and then use it to release a laplace sum
    sub_analysis = analysis.query().sequential_composition(split_evenly_over=3).release()
    dp_sum = (
        sub_analysis.query()
        .clamp((1, 10))
        .pureDP_to_fixed_approxDP(lambda q: q.sum().laplace())
    )
    print("laplace dp_sum", dp_sum.release())

    # build a child sequential compositor in zCDP, and then use it to release some gaussian queries
    sub_analysis = analysis.query().zCDP_to_approxDP(
        lambda q: q.sequential_composition(split_evenly_over=2)
    ).release()
    dp_sum = sub_analysis.query().clamp((1, 10)).sum().gaussian()
    # with partials, fusing, and measure convention, would shorten to
    # dp_sum = sub_analysis.query().dp_sum((1, 10))
    print("gaussian dp_sum", dp_sum.release())

    dp_mean = (
        sub_analysis.query()
        .cast_default(float)
        .clamp((1.0, 10.0))
        .resize(3, constant=5.0)
        .mean()
        .gaussian()
    )
    # with partials, fusing, and measure convention, would shorten to
    # dp_mean = sub_analysis.query().cast(float).dp_mean((1., 10.))
    print("gaussian dp_mean", dp_mean.release())


def test_distance_of():
    assert dp.unit_of(contributions=3) == (dp.symmetric_distance(), 3)
    assert dp.unit_of(l1=2.0) == (dp.l1_distance(T=float), 2.0)


def test_privacy_loss_of():
    assert dp.loss_of(epsilon=3.0) == (dp.max_divergence(T=float), 3.0)
    assert dp.loss_of(rho=2.0) == (dp.zero_concentrated_divergence(T=float), 2.0)
    assert dp.loss_of(epsilon=2.0, delta=1e-6) == (
        dp.fixed_smoothed_max_divergence(T=float),
        (2.0, 1e-6),
    )
test_privacy_loss_of()