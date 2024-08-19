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

