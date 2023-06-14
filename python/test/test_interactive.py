import opendp.prelude as dp

dp.enable_features("contrib")


def test_sequential_composition():
    max_influence = 1
    sc_meas = dp.c.make_sequential_composition(
        input_domain=dp.vector_domain(dp.atom_domain(T=int)),
        input_metric=dp.symmetric_distance(),
        output_measure=dp.max_divergence(float),
        d_in=max_influence,
        d_mids=[0.2, 0.3, 0.4, 0.7],
    )

    sc_qbl: dp.Queryable = sc_meas([1] * 200)

    print("SeqComp IM:", sc_qbl)
    sum_query = (
        sc_meas.input_space
        >> dp.t.then_clamp((0, 10))
        >> dp.t.then_sum()
        >> dp.m.then_base_discrete_laplace(100.0)
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
            output_measure=dp.max_divergence(float),
            d_in=exact_sum.map(max_influence),
            d_mids=[0.2, 0.09],
        )
    )
    noise_query = exact_sum.output_space >> dp.m.then_base_discrete_laplace(200.0)

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
        d_mids=[(1.0, 1e-6), (1.0, 1e-6)],
    )

    sc_qbl: dp.Queryable = sc_meas([1] * 200)

    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()

    sum_meas = (
        input_space
        >> dp.t.then_clamp((0, 10))
        >> dp.t.then_sum()
        >> dp.m.then_base_discrete_gaussian(100.0)
    )
    sum_meas = dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(sum_meas), 1e-6)
    sc_qbl(sum_meas)
