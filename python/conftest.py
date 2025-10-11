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
        my_options = optionflags
        # ELLIPSIS can be safely ignored for our extensions.
        if optionflags & doctest.ELLIPSIS:
            my_options -= doctest.ELLIPSIS
        if FUZZY_DF & my_options:
            others = my_options - FUZZY_DF
            if others:
                raise Exception(f'FUZZY_DF can not be used with other flags: {others}')
            return _norm_df(want) == _norm_df(got)
        if IGNORE & my_options:
            others = my_options - IGNORE
            if others:
                raise Exception(f'IGNORE can not be used with other flags: {others}')
            return 'Traceback' not in got and bool(len(want)) == bool(len(got))
        return super().check_output(want, got, optionflags)

doctest.OutputChecker = CustomOutputChecker  # type: ignore