import subprocess
import pytest


@pytest.mark.parametrize("cmd", [
    'cd .. && flake8 . --count --show-source --statistics'
])
def test_subprocess(cmd):
    subprocess.run(cmd, shell=True, check=True)
