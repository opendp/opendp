import argparse
import subprocess
import re
from collections import defaultdict
from pathlib import Path
from channel_tool import match_first_changelog_header, get_changelog_lines


def get_prev_version():
    # retrieve previous version from the dev changelog entry
    _, match = match_first_changelog_header(get_changelog_lines())
    return match.group(2)


def log_until(tag):
    return subprocess.check_output(['git', 'log', f"{tag}..HEAD", '--oneline'], text=True).splitlines()
    


def get_changelog_update(lines):
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

    old_changelog_lines = (Path(__file__).parent.parent / 'CHANGELOG.md').read_text().splitlines()
    new_changelog_lines = []

    prev_version = get_prev_version()
    log_lines = log_until(prev_version.replace('-dev', ''))
    changelog_update = get_changelog_update(log_lines)

    for line in old_changelog_lines:
        if prev_version in line:
            new_changelog_lines.append(changelog_update)
            new_changelog_lines.append('')
        new_changelog_lines.append(line)

    (Path(__file__).parent.parent / 'CHANGELOG.md').write_text('\n'.join(new_changelog_lines))


if __name__ == "__main__":
    main()