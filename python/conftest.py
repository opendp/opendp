import pytest
import opendp.prelude as dp

@pytest.fixture(autouse=True)
def add_dp(doctest_namespace):
    doctest_namespace['dp'] = dp