import sys
import re
from contextlib import contextmanager
import pytest


@contextmanager
def optional_dependency(module_name):
    '''
    A number of opendp functions rely on optional dependencies.
    If a user calls one of these functions without the appropriate installs,
    we want to be sure there is always a helpful error message.
    To that end, we should:
    - Make sure the test suite runs for any set of optional dependencies.
    - Avoid pytest.skipif: There is always a defined behavior to check.
    - If the test uses a dependency indirectly,
      wrap the first call with "with optional_dependency('numpy'):" or similar
    - If the test uses a library directly,
      use "np = pytest.importorskip('numpy')" as late in the code as possible,
      so we can catch any preceding indirect usages.

    If "optional_dependency('numpy')" and "pytest.importorskip('numpy')"
    are used in the same test, it is redundant, but doesn't do any harm. 
    '''
    if module_name in sys.modules:
        # Proceed normally if installed:
        yield
    else:
        install_names = {
            'sklearn': 'scikit-learn',
        }
        install_name = install_names.get(module_name) or module_name
        expected_message = f'The optional install {install_name} is required for this functionality'
        # Otherwise, check that the expected error is raised...
        with pytest.raises(ImportError, match=re.escape(expected_message)):
            yield
        # ... and then skip the rest of the test.
        raise pytest.skip('Saw expected ImportError; skipping rest of test')
