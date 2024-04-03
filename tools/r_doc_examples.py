from pathlib import Path
import subprocess
import sys
import os

total = 0
success = 0
fails = []
r_glob = '**/*.R'

os.chdir(Path(__file__).parent.parent / 'docs')
for r_script in Path('.').glob(r_glob):
    total += 1
    result = subprocess.run(['Rscript', r_script])
    if result.returncode == 0:
        success += 1
    else:
        fails.append(r_script)

print(f'{success}/{total} scripts passed')

if total == 0:
    print(f'Nothing matched "{r_glob}"')
    sys.exit(1)
if fails:
    print(f'These failed: {", ".join(str(f) for f in fails)}')
    sys.exit(1)