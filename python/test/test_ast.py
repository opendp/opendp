from pathlib import Path
import ast
import re
from typing import NamedTuple

import pytest


def ends_with_ellipsis(node):
    '''
    >>> ends_with_ellipsis(ast.parse('def nope(): pass').body[0])
    False
    >>> ends_with_ellipsis(ast.parse('def yeah(): ...').body[0])
    True

    Need body[0] because parse wraps these in a "Module".
    '''
    last_node_value = getattr(node.body[-1], 'value', None)
    last_node_value_value = getattr(last_node_value, 'value', None)
    return last_node_value_value == Ellipsis

class Function(NamedTuple):
    file: str
    node: ast.AST


public_functions = []

src_dir_path = Path(__file__).parent.parent / 'src'
for code_path in src_dir_path.glob('**/*.py'):
    if any(parent.name.startswith('_') for parent in code_path.parents):
        continue
    if code_path.name.startswith('_') and code_path.name != '__init__.py':
        continue
    code = code_path.read_text()
    tree = ast.parse(code)
    for node in ast.walk(tree):
        if not isinstance(node, ast.FunctionDef):
            continue
        if node.name.startswith('_'):
            continue
        if ends_with_ellipsis(node):
            continue
        short_path = f'{code_path.parent.name}/{code_path.name}'
        public_functions.append(Function(file=short_path, node=node))

assert len(public_functions) > 100


@pytest.mark.parametrize("file,name,node", [(f.file, f.node.name, f.node) for f in public_functions])  # type: ignore[attr-defined]
def test_function_docs(file, name, node):
    where = f'In {file}, def {name}, line {node.lineno}'
    docstring = ast.get_docstring(node)
    assert docstring is not None, f'{where}, add docstring or make private'

    directives = set(re.findall(r'^\s*:(\w+)', docstring, re.MULTILINE))
    unknown_directives = directives - {'param', 'rtype', 'type', 'raises', 'example', 'return'}
    assert not unknown_directives, (
        f'{where} has unknown directives: {", ".join(unknown_directives)}'
    )

    param_names = set(re.findall(r':param (\w+):', docstring))
    args = (
        node.args.posonlyargs
        + node.args.args
        + node.args.kwonlyargs
    )
    if node.args.kwarg is not None:
        args.append(node.args.kwarg)

    # TODO: For "self" and "cls", check that it really is a method.
    arg_names = {arg.arg for arg in args} - {'self'} - {'cls'}
    assert param_names == arg_names, (
        f'{where}, docstring params ({", ".join(param_names)}) '
        f'!= function signature ({", ".join(arg_names)})'
    )

 