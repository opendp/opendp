'''
The tests here do the bare minimum so the generated API has 100% test coverage:
If new methods are added to the API, and we don't have better tests elsewhere,
we should add a minimal test here as a reminder, although
the real tests for the FFI should be on the Rust side in many cases.
'''

import opendp.prelude as dp

def test_barely():
    dp.accuracy_to_discrete_gaussian_scale(1.0, 0.5)
    dp.accuracy_to_discrete_laplacian_scale(1.0, 0.5)
    dp.discrete_gaussian_scale_to_accuracy(1.0, 0.5)

    dp.smoothed_max_divergence(float)

    dp.m.then_private_expr(None, None) # type: ignore[arg-type]
    dp.m.then_private_lazyframe(None, None) # type: ignore[arg-type]

    dp.t.then_is_equal(None)
    dp.t.then_metric_bounded()
    dp.t.then_ordered_random()
    dp.t.then_quantile_score_candidates([], 0.5)
    dp.t.then_sum_of_squared_deviations()