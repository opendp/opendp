import opendp.prelude as dp
from opendp.analysis import Analysis, distance_of, privacy_loss_of

dp.enable_features("contrib")


def test_analysis_init():
    analysis = Analysis(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(float),
        data=[1, 2, 3],
        d_in=1,
        d_mids=[0.2, 0.3, 0.4, 0.7],
    )

    dp_sum = (
        analysis.query()
        .clamp((1, 10))
        .bounded_sum((1, 10))
        .base_discrete_laplace(100.0)
    )
    print(dp_sum)

    # this time the scale parameter is omitted, but it is resolved from the analysis
    print(
        analysis.query()
        .clamp((1, 10))
        .bounded_sum((1, 10))
        .base_discrete_laplace()
    )
    # where we're headed:
    # print(analysis.query().clamp((1, 10)).sum().laplace())
    # print(analysis.query().dp_sum((1, 10)))


def test_distance_of():
    assert distance_of(contributions=3) == (dp.symmetric_distance(), 3)
    assert distance_of(l1=2.) == (dp.l1_distance(T=float), 2.)

def test_privacy_loss_of():
    assert privacy_loss_of(epsilon=3.) == (dp.max_divergence(T=float), 3.)
    assert privacy_loss_of(rho=2.) == (dp.zero_concentrated_divergence(T=float), 2.)
    assert privacy_loss_of(epsilon=2., delta=1e-6) == (dp.fixed_smoothed_max_divergence(T=float), (2., 1e-6))
