import pytest
import doctest
import re

import opendp.prelude as dp


@pytest.fixture(autouse=True)
def add_dp(doctest_namespace):
    doctest_namespace['dp'] = dp


FUZZY = doctest.register_optionflag('FUZZY')

class CustomOutputChecker(doctest.OutputChecker):
    def check_output(self, want, got, optionflags):
        if FUZZY & optionflags:
            # A replacement string has special behavior with backslashes,
            # but the value returned by a lambda does not.
            number_re = lambda _: r'-?\d+(\.\d*)?'
            # Replace each tilde-number with a regex representing any possible number.
            want_re = re.sub(r'\\~(\\-)?\d+(\\\.\d*)?', number_re, re.escape(want.strip()))
            return bool(re.search(want_re, got))
        return super().check_output(want, got, optionflags)

doctest.OutputChecker = CustomOutputChecker  # type: ignore