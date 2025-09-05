import argparse
import subprocess
import re
from collections import defaultdict
from pathlib import Path


changelog_path = (Path(__file__).parent.parent / 'CHANGELOG.md')


def get_changelog_lines():
    '''
    >>> '# OpenDP Changelog' in get_changelog_lines()
    True
    '''
    return changelog_path.read_text().splitlines()


def get_prev_version(lines):
    '''
    >>> lines = ['# Title', '', '## [v1.2.3](github link)', '', 'changes!']
    >>> get_prev_version(lines)
    (2, 'v1.2.3')
    '''
    for i, line in enumerate(lines):
        if match := re.search(r'\[(v[^]]+)\]', line):
            return i, match.group(1)


def log_until(tag):
    subprocess.check_output(['git', 'fetch', '--tags']) # To make sure we have tags locally.
    return subprocess.check_output(['git', 'log', f"{tag}..HEAD", '--oneline'], text=True).splitlines()
    


def reformat_log(lines):
    '''
    >>> print(reformat_log([
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


def insert_updates(old_lines, new_lines, i):
    '''
    >>> old_lines = ['a', 'b', 'c']
    >>> new_lines = ['X', 'Y', 'Z']
    >>> insert_updates(old_lines, new_lines, 2)
    ['a', 'b', 'X', 'Y', 'Z', 'c']
    '''
    return old_lines[:i] + new_lines + old_lines[i:]

def main():
    parser = argparse.ArgumentParser(description="Helps generate CHANGELOG entries")
    parser.parse_args()

    old_lines = get_changelog_lines()

    i, prev_version = get_prev_version()
    raw_new_lines = log_until(prev_version.replace('-dev', ''))
    # TODO: I'm not sure where in the process "-dev" is added or removed.
    new_lines = reformat_log(raw_new_lines)

    updated_changelog = '\n'.join(insert_updates(old_lines, new_lines, i))

    changelog_path.write_text(updated_changelog)


if __name__ == "__main__":
    main()