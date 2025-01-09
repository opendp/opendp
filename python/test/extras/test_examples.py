from pathlib import Path
import re

import pytest

@pytest.mark.parametrize("ext", ['ipynb', 'rst'])
@pytest.mark.parametrize("cmd", ['wget', 'curl'])
def test_no_shell_http(ext, cmd):
    paths = list((Path(__file__).parent.parent.parent.parent / 'docs' / 'source').glob(f'**/*.{ext}'))
    assert len(paths) > 0
    mistakes = []
    for path in paths:
        for n, line in enumerate(path.read_text().splitlines()):
            match = re.search(fr'\b{cmd}\s+\S+', line)
            if match:
                mistakes.append(f'"{match.group(0)}" on line {n} of {path}')
    assert len(mistakes) == 0, "\n".join(mistakes)