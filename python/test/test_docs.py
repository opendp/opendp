from pathlib import Path
from json import loads
import pytest
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


@pytest.mark.parametrize(
    "nb_path",
    list((Path(__file__).parent.parent.parent / 'docs' / 'source').glob("**/*.ipynb")),
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

