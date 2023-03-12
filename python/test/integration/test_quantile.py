from opendp.mod import enable_features

enable_features("floating-point", "contrib")


def test_quantile_score_candidates():
    from opendp.transformations import make_quantile_score_candidates
    from opendp.measurements import make_base_discrete_exponential

    candidates = [20, 33, 40, 50, 72, 100]
    quant_trans = make_quantile_score_candidates(candidates, alpha=0.5)

    print(quant_trans(list(range(100))))

    expo_meas = make_base_discrete_exponential(1000., "min", "usize")

    quantile_meas = quant_trans >> expo_meas
    idx = quantile_meas(list(range(100)))
    print(candidates[idx])

    assert quantile_meas.map(1) >= 0.1
