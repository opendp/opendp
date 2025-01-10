from pathlib import Path
import re

import pytest

import opendp.prelude as dp

@pytest.mark.parametrize("ext", ['ipynb', 'rst'])
@pytest.mark.parametrize("cmd", ['wget', 'curl'])
def test_no_shell_http(ext, cmd):
    paths = list((Path(__file__).parent.parent.parent.parent / 'docs' / 'source').glob(f'**/*.{ext}'))
    assert len(paths) > 0
    mistakes = []
    for path in paths:
        for n, line in enumerate(path.read_text().splitlines()):
            match = re.search(fr'\b{cmd}\W+\w\S+', line)
            if match:
                mistakes.append(f'"{match.group(0)}" on line {n} of {path}')
    assert len(mistakes) == 0, "\n".join(mistakes)

def test_california_pums():
    assert dp.examples.get_california_pums_path().read_text().startswith('59,1,9,1,0,1')

def test_france_lfs():
    assert dp.examples.get_france_lfs_path().read_text().startswith('COEFF,QUARTER,REFYEAR,REFWEEK')


