from pathlib import Path
import ast
import re
from typing import NamedTuple

import pytest


class Function(NamedTuple):
    file: str
    node: ast.AST


def get_params_from_node_docstring(node):
    docstring = ast.get_docstring(node)
    return set(re.findall(r':param (\w+):', docstring))  # type: ignore[arg-type]


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
        if not ast.get_docstring(node):
            # TODO: All public functions should have docstrings
            continue
        if not get_params_from_node_docstring(node):
            # TODO: If the docstring has no params, should make sure that matches signature
            continue
        rel_path = re.sub(r'.*/src/', '', str(code_path))
        public_functions.append(Function(file=rel_path, node=node))


@pytest.mark.parametrize("file,name,node", [(f.file, f.node.name, f.node) for f in public_functions])  # type: ignore[attr-defined]
def test_function_docs(file, name, node):
    param_names = get_params_from_node_docstring(node)
    args = (
        node.args.posonlyargs
        + node.args.args
        + node.args.kwonlyargs
    )
    if node.args.kwarg is not None:
        args.append(node.args.kwarg)
    arg_names = {arg.arg for arg in args} - {'self'} - {'cls'} # TODO: Check that it really is a class method.

    assert param_names == arg_names, f'In {file}, function {name}, line {node.lineno}, docstring params ({", ".join(param_names)}) != function signature ({", ".join(arg_names)})'

 