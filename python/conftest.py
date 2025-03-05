import pytest
import doctest
import re

import opendp.prelude as dp


@pytest.fixture(autouse=True)
def add_dp(doctest_namespace):
    doctest_namespace['dp'] = dp


FUZZY_DF = doctest.register_optionflag('FUZZY_DF')
IGNORE = doctest.register_optionflag('IGNORE')

# Related doctest flags:
# - NUMBER (https://docs.pytest.org/en/7.1.x/how-to/doctest.html?highlight=NUMBER#using-doctest-options)
#   Floating-point numbers only need to match as far as the precision you have written in the expected doctest output
# - SKIP (https://docs.python.org/3/library/doctest.html#doctest.SKIP)
#   Do not run the example at all
# - FUZZY (https://github.com/opendp/opendp/pull/2249#issuecomment-2667132750)
#   Just ignores a number preceded by "~", while still checking the rest of the output.

def _norm_df(df):
    df = re.sub(r'╞.*', '', df, flags=re.DOTALL)  # Ignore everything after the header.
    df = re.sub(r'\s+', ' ', df)  # Data in the table can change width of columns, so ignore whitespace.
    df = re.sub(r'─+', '─', df)  # Similarly, ignore changes in the special character used in box.
    df = re.sub(r'shape: \(\d+,', 'shape: (#', df)
    return df

class CustomOutputChecker(doctest.OutputChecker):
    def check_output(self, want, got, optionflags):
        if FUZZY_DF & optionflags:
            if optionflags - FUZZY_DF:
                raise Exception('FUZZY_DF can not be used with other flags')
            return _norm_df(want) == _norm_df(got)
        if IGNORE & optionflags:
            if optionflags - IGNORE:
                raise Exception('IGNORE can not be used with other flags')
            return 'Traceback' not in got and bool(len(want)) == bool(len(got))
        return super().check_output(want, got, optionflags)

doctest.OutputChecker = CustomOutputChecker  # type: ignore
