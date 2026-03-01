from pathlib import Path
import subprocess
import sys
import os


def for_each(glob, template):
    total = 0
    success = 0
    fails = []

    for file in Path('.').glob(glob):
        total += 1
        cmd = template.replace('{}', str(file))
        print(f'START: {cmd}')
        result = subprocess.run(cmd, shell=True)
        if result.returncode == 0:
            print(f'PASS: {cmd}')
            success += 1
        else:
            print(f'FAIL: {cmd}')
            fails.append(str(file))
    
    print(f'{success}/{total} for "{template}"')
    if len(fails) != 0:
        print('These failed:')
        print('\n'.join(fails))
        print()
    if total == 0:
        print(f'Nothing matched "{glob}"')

    return total != 0 and len(fails) == 0

def main():
    os.chdir(Path(__file__).parent.parent / 'docs' / 'source')
    tests_pass = for_each('**/*.R', "Rscript {}")

    if not tests_pass:
        sys.exit(1)

if __name__ == "__main__":
    main()