import argparse
import subprocess
import re
from collections import defaultdict
from pathlib import Path


def get_prev_version():
    return (Path(__file__).parent.parent / 'VERSION').read_text().strip()


def log_until(match):
    lines = subprocess.check_output(['git', 'log', '--oneline'], text=True).splitlines()
    if match is None:
        return lines
    head = []
    for line in lines:
        if match in line:
            break
        head.append(line)
    return head


def parse_log(lines):
    '''
    >>> print(parse_log([
    ...     'abcd0000 Add: Colon and capital (#3)',
    ...     'abcd0001 add still works if missing (#2)',
    ...     'abcd0002 (tag) remove tags (#1)'
    ... ]))
    ### Add
    <BLANKLINE>
    - Colon and capital [#3](https://github.com/opendp/opendp/pull/3)
    - Still works if missing [#2](https://github.com/opendp/opendp/pull/2)
    <BLANKLINE>
    ### Remove
    <BLANKLINE>
    - Tags [#1](https://github.com/opendp/opendp/pull/1)
    <BLANKLINE>
    '''
    categories = defaultdict(list)
    for line in lines:
        line = re.sub(r'^\w+\s+', '', line) # Remove hash
        line = re.sub(r'^\([^)]+\)\s+', '', line) # Remove tag
        line = re.sub(r'\(#(\d+)\)', r'[#\1](https://github.com/opendp/opendp/pull/\1)', line)
        words = line.split(' ')
        first = re.sub(r'\W', '', words[0]).capitalize()
        rest = ' '.join(words[1:]).capitalize()
        categories[first].append(rest)

    output_lines = []
    for k, v in categories.items():
        output_lines.append(f'### {k}')
        output_lines.append('')
        for line in v:
            output_lines.append(f'- {line}')
        output_lines.append('')
    
    return '\n'.join(output_lines)


def main():
    parser = argparse.ArgumentParser(description="Helps generate CHANGELOG entries")
    parser.parse_args()
    prev_version = get_prev_version()
    lines = log_until(prev_version.replace('-dev', ''))
    print(parse_log(lines))


if __name__ == "__main__":
    main()