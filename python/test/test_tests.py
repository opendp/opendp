from doctest import run_docstring_examples, OPTIONFLAGS_BY_NAME

import pytest

import opendp.prelude as dp


def test_number_of_tests_found(request):
    tests_found = len(request.session.items)
    assert tests_found >= 835


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


def capture_doctest_result(doctest, capsys, options={}):
    optionflags = sum(OPTIONFLAGS_BY_NAME[option] for option in options)
    run_docstring_examples(doctest, {}, optionflags=optionflags)
    return capsys.readouterr().out

def assert_doctest_pass(doctest, capsys, options={}):
    result = capture_doctest_result(doctest, capsys, options)
    assert 'Failed example' not in result

def assert_doctest_fail(doctest, capsys, options={}):
    result = capture_doctest_result(doctest, capsys, options)
    assert 'Failed example' in result


def test_doctest_ignore(capsys):
    assert_doctest_fail('>>> 1/0', capsys)
    assert_doctest_pass('>>> 1/0', capsys, {'SKIP'})
    assert_doctest_fail('>>> 1/0', capsys, {'IGNORE'})
    assert_doctest_pass('>>> 2+2\nextra!', capsys, {'IGNORE'})

    with pytest.raises(Exception, match='IGNORE can not be used with other flags'):
        assert_doctest_pass('>>> 2+2', capsys, {'IGNORE', 'NUMBER'})


def test_doctest_fuzzy(capsys):
    assert_doctest_pass('>>> 2+2\n4', capsys)
    assert_doctest_fail('>>> 2+2\n5', capsys)
    assert_doctest_pass('>>> 2+2\n~5', capsys, {'FUZZY'})
    assert_doctest_fail('>>> 2+2\n~5 extra!', capsys, {'FUZZY'})

    with pytest.raises(Exception, match='FUZZY can not be used with other flags'):
        assert_doctest_pass('>>> 2+2', capsys, {'FUZZY', 'NUMBER'})