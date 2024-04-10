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
    print(f'RUN: {cmd}')
    completed_process = subprocess.run(
        cmd,
        capture_output=True,
        shell=True,
        text=True
    )
    if completed_process.stderr:
        print(f'STDERR: {completed_process.stderr}')
    if completed_process.returncode != 0:
        raise Exception(f'subprocess failed : {cmd}')
    return completed_process.stdout


def clean_rst(rst_text, prefix):
    '''
    Translate Sphinx extension tags to RST built-ins that Pandoc will process.
    TODO: Pick just the first tab of a set.

    >>> print(clean_rst("""
    ... .. tab-set::
    ... 
    ...     .. tab-item:: Context API
    ...         :sync: context
    ... 
    ...         .. literalinclude:: code/typical-workflow-context.rst
    ...             :language: python
    ...             :start-after: unit-of-privacy
    ...             :end-before: /unit-of-privacy
    ... """, prefix="/root"))
    <BLANKLINE>
    .. include:: /root/code/typical-workflow-context.rst
        :code: python
        :start-after: unit-of-privacy
        :end-before: /unit-of-privacy
    <BLANKLINE>
    '''
    spaces = ' ' * 4
    rst_text = re.sub(r'^\s+?\.\. literalinclude:: (\S+)', fr'.. include:: {prefix}/\1\n{spaces}:code: python\n', rst_text, flags=re.MULTILINE)
    rst_text = re.sub(r'^\s+?\.\. tab-set::', '', rst_text, flags=re.MULTILINE)
    rst_text = re.sub(r'^\s+?\.\. tab-item::.*', '', rst_text, flags=re.MULTILINE) 
    rst_text = re.sub(r'^\s+?:sync:.*', '', rst_text, flags=re.MULTILINE)
    rst_text = re.sub(r'^\s+?:language: python', '', rst_text, flags=re.MULTILINE)
    rst_text = re.sub(r'^\s+?(:start-after: \S+)', fr'{spaces}\1', rst_text, flags=re.MULTILINE)
    rst_text = re.sub(r'^\s+?(:end-before: \S+)', fr'{spaces}\1', rst_text, flags=re.MULTILINE)
    return rst_text


def rst_to_md(rst_text, prefix):
    with tempfile.NamedTemporaryFile(delete=False, suffix='.dirty.rst') as temp:
        temp_path = Path(temp.name)
        temp_path.write_text(rst_text)
        print(f'DIRTY RST: {temp.name}')

    clean_rst_text = clean_rst(rst_text, prefix)
    with tempfile.NamedTemporaryFile(delete=False, suffix='.clean.rst') as temp:
        temp_path = Path(temp.name)
        temp_path.write_text(clean_rst_text)
        print(f'CLEAN RST: {temp.name}')
        return run_command(
            f'pandoc --from rst --to markdown {temp_path}')


def undoctest_line(line):
    if line.startswith('>>> '):
        return line.replace('>>> ', '')
    if line.startswith('... '):
        return line.replace('... ', '')
    if line:
        return '# ' + line
    return line


def undoctest(match):
    '''
    >>> docstring = """
    ... ``` code
    ... >>> if True:
    ... ...     print('hello!')
    ... hello!
    ... ```
    ... """
    >>> match = re.search(r'(``` code)(.*?)(```)', docstring, flags=re.DOTALL)
    >>> print(undoctest(match))
    ``` code
    if True:
        print('hello!')
    # hello!
    ```
    '''
    pre = match.group(1)
    post = match.group(3)
    inside = '\n'.join(undoctest_line(line) for line in match.group(2).split('\n'))
    return pre + inside + post



def clean_md(md_text):
    md_text = re.sub(r'``` \{.*?\}', '``` code', md_text, flags=re.DOTALL)
    md_text = re.sub(r'(``` code)(.*?)(```)', undoctest, md_text, flags=re.DOTALL)
    return md_text


def md_to_nb(md_text, resource_path):
    with tempfile.NamedTemporaryFile(delete=False, suffix='.dirty.md') as temp:
        temp_path = Path(temp.name)
        temp_path.write_text(md_text)
        print(f'DIRTY MD: {temp.name}')

    clean_md_text = clean_md(md_text)
    with tempfile.NamedTemporaryFile(delete=False, suffix='.clean.md') as temp:
        temp_path = Path(temp.name)
        temp_path.write_text(clean_md_text)
        print(f'CLEAN MD: {temp.name}')

        return run_command(
            f'pandoc --from markdown --to ipynb --resource-path {resource_path} {temp_path}')


def rst_to_nb(rst_text, resource_path):
    prefix = resource_path.absolute()
    md_text = rst_to_md(rst_text, prefix)
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