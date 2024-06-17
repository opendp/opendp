import opendp.prelude as dp


def test_number_of_tests_found(request):
    tests_found = len(request.session.items)
    assert tests_found >= 304


def test_barely():
    '''
    The tests here do the bare minimum so the generated API has 100% test coverage:
    If new methods are added to the API, and we don't have better tests elsewhere,
    we should add a minimal test here as a reminder, although
    the real tests for the FFI may be on the Rust side in many cases.
    '''
    assert dp.accuracy_to_discrete_gaussian_scale(1.0, 0.5) == 0.797878994872694 # TODO: Closed form expression
    assert dp.accuracy_to_discrete_laplacian_scale(1.0, 0.5) == 0.9102392266268373 # TODO: Closed form expression
    assert dp.discrete_gaussian_scale_to_accuracy(1.0, 0.5) == 2.0

    # TODO: Add meaningful tests of measures:
    assert str(dp.smoothed_max_divergence(float)) == 'SmoothedMaxDivergence(f64)' # TODO: Add a test that does something with SmoothedMaxDivergence 

    # TODO: Add meaningful tests of partial constructors:
    dp.m.then_private_expr(None, None) == 0 # type: ignore[arg-type]
    dp.m.then_private_lazyframe(None, None) == 0 # type: ignore[arg-type]
    dp.t.then_is_equal(None)
    dp.t.then_metric_bounded()
    dp.t.then_ordered_random()
    dp.t.then_quantile_score_candidates([], 0.5)
    dp.t.then_sum_of_squared_deviations()