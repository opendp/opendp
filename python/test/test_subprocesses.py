import subprocess
import pytest

tests = {
    'linting': 'cd .. && flake8 . --count --show-source --statistics',
    'type checking': 'mypy .'
}

@pytest.mark.parametrize("cmd", tests.values(), ids=tests.keys())
def test_subprocess(cmd):
    subprocess.run(cmd, shell=True, check=True)
