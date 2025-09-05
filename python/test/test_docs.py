from pathlib import Path
from json import loads
import pytest
import re
from opendp import measurements, transformations

@pytest.mark.parametrize(
    "module,function",
    [
        (m, f)
        for m in [measurements, transformations]
        for f in [
            getattr(m, f_name)
            for f_name in dir(m)
            if f_name.startswith('then_')
        ]
    ])
def test_thens_are_documented(module, function):
    m_name = module.__name__
    then_name = function.__name__
    make_name = then_name.replace('then_', 'make_')

    assert function.__doc__ is not None, 'missing documentation'
    assert f':py:func:`{m_name}.{make_name}`' in function.__doc__, f'no link to {make_name}'


docs_source = Path(__file__).parent.parent.parent / 'docs' / 'source'
assert docs_source.exists()


def get_self_and_parent(path: Path):
    return f'{path.parent.name}/{path.name}'

@pytest.mark.parametrize(
    "rst_path",
    list(docs_source.glob("**/*.rst")),
    ids=get_self_and_parent
)
def test_code_block_language(rst_path: Path):
    rst_lines = rst_path.read_text().splitlines()
    expected = ['pycon', 'rust', 'latex', 'r', 'shell']

    # For autoformatting of doctests, code blocks should be "pycon", not "python".
    # https://github.com/adamchainz/blacken-docs?tab=readme-ov-file#restructuredtext
    assert 'python' not in expected

    errors = []
    for i, line in enumerate(rst_lines):
        m = re.search(r'\.\.\s+code::\s+(\w+)', line)
        if m:
            language = m.group(1)
            if language not in expected:
                errors.append(f'line {i+1}: Got "{language}", expected one of: {", ".join(expected)}')
    assert not errors, '\n'.join(errors)


@pytest.mark.parametrize(
    "rst_path",
    list(docs_source.glob("**/*.rst")),
    ids=get_self_and_parent
)
def test_single_backticks(rst_path: Path):
    rst_lines = rst_path.read_text().splitlines()
    errors = []
    for i, line in enumerate(rst_lines):
        line = line.strip()
        if line.startswith(('>>>', '...', '//')):
            # Probably in a comment in a code sample: Skip.
            continue
        m = re.search(r'''
            ([^`:]|^)   # Non-backtick or start of line
            `           # backtick
            ([^`<>_:]+) # content, excluding RST links and tags
            `           # backtick
            ([^`]|$)    # Non-backtick or end of line
        ''', line, re.VERBOSE)
        if m:
            content = m.group(2)
            errors.append(f'line {i+1}: "{content}" will be italicized: add double-backticks, or change to "*".')
    assert not errors, '\n'.join(errors)


python_src = Path(__file__).parent.parent / 'src'
assert python_src.exists()


@pytest.mark.parametrize(
    "path",
    list(docs_source.glob("**/*.rst")) + list(python_src.glob("**/*.py")),
    ids=get_self_and_parent
)
def test_doctest_empty_lines(path: Path):
    lines = path.read_text().splitlines()
    errors = []
    for i, line in enumerate(lines):
        if re.search(r'>>>\s*$', line):
            errors.append(f'line {i+1}: wrap with "code::" block and drop empty ">>>".')
    assert not errors, '\n'.join(errors)


@pytest.mark.parametrize(
    "nb_path",
    list(docs_source.glob("**/*.ipynb")),
    ids=lambda path: path.name
)
def test_notebooks_are_executed(nb_path):
    nb = loads(nb_path.read_text())
    counts_sources = [(cell.get('execution_count'), ''.join(cell.get('source', ''))) for cell in nb['cells'] if cell['cell_type'] == 'code']
    triples = [(index, count, source) for (index, (count, source)) in enumerate(counts_sources, start=1)]
    indexes, counts, sources = zip(*triples)

    # Info for error message:
    bad_sources = [source for (index, count, source) in triples if index != count]
    from sys import version_info
    short_path = nb_path.relative_to(Path.cwd(), walk_up=True) if version_info >= (3, 12) else nb_path.name

    # Notebook execution requires dependencies (jupyter, matplotlib, ...) beyond the basic dev environment.
    assert indexes == counts, f'''Notebook not completely executed.
To fix: jupyter nbconvert --to notebook --execute {short_path} --inplace
First cell with missing or misordered execution:\n{bad_sources[0].splitlines()[0]}'''


def is_public_and_not_top(path: Path):
    '''
    >>> is_public_and_not_top(Path('_parent/child.py'))
    False
    >>> is_public_and_not_top(Path('_parent/__init__.py'))
    False
    >>> is_public_and_not_top(Path('parent/__init__.py'))
    True
    >>> is_public_and_not_top(Path('parent/_child.py'))
    False
    >>> is_public_and_not_top(Path('parent/child.py'))
    True
    '''
    return not path.parent.name.startswith('_') and (
        not path.name.startswith('_') or path.name == '__init__.py'
    ) and not (path.parent.name == 'extras' and path.name == '__init__.py')


public_extras = [
    path for path in 
    Path(__file__).parent.parent.glob("src/opendp/extras/**/*.py")
    if is_public_and_not_top(path)
]


def get_namespace(py_path: Path):
    # Getting __doc__ from the module might be cleaner,
    # but walking through the module hierarchy is trickier than path glob.
    ns = []
    parent_parent_name = py_path.parent.parent.name
    parent_name = py_path.parent.name
    stem = py_path.stem
    if parent_parent_name != 'extras' and parent_name != 'extras':
        ns.append(parent_parent_name)
    if parent_name != 'extras':
        ns.append(parent_name)
    if stem != '__init__':
        ns.append(stem)
    return f'dp.{".".join(ns)}'


def get_docstring(py_path: Path):
    import ast
    py = py_path.read_text()
    module_ast = ast.parse(py)
    # This is a little fragile, but as long as a docstring is first, it should work.
    return module_ast.body[0].value.value.strip() # type: ignore[attr-defined]
 

@pytest.mark.parametrize(
    "py_path",
    public_extras,
    ids=get_namespace
)
def test_extras_docstring_convenience(py_path):
    actual = get_docstring(py_path)
    expected_ns = get_namespace(py_path)
    expected = f"""
For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``{expected_ns}``.    
""".strip()
    assert expected in actual, f"expected not in actual: {expected=}\n{actual=}"


@pytest.mark.parametrize(
    "py_path",
    public_extras,
    ids=get_namespace
)
def test_extras_docstring_installs(py_path):
    actual = get_docstring(py_path)
    namespace = get_namespace(py_path)
    if 'sklearn' in namespace:
        extra = 'scikit-learn'
    elif 'numpy' in namespace:
        extra = 'numpy'
    elif 'polars' in namespace:
        extra = 'polars'
    elif 'mbi' in namespace:
        extra = 'mbi'
    elif 'examples' in namespace:
        pytest.skip("dp.examples does not need extra installs")
    expected = f"""
This module requires extra installs: ``pip install 'opendp[{extra}]'``   
""".strip()
    assert expected in actual, f"expected not in actual: {expected=}\n{actual=}"