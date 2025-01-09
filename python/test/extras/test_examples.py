from pathlib import Path
import re
import polars as pl

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
    assert dp.examples.get_california_pums().read().startswith('59,1,9,1,0,1')

def test_france_lfs():
    assert dp.examples.get_france_lfs().read().startswith('COEFF,QUARTER,REFYEAR,REFWEEK')

def test_context_with_stringio_fails():
    # TODO: Let this work. https://github.com/opendp/opendp/issues/2230 
    with pytest.raises(RuntimeError, match='the enum variant ScanSources::Buffers cannot be serialized'):
        dp.Context.compositor(
            data=pl.scan_csv(dp.examples.get_france_lfs(), ignore_errors=True),
            privacy_unit=dp.unit_of(contributions=36),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=10
        )

