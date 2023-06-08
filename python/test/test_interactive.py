from typing import List
import opendp.prelude as dp
dp.enable_features("contrib")

def test_sequential_composition():
    max_influence = 1
    sc_meas = dp.c.make_sequential_composition(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(float),
        d_in=max_influence,
        d_mids=[0.2, 0.3, 0.4, 0.7]
    )

    sc_qbl: dp.Queryable = sc_meas([1] * 200)

    print("SeqComp IM:", sc_qbl)
    sum_query = sc_meas.input_space >> dp.t.part_clamp((0, 10)) >> dp.t.make_bounded_sum((0, 10)) >> dp.m.part_base_discrete_laplace(100.)

    print("evaluating")
    print(sc_qbl(sum_query))

    exact_sum = sc_meas.input_space >> dp.t.part_clamp((0, 10)) >> dp.t.make_bounded_sum((0, 10))
    print("exact sum:", exact_sum)
    exact_sum_sc_qbl = sc_qbl(exact_sum >> dp.c.make_sequential_composition(
        input_domain=exact_sum.output_domain,
        input_metric=exact_sum.output_metric,
        output_measure=dp.max_divergence(float),
        d_in=exact_sum.map(max_influence),
        d_mids=[0.2, 0.09]
    ))
    noise_query = exact_sum.output_space >> dp.m.part_base_discrete_laplace(200.)

    print("child release:", exact_sum_sc_qbl(noise_query))
    print("child release:", exact_sum_sc_qbl(noise_query))
    print("root release: ", sc_qbl(sum_query))


def test_sequential_composition_approxdp():
    max_influence = 1
    sc_meas = dp.c.make_sequential_composition(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.fixed_smoothed_max_divergence(float),
        d_in=max_influence,
        d_mids=[(1., 1e-6), (1., 1e-6)]
    )

    sc_qbl: dp.Queryable = sc_meas([1] * 200)

    gauss_meas = dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(dp.m.make_base_discrete_gaussian(100.)), 1e-6)

    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    sum_meas = input_space >> dp.t.part_clamp((0, 10)) >> dp.t.make_bounded_sum((0, 10)) >> gauss_meas
    sc_qbl(sum_meas)


def test_sequential_odometer():
    max_influence = 1
    sc_odo = dp.c.make_sequential_odometer(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(float),
        Q=dp.Measurement
    )

    sc_qbl: dp.Queryable = sc_odo([1] * 200)

    print("SeqComp IM:", sc_qbl)
    sum_query = sc_odo.input_space >> dp.t.part_clamp((0, 10)) >> dp.t.make_bounded_sum((0, 10)) >> dp.m.make_base_discrete_laplace(100.)

    print("evaluating")
    print(sc_qbl(sum_query))

    noise_query = dp.m.make_base_discrete_laplace(200.)
    exact_sum = sc_odo.input_space >> dp.t.part_clamp((0, 10)) >> dp.t.make_bounded_sum((0, 10))
    print("exact sum:", exact_sum)
    exact_sum_sc_qbl = sc_qbl(exact_sum >> dp.c.make_sequential_composition(
        input_domain=exact_sum.output_domain,
        input_metric=exact_sum.output_metric,
        output_measure=dp.max_divergence(float),
        d_in=exact_sum.map(max_influence),
        d_mids=[0.2, 0.09]
    ))

    print("child release:", exact_sum_sc_qbl(noise_query))
    print("child release:", exact_sum_sc_qbl(noise_query))
    print("root release: ", sc_qbl(sum_query))

    print("privacy usage", sc_qbl.map(1))


def test_odometer_chain_ot():
    sc_odo = dp.c.make_sequential_odometer(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(float),
        Q=dp.Measurement
    )

    sc_odo2 = dp.space_of(List[str]) >> dp.t.part_cast_default(TOA=int) >> sc_odo

    sc_qbl2: dp.Queryable = sc_odo2(["1"] * 200)


    print("SeqComp IM:", sc_qbl2)
    sum_query = sc_odo.input_space >> dp.t.part_clamp((0, 10)) >> dp.t.make_bounded_sum((0, 10)) >> dp.m.make_base_discrete_laplace(100.)

    print("evaluating")
    print(sc_qbl2(sum_query))

    print("privacy usage", sc_qbl2.map(1))



def test_odometer_chain_po():
    sc_odo = dp.c.make_sequential_odometer(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(float),
        Q=dp.Measurement
    )

    sc_odo2 = sc_odo >> dp.c.make_user_postprocessor(lambda x: str(x + 10_000), str)

    sc_qbl2: dp.Queryable = sc_odo2([1] * 200)

    print("SeqComp IM:", sc_qbl2)
    sum_query = sc_odo.input_space >> dp.t.part_clamp((0, 10)) >> dp.t.make_bounded_sum((0, 10)) >> dp.m.make_base_discrete_laplace(100.)

    print("evaluating")
    print(sc_qbl2(sum_query))

    print("privacy usage", sc_qbl2.map(1))

