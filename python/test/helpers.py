from contextlib import contextmanager
import pytest


@contextmanager
def optional_dependency(module_name):
    import sys
    import re
    install_names = {
        'sklearn': 'scikit-learn',
    }
    if module_name not in sys.modules:
        install_name = install_names.get(module_name) or module_name
        expected_message = f'The optional install {install_name} is required for this functionality'
        with pytest.raises(ImportError, match=re.escape(expected_message)):
            yield
        raise pytest.skip('Saw expected ImportError; skipping rest of test')
    else:
        yield