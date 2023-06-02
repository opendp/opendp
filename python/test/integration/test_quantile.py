from opendp.mod import enable_features

enable_features("floating-point", "contrib")


def test_quantile_score_candidates():
    from opendp.transformations import make_quantile_score_candidates
    from opendp.measurements import part_base_discrete_exponential
    from opendp.domains import vector_domain, atom_domain
    from opendp.metrics import symmetric_distance

    input_domain = vector_domain(atom_domain(T=int))
    input_metric = symmetric_distance()
    candidates = [20, 33, 40, 50, 72, 100]
    quant_trans = make_quantile_score_candidates(input_domain, input_metric, candidates, alpha=0.5)

    print(quant_trans(list(range(100))))

    expo_meas = part_base_discrete_exponential(1000., "min")

    quantile_meas = quant_trans >> expo_meas
    idx = quantile_meas(list(range(100)))
    print(candidates[idx])

    assert quantile_meas.map(1) >= 0.1
