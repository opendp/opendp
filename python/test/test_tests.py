import opendp.prelude as dp


def test_number_of_tests_found(request):
    tests_found = len(request.session.items)
    assert tests_found >= 304


def test_TODOs():
    '''
    The tests here do the bare minimum so the generated API has 100% test coverage:
    At least we know the functions run.
    If new methods are added to the API, and we don't have better tests elsewhere,
    we should add a minimal test hxere as a reminder, although
    the real tests for the FFI may be on the Rust side in many cases.
    '''
    # TODO: Add meaningful tests of measures:
    assert str(dp.smoothed_max_divergence()) == 'SmoothedMaxDivergence'

    # TODO: Add meaningful tests of partial constructors:
    dp.m.then_private_expr(None, None) # type: ignore[arg-type]
    dp.m.then_private_lazyframe(None, None) # type: ignore[arg-type]
    dp.t.then_is_equal(None)
    dp.t.then_metric_bounded()
    dp.t.then_ordered_random()
    dp.t.then_quantile_score_candidates([], 0.5)
    dp.t.then_sum_of_squared_deviations()