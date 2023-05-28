import opendp.prelude as dp
from opendp.analysis import Analysis, distance_of, privacy_of

dp.enable_features("contrib")


def test_analysis_init():
    analysis = Analysis.sequential_composition(
        data=[1, 2, 3],
        unit_of_privacy=distance_of(contributions=3),
        privacy_budget=privacy_of(epsilon=3.0),
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
        unit_of_privacy=distance_of(contributions=1),
        privacy_budget=privacy_of(epsilon=3.0, delta=1e-6),
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
        .zCDP_to_approxDP(
            lambda x: x.bounded_sum((1, 10)).base_discrete_gaussian()
        )
    )
    print(dp_sum)


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
