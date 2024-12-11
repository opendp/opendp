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


def return_type(tree):
    '''
    >>> return_type(ast.parse('def f() -> list[int]: return [1]').body[0])
    'list[int]'

    Need body[0] because parse wraps these in a "Module".
    '''
    type_node = tree.returns
    if type_node is not None:
        return ast.unparse(type_node)



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
    where = f'In {file}, line {node.lineno}, def {name}'

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
    # TODO: Has 68 failures; Enable and fill in the docs
    # k_missing_v = [k for k, v in param_dict.items() if not v.strip()]
    # assert not k_missing_v, (
    #     f'{where} has params missing descriptions: {", ".join(k_missing_v)}'
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

    if is_public:
        has_return_statement = has_return(node)
        has_return_directive = ':return:' in docstring
        # TODO: Has 142 failures; Enable and fill in the docs.
        # if has_return_statement:
        #     assert has_return_directive, (
        #         f'{where}, function has return statement, '
        #         'but no :return: in docstring'
        #     )
        if has_return_directive:
            assert has_return_statement, (
                f'{where}, function has :return: directive, '
                'but no return statement'
            )

    rtype_match = re.search(r':rtype:(.*)', docstring)
    if rtype_match:
        doc_rtype = rtype_match.group(1).strip().replace('"', "'")
        sig_rtype = return_type(node)
        # We trust mypy to check that the annotation is consistent with the actual return,
        # so we just check that the annotation is consistent with docstring.
        if sig_rtype is None:
            assert False, (
                f'{where}, to match :rtype:, add "-> {doc_rtype}"'
            )
        # If it's a single character, probably a typevar, so just skip.
        elif len(sig_rtype) > 1:
            assert doc_rtype == sig_rtype, (
                f'{where}, update :rtype: from "{doc_rtype}" to "{sig_rtype}"'
            )
 