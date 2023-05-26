import opendp.prelude as dp
from opendp.analysis import Analysis

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

    