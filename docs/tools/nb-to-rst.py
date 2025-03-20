import argparse
import subprocess
from pathlib import Path
import re


def get_rst(nb_path):
    return subprocess.check_output(
        ['jupyter', 'nbconvert', nb_path, '--to', 'rst', '--stdout'],
        text=True
    )


def unindent(text):
    return '\n'.join(line[4:] for line in text.split('\n'))


def doctest(text):
    # Not perfect, but usually works.
    return '\n'.join(
        (f'... {line}' if line[0] in [' ', ']', ')', '}'] else f'>>> {line}')
        if line else ''
        for line in text.split('\n')
    )


def reindent(text):
    return '\n'.join(' ' * 4 + line for line in text.split('\n'))


def convert_block(match):
    '''
    Given a match object, with subexpressions for the input and output lines,
    return a python code block containing a doctest.
    '''
    input = unindent(match.group(1))
    output = unindent((match.group(2) or ''))
    indent_input = reindent(reindent(reindent(doctest(input))))
    indent_output = reindent(reindent(reindent(output)))
    return f'''.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            {indent_input.strip()}
            {indent_output.strip()}

'''

_test_input = '''
Lorem

.. code:: ipython3

    successive()
    lines()

.. parsed-literal::

    works!

.. code:: ipython3

    or_split(
        'across lines'
    )

.. parsed-literal::

    also works!

ipsum!
'''

_test_output = '''
Lorem

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> successive()
            >>> lines()
            works!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> or_split(
            ...     'across lines'
            ... )
            also works!

ipsum!
'''

def convert_blocks(rst):
    '''
    Converts `ipython3` and `parsed-literal` blocks into doctests.
    This is not perfect because cells can combine multiple statements:
    We'll need to finish it by hand.

    >>> assert convert_blocks(_test_input) == _test_output
    Converted 2 code blocks
    '''
    pattern = r'\.\. code:: ipython3\n(.*?)(?:^\.\. parsed-literal::\n(.*?))?^(?=\S)'
    rst, count = re.subn(pattern, convert_block, rst, flags=re.MULTILINE | re.DOTALL)
    print(f"Converted {count} code blocks")
    return rst


# TODO
# def extract_code(rst, parent_path):
#     '''
#     Extracts code into standalone files
#     '''
#     return rst



def convert(nb_path):
    rst = get_rst(nb_path)
    rst = convert_blocks(rst)
    # rst = extract_code(rst)
    return rst


def main():
    parser = argparse.ArgumentParser(
        description="Wraps nbconvert, also converting code cells to tabbed doctests")
    parser.add_argument("nb_path", help="Notebook to convert", type=Path)
    args = parser.parse_args()
    rst = convert(args.nb_path)
    rst_path = args.nb_path.parent / f"{args.nb_path.stem}.rst"
    rst_path.write_text(rst)
    print(f"Wrote to: {rst_path}")



if __name__ == "__main__":
    main()
