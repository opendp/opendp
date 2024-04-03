from pathlib import Path
import subprocess
import sys
import os

def scan_r_examples(template):
    total = 0
    success = 0
    fails = []

    os.chdir(Path(__file__).parent.parent / 'docs')
    r_glob = '**/*.R'
    for r_example in Path('.').glob(r_glob):
        total += 1
        cmd = template.replace('{}', str(r_example))
        result = subprocess.run(cmd, shell=True)
        if result.returncode == 0:
            success += 1
        else:
            fails.append(str(r_example))

    if total == 0:
        print(f'Nothing matched "{r_glob}"')
        sys.exit(1)
    
    print(f'{success}/{total} for "{template}"')
    if fails:
        print('Failed:')
        print('\n'.join(fails))
        print()
    return bool(fails)

tests_pass = scan_r_examples("Rscript {}")
lints_pass = scan_r_examples("Rscript -e 'lintr::lint(\"{}\")'")

if not all([tests_pass, lints_pass]):
    print('FAIL')
    sys.exit(1)