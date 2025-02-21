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

def assert_pass(doctest, capsys, options={}):
    result = capture_doctest_result(doctest, capsys, options)
    assert 'Failed example' not in result

# Use caps-lock just so it's easier to read the tests and see the differences.
def assert_FAIL(doctest, capsys, options={}):
    result = capture_doctest_result(doctest, capsys, options)
    assert 'Failed example' in result


def test_doctest_ignore(capsys):
    assert_FAIL('>>> 1/0', capsys)
    assert_pass('>>> 1/0', capsys, {'SKIP'})
    assert_FAIL('>>> 1/0', capsys, {'IGNORE'})

    assert_FAIL('>>> 2+2', capsys)
    assert_pass('>>> 2+2', capsys, {'SKIP'})
    assert_FAIL('>>> 2+2', capsys, {'IGNORE'})

    assert_pass('>>> 2+2\n4', capsys)
    assert_pass('>>> 2+2\n4', capsys, {'SKIP'})
    assert_pass('>>> 2+2\n4', capsys, {'IGNORE'})

    assert_FAIL('>>> 2+2\nextra!', capsys)
    assert_pass('>>> 2+2\nextra!', capsys, {'SKIP'})
    assert_pass('>>> 2+2\nextra!', capsys, {'IGNORE'})

    with pytest.raises(Exception, match='IGNORE can not be used with other flags'):
        assert_pass('>>> 2+2', capsys, {'IGNORE', 'NUMBER'})


# Include "polars" in name to filter out in smoke-test.yml.
def test_doctest_fuzzy_polars(capsys):
    doctest_in = '''
    >>> import polars as pl
    >>> pl.DataFrame({'col': ['value']})'''
    doctest_out = '''
    shape: (1, 1)
    ┌───────┐
    │ col   │
    │ ---   │
    │ str   │
    ╞═══════╡
    │ value │
    └───────┘
    '''
    doctest_out_rows = '''
    shape: (2, 1)
    ┌───────────┐
    │ col       │
    │ ---       │
    │ str       │
    ╞═══════════╡
    │ value     │
    │ surprise! │
    └───────────┘
    '''
    doctest_out_cols = '''
    shape: (1, 100)
    ┌───────┐
    │ col   │
    │ ---   │
    │ str   │
    ╞═══════╡
    │ value │
    └───────┘
    '''
    assert_FAIL(doctest_in, capsys)
    assert_pass(doctest_in, capsys, {'SKIP'})
    assert_FAIL(doctest_in, capsys, {'FUZZY_DF'})

    assert_pass(doctest_in + doctest_out, capsys)
    assert_pass(doctest_in + doctest_out, capsys, {'SKIP'})
    assert_pass(doctest_in + doctest_out, capsys, {'FUZZY_DF'})

    assert_FAIL(doctest_in + doctest_out_rows, capsys)
    assert_pass(doctest_in + doctest_out_rows, capsys, {'SKIP'})
    assert_pass(doctest_in + doctest_out_rows, capsys, {'FUZZY_DF'})

    assert_FAIL(doctest_in + doctest_out_cols, capsys)
    assert_pass(doctest_in + doctest_out_cols, capsys, {'SKIP'})
    assert_FAIL(doctest_in + doctest_out_cols, capsys, {'FUZZY_DF'})
    
    with pytest.raises(Exception, match='FUZZY_DF can not be used with other flags'):
        assert_pass(doctest_in, capsys, {'FUZZY_DF', 'NUMBER'})