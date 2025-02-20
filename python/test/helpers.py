import sys
import re
from contextlib import contextmanager
import pytest

from opendp._lib import install_names


@contextmanager
def optional_dependency(name):
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
    # Proceed normally if installed:
    if name in sys.modules:
        yield
        return

    # Otherwise, confirm that expected error is raised...:
    root_name = name.split(".")[0]
    install_name = install_names.get(root_name) or root_name
    expected_message = f'The optional install {install_name} is required for this functionality'
    expected_message_re = re.escape(expected_message)
    with pytest.raises(Exception, match=expected_message_re):
        yield
    # ... and then skip the rest of the test.
    raise pytest.skip(f'Saw expected exception "{expected_message}". Skipping rest of test.')
