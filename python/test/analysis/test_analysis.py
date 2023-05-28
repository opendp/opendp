import opendp.prelude as dp
from opendp.analysis import Analysis, distance_of, privacy_of

dp.enable_features("contrib")


def test_analysis_init():
    analysis = Analysis.sequential_composition(
        data=[1, 2, 3],
        privacy_unit=distance_of(contributions=3),
        privacy_loss=privacy_of(epsilon=3.0),
        weights=3,
        domain=dp.vector_domain(dp.atom_domain(T=int)),
    )

    dp_sum = (
        analysis.query()
        .clamp((1, 10))
        .bounded_sum((1, 10))
        .base_discrete_laplace(100.0)
    )
    print(dp_sum)

    # this time the scale parameter is omitted, but it is resolved from the analysis
    print(analysis.query().clamp((1, 10)).bounded_sum((1, 10)).base_discrete_laplace())
    # where we're headed:
    # print(analysis.query().clamp((1, 10)).sum().laplace())
    # print(analysis.query().dp_sum((1, 10)))


def test_analysis_zCDP():
    analysis = Analysis.sequential_composition(
        data=[1, 2, 3],
        privacy_unit=distance_of(contributions=1),
        privacy_loss=privacy_of(epsilon=3.0, delta=1e-6),
        weights=2,
        domain=dp.vector_domain(dp.atom_domain(T=int)),
    )

    dp_sum = (
        analysis.query()
        .clamp((1, 10))
        .zCDP_to_approxDP(
            lambda x: x.bounded_sum((1, 10)).base_discrete_gaussian(100.0)
        )
    )
    print(dp_sum)

    dp_sum = (
        analysis.query()
        .clamp((1, 10))
        .zCDP_to_approxDP(lambda x: x.bounded_sum((1, 10)).base_discrete_gaussian())
    )
    print(dp_sum)


def test_sc_query():
    analysis = Analysis.sequential_composition(
        data=[1, 2, 3],
        privacy_unit=distance_of(contributions=1),
        privacy_loss=privacy_of(epsilon=3.0, delta=1e-6),
        weights=2,
        domain=dp.vector_domain(dp.atom_domain(T=int)),
    )

    # build a child sequential compositor, and then use it to release a laplace sum
    sub_analysis = analysis.query().sequential_composition(weights=3)
    dp_sum = sub_analysis.query().clamp((1, 10)).pureDP_to_fixed_approxDP(
        lambda q: q.bounded_sum((1, 10)).base_discrete_laplace()
    )
    print("laplace dp_sum", dp_sum)

    # build a child sequential compositor in zCDP, and then use it to release some gaussian queries
    sub_analysis = (
        analysis.query()
        .zCDP_to_approxDP(lambda q: q.sequential_composition(weights=2))
    )
    dp_sum = (
        sub_analysis.query()
        .clamp((1, 10))
        .bounded_sum((1, 10))
        .base_discrete_gaussian()
    )
    # with partials, fusing, and measure convention, would shorten to 
    # dp_sum = sub_analysis.query().dp_sum((1, 10))
    print("gaussian dp_sum", dp_sum)

    dp_mean = (
        sub_analysis.query()
        .cast_default(int, float)
        .clamp((1., 10.))
        .resize(3, dp.atom_domain((1., 10.)), constant=5.)
        .sized_bounded_mean(3, (1., 10.))
        .base_gaussian()
    )
    # with partials, fusing, and measure convention, would shorten to 
    # dp_mean = sub_analysis.query().cast(float).dp_mean((1., 10.))
    print("gaussian dp_mean", dp_mean)


def test_distance_of():
    assert distance_of(contributions=3) == (dp.symmetric_distance(), 3)
    assert distance_of(l1=2.0) == (dp.l1_distance(T=float), 2.0)


def test_privacy_loss_of():
    assert privacy_of(epsilon=3.0) == (dp.max_divergence(T=float), 3.0)
    assert privacy_of(rho=2.0) == (dp.zero_concentrated_divergence(T=float), 2.0)
    assert privacy_of(epsilon=2.0, delta=1e-6) == (
        dp.fixed_smoothed_max_divergence(T=float),
        (2.0, 1e-6),
    )
