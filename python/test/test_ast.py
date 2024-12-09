from pathlib import Path
import ast
import re
from typing import NamedTuple

import pytest


class Function(NamedTuple):
    file: str
    node: ast.AST

    def __repr__(self):
        return f'{self.file} [line {self.node.lineno}]: {self.node.name}()'


public_functions = []

src_dir_path = Path(__file__).parent.parent / 'src'
for code_path in src_dir_path.glob('**/*.py'):
    if code_path.name.startswith('_') and code_path.name != '__init__.py':
        continue
    code = code_path.read_text()
    tree = ast.parse(code)
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef):
            if not node.name.startswith('_'):
                rel_path = re.sub(r'.*/src/', '', str(code_path))
                public_functions.append(Function(file=rel_path, node=node))


# _file and _name are included to make the test output more readable.
@pytest.mark.parametrize("_file,_name,node", [(f.file, f.node.name, f.node) for f in public_functions])
def test_function_docs(_file, _name, node):
    docstring = ast.get_docstring(node) or ''
    param_names = set(re.findall(r':param (\w+):', docstring))
    args = (
        node.args.posonlyargs
        + node.args.args
        + node.args.kwonlyargs
    )
    arg_names = {arg.arg for arg in args} - {'self'}

    assert param_names == arg_names

 