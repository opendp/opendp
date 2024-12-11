from pathlib import Path
import ast
import re
from typing import NamedTuple

import pytest


def ast_ends_with_ellipsis(node):
    '''
    >>> ast_ends_with_ellipsis(ast.parse('def nope(): pass').body[0])
    False
    >>> ast_ends_with_ellipsis(ast.parse('def yeah(): ...').body[0])
    True

    Need body[0] because parse wraps these in a "Module".
    '''
    last_node_value = getattr(node.body[-1], 'value', None)
    last_node_value_value = getattr(last_node_value, 'value', None)
    return last_node_value_value == Ellipsis


def ast_has_return(tree):
    '''
    >>> ast_has_return(ast.parse('def nope(): pass'))
    False
    >>> ast_has_return(ast.parse('def yeah(): return 1'))
    True
    '''
    for node in ast.walk(tree):
        if isinstance(node, ast.Return):
            return True
    return False


def ast_return_type(tree):
    '''
    >>> ast_return_type(ast.parse('def f() -> list[int]: return [1]').body[0])
    'list[int]'

    Need body[0] because parse wraps these in a "Module".
    '''
    type_node = tree.returns
    if type_node is not None:
        return ast.unparse(type_node)


class Checker():
    def __init__(self, tree, docstring, is_public):
        self.tree = tree
        self.docstring = docstring
        self.is_public = is_public
        self.errors = []

    def _check_docstring(self):
        directives = set(re.findall(r'^\s*(\:\w+:?)', self.docstring, re.MULTILINE))
        unknown_directives = directives - {':param', ':rtype:', ':type', ':raises', ':example:', ':return:'}
        if unknown_directives:
            self.errors.append(f'unknown directives: {", ".join(unknown_directives)}')

    def _check_params(self):
        doc_arg_dict = dict(re.findall(r':param (\w+):(.*)', self.docstring))
        # TODO: Has 68 failures; Enable and fill in the docs
        # k_missing_v = [k for k, v in doc_arg_dict.items() if not v.strip()]
        # if k_missing_v:
        #     errors.append(f'params missing descriptions: {", ".join(k_missing_v)}')

        args = self.tree.args
        all_ast_args = (
            args.posonlyargs
            + args.args
            + args.kwonlyargs
        )
        if args.kwarg is not None:
            all_ast_args.append(args.kwarg)

        # TODO: For "self" and "cls", confirm that it really is a method.
        all_arg_names = {arg.arg for arg in all_ast_args} - {'self'} - {'cls'}
        if doc_arg_dict or is_public:
            # Private functions don't need to document params,
            # but if they do, they should be consistent with signature.
            if doc_arg_dict.keys() != all_arg_names:
                self.errors.append(
                    f'docstring params ({", ".join(doc_arg_dict.keys())}) '
                    f'!= function signature ({", ".join(all_arg_names)})'
                )

    def _check_return(self):
        has_return_statement = ast_has_return(self.tree)
        has_return_directive = ':return:' in self.docstring
        # TODO: Has 261 failures; Enable and fill in the docs.
        # if has_return_statement and not has_return_directive:
        #     errors.append('return statement, but no :return: in docstring')
        if has_return_directive and not has_return_statement:
            self.errors.append(':return: directive, but no return statement')

        rtype_match = re.search(r':rtype:(.*)', self.docstring)
        if rtype_match:
            doc_rtype = rtype_match.group(1).strip().replace('"', "'")
            ast_rtype = ast_return_type(self.tree)
            # We trust mypy to check that the annotation is consistent with the actual return,
            # so we just check that the annotation is consistent with docstring.
            if ast_rtype is None:
                self.errors.append(f'to match :rtype:, add "-> {doc_rtype}"')
            # If it's a single character, probably a typevar, which we won't try to resolve.
            elif len(ast_rtype) > 1:
                if doc_rtype != ast_rtype:
                    self.errors.append(f'update :rtype: from "{doc_rtype}" to "{ast_rtype}"')

    def get_errors(self):
        self._check_docstring()
        self._check_params()
        self._check_return()
        if self.errors:
            return '; '.join(f'({i+1}) {e}' for i, e in enumerate(self.errors))


PUBLIC = 'public'
PRIVATE = 'private'

class Function(NamedTuple):
    file: str
    name: str
    tree: ast.AST
    visibility: str  # str rather than bool so the pytest report is more readable.


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
        if ast_ends_with_ellipsis(node):
            continue
        if node.name.startswith('_'):
            is_public = False
        short_path = f'{code_path.parent.name}/{code_path.name}'

        function = Function(
            file=short_path,
            name=node.name,
            tree=node,
            visibility=PUBLIC if is_public else PRIVATE,
        )
        functions.append(function)

# Typo check:
assert len(functions) > 100


@pytest.mark.parametrize("file,name,tree,visibility", functions)  # type: ignore[attr-defined]
def test_public_function_docs(file, name, tree, visibility):
    where = f'In {file}, line {tree.lineno}, def {name}'
    is_public = visibility == PUBLIC

    docstring = ast.get_docstring(tree)
    if not is_public and docstring is None:
       return
    assert docstring is not None, f'{where}, add docstring or make private'

    errors = Checker(
        tree=tree,
        docstring=docstring,
        is_public=is_public
    ).get_errors()
    if errors:
        pytest.fail(f'{where}: {errors}')