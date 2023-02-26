
from opendp.mod import Queryable, enable_features
enable_features("contrib")

from opendp.transformations import *
from opendp.measurements import make_base_discrete_laplace
from opendp.combinators import make_concurrent_composition

from opendp.domains import *
from opendp.metrics import symmetric_distance
from opendp.measures import max_divergence

def test_concurrent_composition():
    max_influence = 1
    cc_meas = make_concurrent_composition(
        input_domain=vector_domain(atom_domain(T=int)),
        input_metric=symmetric_distance(),
        output_measure=max_divergence(float),
        d_in=max_influence,
        d_mids=[0.2, 0.3, 0.4, 0.7]
    )

    cc_qbl: Queryable = cc_meas([1] * 200)

    print("ConComp IM:", cc_qbl)
    sum_query = make_clamp((0, 10)) >> make_bounded_sum((0, 10)) >> make_base_discrete_laplace(100.)

    print("evaluating")
    print(cc_qbl(sum_query))

    noise_query = make_base_discrete_laplace(200.)
    exact_sum = make_clamp((0, 10)) >> make_bounded_sum((0, 10))
    print("exact sum:", exact_sum)
    exact_sum_cc_qbl = cc_qbl(exact_sum >> make_concurrent_composition(
        input_domain=exact_sum.output_domain,
        input_metric=exact_sum.output_metric,
        output_measure=max_divergence(float),
        d_in=exact_sum.map(max_influence),
        d_mids=[0.2, 0.09]
    ))

    print("child release:", exact_sum_cc_qbl(noise_query))
    print("root release: ", cc_qbl(sum_query))
    print("child release:", exact_sum_cc_qbl(noise_query))
