import pytest
import doctest
import re

import opendp.prelude as dp


@pytest.fixture(autouse=True)
def add_dp(doctest_namespace):
    doctest_namespace['dp'] = dp


FUZZY = doctest.register_optionflag('FUZZY')
FUZZY_DF = doctest.register_optionflag('FUZZY_DF')
IGNORE = doctest.register_optionflag('IGNORE')

# Related doctest flags:
# - NUMBER (https://docs.pytest.org/en/7.1.x/how-to/doctest.html?highlight=NUMBER#using-doctest-options)
#   Floating-point numbers only need to match as far as the precision you have written in the expected doctest output
# - SKIP (https://docs.python.org/3/library/doctest.html#doctest.SKIP)
#   Do not run the example at all

def _norm_df(df):
    df = re.sub(r'╞.*', '', df, flags=re.DOTALL)
    df = re.sub(r'\s+', ' ', df)
    df = re.sub(r'─+', '─', df)  # Special character used in box
    df = re.sub(r'shape: \(\d+,', 'shape: (#', df)
    return df

class CustomOutputChecker(doctest.OutputChecker):
    def check_output(self, want, got, optionflags):
        if FUZZY & optionflags:
            if optionflags - FUZZY:
                raise Exception('FUZZY can not be used with other flags')
            # A replacement string has special behavior with backslashes,
            # but the value returned by a lambda does not.
            number_re = lambda _: r'-?\d+(\.\d*)?'
            # Replace each tilde-number with a regex representing any possible number.
            want_re = re.sub(r'\\~(\\-)?\d+(\\\.\d*)?', number_re, re.escape(want.strip()))
            return bool(re.search(want_re, got))
        if FUZZY_DF & optionflags:
            if optionflags - FUZZY_DF:
                raise Exception('FUZZY_DF can not be used with other flags')
            return _norm_df(want) == _norm_df(got)
        if IGNORE & optionflags:
            if optionflags - IGNORE:
                raise Exception('IGNORE can not be used with other flags')
            return 'Traceback' not in got
        return super().check_output(want, got, optionflags)

doctest.OutputChecker = CustomOutputChecker  # type: ignore