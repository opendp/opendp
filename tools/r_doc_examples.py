from pathlib import Path
import subprocess
import sys
import os

def scan_r_examples(template):
    total = 0
    success = 0
    fails = []

    os.chdir(Path(__file__).parent.parent / 'docs' / 'source')
    r_glob = '**/*.R'
    for r_example in Path('.').glob(r_glob):
        total += 1
        cmd = template.replace('{}', str(r_example))
        print(f'Running: {cmd}')
        result = subprocess.run(cmd, shell=True)
        if result.returncode == 0:
            success += 1
        else:
            fails.append(str(r_example))
    
    print(f'{success}/{total} for "{template}"')
    if fails:
        print('Failed:')
        print('\n'.join(fails))
        print()
    return not bool(fails)

tests_pass = scan_r_examples("Rscript {}")

if not tests_pass:
    print('FAIL')
    sys.exit(1)