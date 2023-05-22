from opendp.mod import Queryable
import opendp.prelude as dp
import pytest

dp.enable_features("contrib")


def test_sequential_composition():
    max_influence = 1
    sc_meas = dp.c.make_sequential_composition(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
        d_in=max_influence,
        d_mids=[0.2, 0.3, 0.4, 0.7],
    )

    sc_qbl: dp.Queryable = sc_meas([1] * 200)

    print("SeqComp IM:", sc_qbl)
    sum_query = (
        sc_meas.input_space
        >> dp.t.then_clamp((0, 10))
        >> dp.t.then_sum()
        >> dp.m.then_laplace(100.0)
    )

    print("evaluating")
    print(sc_qbl(sum_query))

    exact_sum = (
        sc_meas.input_space
        >> dp.t.then_clamp((0, 10))
        >> dp.t.then_sum()
    )
    print("exact sum:", exact_sum)
    exact_sum_sc_qbl = sc_qbl(
        exact_sum
        >> dp.c.make_sequential_composition(
            input_domain=exact_sum.output_domain,
            input_metric=exact_sum.output_metric,
            output_measure=dp.max_divergence(),
            d_in=exact_sum.map(max_influence),
            d_mids=[0.2, 0.09],
        )
    )
    noise_query = exact_sum.output_space >> dp.m.then_laplace(200.0)

    print("child release:", exact_sum_sc_qbl(noise_query))
    print("child release:", exact_sum_sc_qbl(noise_query))
    print("root release: ", sc_qbl(sum_query))


def test_sequential_composition_approxdp():
    max_influence = 1
    sc_meas = dp.c.make_sequential_composition(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.fixed_smoothed_max_divergence(),
        d_in=max_influence,
        d_mids=[(1.0, 1e-6), (1.0, 1e-6)],
    )

    sc_qbl: dp.Queryable = sc_meas([1] * 200)

    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()

    sum_meas = (
        input_space
        >> dp.t.then_clamp((0, 10))
        >> dp.t.then_sum()
        >> dp.m.then_gaussian(100.0)
    )
    sum_meas = dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(sum_meas), 1e-6)
    sc_qbl(sum_meas)


def test_plugin_queryable_int():
    def transition(query):
        assert query == 2
        return query + 1
    qbl = dp.new_queryable(transition, int, int)
    assert qbl(2) == 3


def test_plugin_queryable_list():
    def transition(query, _is_internal):
        assert query == [2, 3]
        return query[-1]
    qbl = dp.new_queryable(transition, "Vec<i32>", int)
    assert qbl([2, 3]) == 3


def test_plugin_queryable_error():
    def transition(_query, _is_internal):
        raise ValueError("test clean stack trace")
    qbl = dp.new_queryable(transition, "Vec<i32>", int)

    with pytest.raises(dp.OpenDPException):
        qbl([2, 3])

    with pytest.raises(TypeError):
        qbl(2)

def test_sequential_odometer():
    dp.enable_features("rust-stack-trace")
    max_influence = 1
    sc_odo = dp.c.make_sequential_odometer(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
    )

    assert str(sc_odo) == """Odometer(
    input_domain   = VectorDomain(AtomDomain(T=i32)),
    input_metric   = SymmetricDistance(),
    output_measure = MaxDivergence())"""

    sc_odo.invoke([1] * 200)
    sc_qbl: dp.Queryable = sc_odo([1] * 200)

    print("SeqComp IM:", sc_qbl)
    sum_query = sc_odo.input_space >> dp.t.then_clamp((0, 10)) >> dp.t.then_sum() >> dp.m.then_laplace(100.)

    print("evaluating")
    print(sc_qbl(sum_query))

    sc_qbl.eval(sc_odo)

    noise_query = dp.m.make_laplace(dp.atom_domain(T=int), dp.absolute_distance(T=int), 200.)
    exact_sum = sc_odo.input_space >> dp.t.then_clamp((0, 10)) >> dp.t.then_sum()
    print("exact sum:", exact_sum)
    exact_sum_sc_qbl = sc_qbl(exact_sum >> dp.c.make_sequential_composition(
        input_domain=exact_sum.output_domain,
        input_metric=exact_sum.output_metric,
        output_measure=dp.max_divergence(),
        d_in=exact_sum.map(max_influence),
        d_mids=[0.2, 0.09]
    ))

    print("child release:", exact_sum_sc_qbl(noise_query))
    print("child release:", exact_sum_sc_qbl(noise_query))
    print("root release: ", sc_qbl(sum_query))

    print("privacy usage", sc_qbl.map(1))


def test_odometer_supporting_elements():
    sc_odo = dp.c.make_sequential_odometer(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
    )

    assert sc_odo.input_domain == dp.vector_domain(dp.atom_domain(T=int))
    assert sc_odo.input_metric == dp.symmetric_distance()
    assert sc_odo.output_measure == dp.max_divergence()
    assert sc_odo.input_space == (dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance())
    assert sc_odo.input_distance_type == dp.u32
    assert sc_odo.output_distance_type == dp.f64
    assert sc_odo.input_carrier_type == dp.Vec[dp.i32]

    
def test_odometer_chain_ot():
    sc_odo = dp.c.make_sequential_odometer(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(float),
        Q=dp.Measurement
    )

    sc_odo2 = dp.t.make_cast_default(TIA=str, TOA=int) >> sc_odo

    sc_qbl2: Queryable = sc_odo2(["1"] * 200)


    print("SeqComp IM:", sc_qbl2)
    sum_query = dp.t.make_clamp((0, 10)) >> dp.t.make_bounded_sum((0, 10)) >> dp.m.make_base_discrete_laplace(100.)

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

    sc_qbl2: Queryable = sc_odo2([1] * 200)

    print("SeqComp IM:", sc_qbl2)
    sum_query = dp.t.make_clamp((0, 10)) >> dp.t.make_bounded_sum((0, 10)) >> dp.m.make_base_discrete_laplace(100.)

    print("evaluating")
    print(sc_qbl2(sum_query))

    print("privacy usage", sc_qbl2.map(1))

