import argparse
import subprocess
import re
from collections import defaultdict


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
    categories = defaultdict(list)
    for line in lines:
        line = re.sub(r'^\w+\s+', '', line)
        line = re.sub(r'\(#(\d+)\)', r'[#\1](https://github.com/opendp/opendp/pull/\1)', line)
        categories['todo'].append(line)

    output_lines = []
    for k, v in categories.items():
        output_lines.append(f'### {k}')
        output_lines.append('')
        for line in v:
            output_lines.append(f'- {line}')
    
    return '\n'.join(output_lines)


def main():
    parser = argparse.ArgumentParser(description="Helps generate CHANGELOG entries")
    parser.add_argument("--until", help="Read from git log until match found", required=False)
    args = parser.parse_args()
    lines = log_until(args.until)
    print(parse_log(lines))


if __name__ == "__main__":
    main()