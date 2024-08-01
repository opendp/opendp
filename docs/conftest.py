import pytest

# initializes a variable pl that can be used by pytest-sphinx to skip doctests when polars is not installed
@pytest.fixture(autouse=True)
def add_doctest_globals(doctest_namespace):
    try:
        import polars as pl
    except ImportError:
        pl = None
    doctest_namespace['pl'] = pl