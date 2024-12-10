from pathlib import Path
import ast
import re
from typing import NamedTuple

import pytest


class Function(NamedTuple):
    file: str
    node: ast.AST


public_functions = []

src_dir_path = Path(__file__).parent.parent / 'src'
for code_path in src_dir_path.glob('**/*.py'):
    if code_path.name.startswith('_') and code_path.name != '__init__.py':
        continue
    code = code_path.read_text()
    tree = ast.parse(code)
    for node in ast.walk(tree):
        if not isinstance(node, ast.FunctionDef):
            continue
        if node.name.startswith('_'):
            continue
        last_node_value = getattr(node.body[-1], 'value', None)
        last_node_value_value = getattr(last_node_value, 'value', None)
        if last_node_value_value == Ellipsis:
            continue
        rel_path = re.sub(r'.*/src/', '', str(code_path))
        public_functions.append(Function(file=rel_path, node=node))


@pytest.mark.parametrize("file,name,node", [(f.file, f.node.name, f.node) for f in public_functions])  # type: ignore[attr-defined]
def test_function_docs(file, name, node):
    docstring = ast.get_docstring(node)
    if docstring is None:
        # TODO: Public functions should have docstrings
        return
    param_names = set(re.findall(r':param (\w+):', docstring))
    args = (
        node.args.posonlyargs
        + node.args.args
        + node.args.kwonlyargs
    )
    if node.args.kwarg is not None:
        args.append(node.args.kwarg)
    arg_names = {arg.arg for arg in args} - {'self'} - {'cls'} # TODO: Check that it really is a class method.

    assert param_names == arg_names, f'In {file}, function {name}, line {node.lineno}, docstring params ({", ".join(param_names)}) != function signature ({", ".join(arg_names)})'

 