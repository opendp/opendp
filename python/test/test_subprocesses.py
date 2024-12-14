import subprocess
import sys
import pytest

tests = {
    # "cd .." because we want to lint the build tools as well.
    'flake8 linting': 'cd .. && flake8 . --count --show-source --statistics',
    # Had non-reproducible errors between local runs, so disable caching.
    'mypy type checking': 'mypy . --cache-dir=/dev/null',
    # Looking for cases where first character is not `'`, `"`, or `f`
    # (for f-string). Negative lookahead would be more precise.
    '"print" in test/ should be explained': "! egrep 'print\\(([^\"'\"'\"'f]' -r test",
    '"print" in src/ should be log instead': "! egrep '^\\s+print\\(' -r src",
}

@pytest.mark.skipif(sys.version_info < (3, 11), reason='mypy will fail on 3.9')
@pytest.mark.parametrize("cmd", tests.values(), ids=tests.keys())
def test_subprocess(cmd):
    result = subprocess.run(cmd, shell=True)
    assert result.returncode == 0, f'"{cmd}" failed'
