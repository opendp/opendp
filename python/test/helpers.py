import sys
import re
from contextlib import contextmanager
import pytest


@contextmanager
def optional_dependency(*module_names):
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
    if all(name in sys.modules for name in module_names): 
        # Proceed normally if installed:
        yield
    else:
        name_map = {
            'sklearn': 'scikit-learn',
        }
        install_names = [name_map.get(name) or name for name in module_names]
        expected_messages = [
            f'The optional install {name} is required for this functionality'
            for name in install_names
        ]
        expected_messages_re = '|'.join(re.escape(message) for message in expected_messages)
        # Otherwise, check that one of the expected errors is raised...
        with pytest.raises(Exception, match=expected_messages_re):
            yield
        # ... and then skip the rest of the test.
        raise pytest.skip('Saw expected OpenDPException; skipping rest of test')
