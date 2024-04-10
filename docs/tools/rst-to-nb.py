# This is reinventing the wheel, but the existing tools that I've found
# are out of date and unmaintained, and/or don't handle the particular structure of our RST.
# Building something outside the Sphinx pipeline seems simpler for now,
# and Pandoc can be used for the heavy lifting.

import argparse
from pathlib import Path
import subprocess
import tempfile
import re
from contextlib import contextmanager


debug = False


# Utility functions:


def run_command(cmd):
    global debug
    if debug:
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


@contextmanager
def text_to_temp(text, file_suffix):
    global debug
    with tempfile.NamedTemporaryFile(delete=not debug, suffix=file_suffix) as temp:
        temp_path = Path(temp.name)
        temp_path.write_text(text)
        if debug:
            print(f'TEMP: {temp.name}')
        yield temp_path


# Ugly regexes:
        

def clean_rst(rst_text, prefix):
    '''
    Translate Sphinx extension tags to RST built-ins that Pandoc will process.
    TODO: Pick just the first tab of a set.

    >>> print(clean_rst("""
    ... The start
    ...
    ... .. tab-set::
    ... 
    ...     .. tab-item:: Context API
    ...         :sync: context
    ...
    ...         A long, long time ago...
    ...
    ...         .. literalinclude:: code/typical-workflow-context.rst
    ...             :language: python
    ...             :start-after: unit-of-privacy
    ...             :end-before: /unit-of-privacy
    ...
    ...         In a galaxy far, far away...
    ...
    ... The end
    ... """, prefix="/root"))
    <BLANKLINE>
    The start
    <BLANKLINE>
    <BLANKLINE>
    A long, long time ago...
    <BLANKLINE>
    .. include:: /root/code/typical-workflow-context.rst
        :code: python
        :start-after: unit-of-privacy
        :end-before: /unit-of-privacy
    <BLANKLINE>
    In a galaxy far, far away...
    <BLANKLINE>
    The end
    <BLANKLINE>
    '''
    def sub(pattern, replacement, text):
        return re.sub(rf'^\s+?{pattern}', replacement, text, flags=re.MULTILINE)

    # Clear unneeded tags:
    rst_text = sub(r'\.\. tab-set::', '', rst_text)
    rst_text = sub(r'\.\. tab-item::.*', '', rst_text) 
    rst_text = sub(r':sync:.*', '', rst_text)
    rst_text = sub(r':language: python', '', rst_text)

    # Switch to RST built-in, remove indentation:
    spaces = ' ' * 4
    rst_text = sub(
        r'\.\. literalinclude:: (\S+)',
        fr'\n.. include:: {prefix}/\1\n{spaces}:code: python\n',
        rst_text)

    # Minimal indentation on inner tags:
    rst_text = sub(r'(:start-after: \S+)', fr'{spaces}\1', rst_text)
    rst_text = sub(r'(:end-before: \S+)', fr'{spaces}\1', rst_text)

    # Remove indentation on any remaining text, but do not remove newlines.
    # (This is sloppy: Running into the limitations of regex approach.)
    rst_text = re.sub(r'^[ \t]+([^: \t])', r'\1', rst_text, flags=re.MULTILINE)
    return rst_text


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


# High level functions:


def rst_to_md(dirty_rst_text, prefix):
    with text_to_temp(dirty_rst_text, '.dirty.rst'): pass

    clean_rst_text = clean_rst(dirty_rst_text, prefix)
    with text_to_temp(clean_rst_text, '.clean.rst') as temp:
        return run_command(
            f'pandoc --from rst --to markdown {temp}')


def md_to_nb(dirty_md_text, resource_path):
    with text_to_temp(dirty_md_text, '.dirty.md'): pass

    clean_md_text = clean_md(dirty_md_text)
    with text_to_temp(clean_md_text, '.clean.md') as temp:
        return run_command(
            f'pandoc --from markdown --to ipynb --resource-path {resource_path} {temp}')


def rst_to_nb(rst_text, resource_path):
    '''
    >>> print(re.sub(r'"id": \\S+', '...', rst_to_nb('hello?', Path('/root'))))
    {
     "cells": [
      {
       "cell_type": "markdown",
       "metadata": {},
       "source": [
        "hello?"
       ],
       ...
      }
     ],
     "nbformat": 4,
     "nbformat_minor": 5,
     "metadata": {}
    }
    <BLANKLINE>
    '''
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
    parser.add_argument('--output', required=True, help='.ipynb output file')
    parser.add_argument('--debug', action='store_true')
    args = parser.parse_args()
    global debug
    debug = args.debug
    read_write(Path(args.input), Path(args.output))


if __name__ == '__main__':
    main()