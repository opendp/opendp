from opendp.mod import enable_features
import opendp.prelude as dp

enable_features("floating-point", "contrib")


def test_quantile_score_candidates():

    input_domain = dp.vector_domain(dp.atom_domain(T=int))
    input_metric = dp.symmetric_distance()
    candidates = [20, 33, 40, 50, 72, 100]
    quant_trans = dp.t.make_quantile_score_candidates(input_domain, input_metric, candidates, alpha=0.5)

    assert quant_trans(list(range(100))) == [59, 33, 19, 1, 45, 100]

    expo_meas = dp.m.then_report_noisy_max_gumbel(1., "min")

    quantile_meas = quant_trans >> expo_meas
    idx = quantile_meas(list(range(100)))
    assert candidates[idx] == 50
    assert quantile_meas.map(1) == 2.0
