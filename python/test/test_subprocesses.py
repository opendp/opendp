import subprocess
import sys
import pytest

tests = {
    'linting': 'cd .. && flake8 . --count --show-source --statistics',
    'type checking': 'mypy .'
}

@pytest.mark.skipif(sys.version_info < (3, 11), reason='mypy will fail on 3.8')
@pytest.mark.parametrize("cmd", tests.values(), ids=tests.keys())
def test_subprocess(cmd):
    result = subprocess.run(cmd, shell=True)
    assert result.returncode == 0, f'"{cmd}" failed'
