
from opendp.mod import Queryable, enable_features
enable_features("contrib")

from opendp.transformations import *
from opendp.measurements import make_base_discrete_laplace
from opendp.combinators import make_concurrent_composition

from opendp.domains import *
from opendp.metrics import symmetric_distance, absolute_distance
from opendp.measures import max_divergence

def test_concurrent_composition():
    cc_meas = make_concurrent_composition(
        input_domain=vector_domain(all_domain(int)),
        input_metric=symmetric_distance(),
        output_measure=max_divergence(float),
        d_in=1,
        d_mids=[0.2, 0.3, 0.4, 0.7]
    )

    cc_qbl: Queryable = cc_meas([1] * 200)

    print(cc_qbl)
    sum_query = make_clamp((0, 10)) >> make_bounded_sum((0, 10)) >> make_base_discrete_laplace(100.)

    print("evaluating")
    print(cc_qbl(sum_query))

    exact_sum_cc_qbl = cc_qbl(make_clamp((0, 10)) >> make_bounded_sum((0, 10)) >> make_concurrent_composition(
        input_domain=all_domain(int),
        input_metric=absolute_distance(int),
        output_measure=max_divergence(float),
        d_in=10,
        d_mids=[0.2, 0.09]
    ))

    noise_query = make_base_discrete_laplace(200.)
    print(exact_sum_cc_qbl(noise_query))
    print(cc_qbl(sum_query))
    print(exact_sum_cc_qbl(noise_query))

