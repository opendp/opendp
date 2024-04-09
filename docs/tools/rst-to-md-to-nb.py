# This is reinventing the wheel, but the existing tools that I've found
# are out of date and unmaintained, and/or don't handle the particular structure of our RST.
# Building something outside the Sphinx pipeline seems simpler for now,
# and Pandoc can be used for the heavy lifting.

import argparse
from pathlib import Path
import subprocess
import tempfile
import re


def run_command(cmd):
    print(cmd)
    completed_process = subprocess.run(
        cmd,
        capture_output=True,
        shell=True,
        text=True
    )
    print(completed_process.stderr)
    if completed_process.returncode != 0:
        raise Exception(f'subprocess failed : {cmd}')
    return completed_process.stdout


def clean_rst(rst_text, resource_path):
    # desired format: https://github.com/jgm/pandoc/issues/6631#issuecomment-678727316
    abs_prefix = resource_path.absolute()
    spaces = ' ' * 4
    rst_text = re.sub(r'^\s+?\.\. literalinclude:: (\S+)', fr'.. include:: {abs_prefix}/\1\n{spaces}:code: python\n', rst_text, flags=re.MULTILINE)
    rst_text = re.sub(r'^\s+?\.\. tab-set::', '', rst_text, flags=re.MULTILINE)
    rst_text = re.sub(r'^\s+?\.\. tab-item::.*', '', rst_text, flags=re.MULTILINE) 
    rst_text = re.sub(r'^\s+?:sync:.*', '', rst_text, flags=re.MULTILINE)
    rst_text = re.sub(r'^\s+?:language: python', '', rst_text, flags=re.MULTILINE)
    rst_text = re.sub(r'^\s+?(:start-after: \S+)', fr'{spaces}\1', rst_text, flags=re.MULTILINE)
    rst_text = re.sub(r'^\s+?(:end-before: \S+)', fr'{spaces}\1', rst_text, flags=re.MULTILINE)
    return rst_text


def rst_to_md(rst_text, resource_path):
    clean_rst_text = clean_rst(rst_text, resource_path)
    with tempfile.NamedTemporaryFile(delete=False, suffix='.clean.rst') as temp:
        temp_path = Path(temp.name)
        temp_path.write_text(clean_rst_text)
        return run_command(
            f'pandoc --from rst --to markdown {temp_path}')


def clean_md(md_text):
    return md_text


def md_to_nb(md_text, resource_path):
    clean_md_text = clean_md(md_text)
    with tempfile.NamedTemporaryFile(delete=False, suffix='.clean.md') as temp:
        temp_path = Path(temp.name)
        temp_path.write_text(clean_md_text)
        return run_command(
            f'pandoc --from markdown --to ipynb --resource-path {resource_path} {temp_path}')


def rst_to_nb(rst_text, resource_path):
    md_text = rst_to_md(rst_text, resource_path=resource_path)
    with tempfile.NamedTemporaryFile(delete=False, suffix='.md') as temp:
        Path(temp.name).write_text(md_text)
        print(f'# Raw MD: {temp.name}')
    nb_text = md_to_nb(md_text, resource_path=resource_path)
    return nb_text


def read_write(input_path, output_path):
    rst_text = input_path.read_text()
    nb_text = rst_to_nb(rst_text, input_path.parent)
    output_path.write_text(nb_text)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', required=True, help='.rst input file')
    parser.add_argument('--output', required=True, help='.ipynb input file')
    args = parser.parse_args()
    read_write(Path(args.input), Path(args.output))


if __name__ == '__main__':
    main()