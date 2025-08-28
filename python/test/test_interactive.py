import opendp.prelude as dp
import pytest



def test_sequential_composition():
    max_influence = 1
    sc_meas = dp.c.make_adaptive_composition(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
        d_in=max_influence,
        d_mids=[0.2, 0.3, 0.4, 0.7],
    )

    sc_qbl: dp.Queryable = sc_meas([1] * 200)

    print("sc_qbl", sc_qbl)
    sum_query = (
        sc_meas.input_space
        >> dp.t.then_clamp((0, 10))
        >> dp.t.then_sum()
        >> dp.m.then_laplace(100.0)
    )

    print("sc_qbl(sum_query)", sc_qbl(sum_query))

    exact_sum = (
        sc_meas.input_space
        >> dp.t.then_clamp((0, 10))
        >> dp.t.then_sum()
    )
    print("exact sum:", exact_sum)
    exact_sum_sc_qbl = sc_qbl(
        exact_sum
        >> dp.c.make_adaptive_composition(
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
    sc_meas = dp.c.make_adaptive_composition(
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


def test_fully_adaptive_composition():
    max_influence = 1
    space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    o_comp = space >> dp.c.then_fully_adaptive_composition(dp.max_divergence())
    assert space == o_comp.input_space

    assert str(o_comp) == """Odometer(
    input_domain   = VectorDomain(AtomDomain(T=i32)),
    input_metric   = SymmetricDistance(),
    output_measure = MaxDivergence)"""

    qbl_comp: dp.OdometerQueryable = o_comp([1] * 200)
    assert qbl_comp.privacy_loss(max_influence) == 0.0

    assert str(qbl_comp) == "OdometerQueryable(Q=AnyMeasurement, QB=u32)"
    m_sum = space >> dp.t.then_clamp((0, 10)) >> dp.t.then_sum() >> dp.m.then_laplace(100.)

    # evaluating
    assert isinstance(qbl_comp(m_sum), int)
    assert qbl_comp.privacy_loss(max_influence) == m_sum.map(max_influence)

    m_lap = dp.m.make_laplace(dp.atom_domain(T=int), dp.absolute_distance(T=int), 200.)
    t_sum = space >> dp.t.then_clamp((0, 10)) >> dp.t.then_sum()
    m_sum_compositor = t_sum >> dp.c.then_adaptive_composition(
        output_measure=dp.max_divergence(),
        d_in=t_sum.map(max_influence),
        d_mids=[0.2, 0.09]
    )
    qbl_summed = qbl_comp.invoke(m_sum_compositor)
    # it's slightly larger, checking greater than will do
    assert qbl_comp.privacy_loss(max_influence) > m_sum.map(max_influence) + 0.2 + 0.09

    assert isinstance(qbl_summed(m_lap), int) # child release
    assert isinstance(qbl_summed(m_lap), int) # child release
    assert isinstance(qbl_comp(m_sum), int) # root release

    # it's slightly larger, checking greater than will do
    assert qbl_comp.privacy_loss(1) > m_sum.map(max_influence) * 2 + 0.2 + 0.09


def test_odometer_supporting_elements():
    o_ac = dp.c.make_fully_adaptive_composition(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(),
    )

    assert isinstance(o_ac.invoke([]), dp.OdometerQueryable)
    assert o_ac.input_domain == dp.vector_domain(dp.atom_domain(T=int))
    assert o_ac.input_metric == dp.symmetric_distance()
    assert o_ac.output_measure == dp.max_divergence()
    assert o_ac.input_space == (dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance())
    assert o_ac.input_distance_type == dp.u32
    assert o_ac.output_distance_type == dp.f64
    assert o_ac.input_carrier_type == dp.Vec[dp.i32]


def test_privacy_filter():
    m_filter = dp.c.make_privacy_filter(
        dp.c.make_fully_adaptive_composition(
            input_domain=dp.vector_domain(dp.atom_domain(T=int)),
            input_metric=dp.symmetric_distance(),
            output_measure=dp.max_divergence(),
        ),
        d_in=1,
        d_out=2.0
    )

    qbl_filter = m_filter([1, 2, 3, 4, 5])

    m_count = dp.t.make_count(*m_filter.input_space) >> dp.m.then_laplace(1.0)

    qbl_filter(m_count)
    assert qbl_filter.privacy_loss(1) == 1.0
    qbl_filter(m_count)
    assert qbl_filter.privacy_loss(1) == 2.0

