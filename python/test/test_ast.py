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


def has_return(tree):
    '''
    >>> has_return(ast.parse('def nope(): pass'))
    False
    >>> has_return(ast.parse('def yeah(): return 1'))
    True
    '''
    for node in ast.walk(tree):
        if isinstance(node, ast.Return):
            return True
    return False


class Function(NamedTuple):
    file: str
    name: str
    node: ast.AST
    is_public: bool


functions = []

src_dir_path = Path(__file__).parent.parent / 'src'
for code_path in src_dir_path.glob('**/*.py'):
    is_public = True
    if any(parent.name.startswith('_') for parent in code_path.parents):
        is_public = False
    if code_path.name.startswith('_') and code_path.name != '__init__.py':
        is_public = False
    code = code_path.read_text()
    tree = ast.parse(code)
    for node in ast.walk(tree):
        if not isinstance(node, ast.FunctionDef):
            continue
        if ends_with_ellipsis(node):
            continue
        if node.name.startswith('_'):
            is_public = False
        short_path = f'{code_path.parent.name}/{code_path.name}'

        function = Function(
            file=short_path,
            name=node.name,
            node=node,
            is_public=is_public,
        )
        functions.append(function)

# Typo check:
assert len(functions) > 100


@pytest.mark.parametrize("file,name,node,is_public", functions)  # type: ignore[attr-defined]
def test_public_function_docs(file, name, node, is_public):
    where = f'In {file}, def {name}, line {node.lineno}'

    # First, check the docstring in isolation:
    docstring = ast.get_docstring(node)
    if is_public:
        assert docstring is not None, f'{where}, add docstring or make private'
    else:
        if docstring is None:
            return

    directives = set(re.findall(r'^\s*(\:\w+:?)', docstring, re.MULTILINE))
    unknown_directives = directives - {':param', ':rtype:', ':type', ':raises', ':example:', ':return:'}
    assert not unknown_directives, (
        f'{where} has unknown directives: {", ".join(unknown_directives)}'
    )

    param_dict = dict(re.findall(r':param (\w+):(.*)', docstring))
    # TODO: Maybe accept either description or type?
    # k_wo_v = [k for k, v in param_dict.items() if not v.strip()]
    # assert not k_wo_v, (
    #     f'{where} has params without descriptions: {", ".join(k_wo_v)}'
    # )

    # Then, compare the docstring to the AST:
    args = (
        node.args.posonlyargs
        + node.args.args
        + node.args.kwonlyargs
    )
    if node.args.kwarg is not None:
        args.append(node.args.kwarg)

    # TODO: For "self" and "cls", confirm that it really is a method.
    arg_names = {arg.arg for arg in args} - {'self'} - {'cls'}
    if param_dict or is_public:
        # Private functions don't need to document params,
        # but if they do, they should be consistent with signature.
        assert param_dict.keys() == arg_names, (
            f'{where}, docstring params ({", ".join(param_dict.keys())}) '
            f'!= function signature ({", ".join(arg_names)})'
        )

    # TODO: check for documentation of return value

 