import opendp.prelude as dp
import pytest



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
