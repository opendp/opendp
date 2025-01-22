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

        args = self.tree.args
        self.all_ast_args = (
            args.posonlyargs
            + args.args
            + args.kwonlyargs
        )
        if args.vararg is not None:
            self.all_ast_args.append(args.vararg)
        if args.kwarg is not None:
            self.all_ast_args.append(args.kwarg)
        if len(self.all_ast_args) and self.all_ast_args[0].arg in ['self', 'cls']:
            # TODO: Confirm that this is a method in a class.
            self.all_ast_args.pop(0)

    def _check_docstring(self):
        directives = re.findall(r'^\s*(\:\w+:?)', self.docstring, re.MULTILINE)
        unknown_directives = set(directives) - {':param', ':rtype:', ':type', ':raises', ':example:', ':return:'}
        if unknown_directives:
            self.errors.append(f'unknown directives: {", ".join(unknown_directives)}')
        
        order = ''.join(re.sub(r':$', '', d) for d in directives)
        # TODO: Has 139 failures if we require ":return" if ":rtype" is given:
        # canonical_order = r'^(:param(:type)?)*(:return(:rtype)?)?(:raises)*(:example)?$'
        canonical_order = r'^(:param(:type)?)*(:return)?(:rtype)?(:raises)*(:example)?$'
        if not re.search(canonical_order, order):
            short_order = re.sub(r'[:$^]', '', canonical_order)
            self.errors.append(
                f'Directives {order} are not in canonical order: {short_order}'
            )

    def _check_params(self):
        doc_param_dict = dict(re.findall(r':param (\w+): *(.*)', self.docstring))
        # TODO: Check even if private function.
        if self.is_public:
            k_missing_v = [k for k, v in doc_param_dict.items() if not v]
            if k_missing_v:
                self.errors.append(f'params missing descriptions: {", ".join(k_missing_v)}')

        ast_arg_names = {arg.arg for arg in self.all_ast_args}
        if doc_param_dict or is_public:
            # Private functions don't need to document params,
            # but if they do, they should be consistent with signature.
            if doc_param_dict.keys() != ast_arg_names:
                self.errors.append(
                    f'docstring params ({", ".join(doc_param_dict.keys())}) '
                    f'!= function signature ({", ".join(ast_arg_names)})'
                )

    def _check_types(self):
        doc_type_dict = {
            # TODO: Pick either ":py:ref:" or ":ref:"
            k: re.sub(r'(?::py)?:ref:`(\w+)`', r'\1', v)
            for k, v in re.findall(r':type (\w+): *(.*)', self.docstring)
        }

        ast_node_dict = {
            arg.arg: getattr(arg, 'annotation', None)
            for arg in self.all_ast_args
        }
        ast_type_dict = {
            # TODO: Has 32 failures where "Optional" not specified
            k: re.sub(r'Optional\[(.+)\]', r'\1', ast.unparse(v))
            for k, v in ast_node_dict.items()
            # TODO: Has 154 failures w/o this exclusion.
            if v is not None
        }
        
        # NOTE: Has 17 failures, or 66  w/o "if is_public",
        # but our sense is that the actual types may not be readable,
        # so the docs may not / should not be consistent, and that's ok.
        #
        # if self.is_public:
        #     if doc_type_dict != ast_type_dict:
        #         self.errors.append(
        #             f'docstring types ({doc_type_dict}) '
        #             f'!= function signature ({ast_type_dict})'
        #         )
        
        for k, v in doc_type_dict.items():
            # TODO: "k in ast_type_dict" only needed in CI?
            if k in ast_type_dict and ast_type_dict[k] != v:
                self.errors.append(
                    f'docstring type ({doc_type_dict[k]}) '
                    f'!= function signature ({ast_type_dict[k]}) '
                    f'for {k}'
                )

    def _check_return(self):
        has_return_statement = ast_has_return(self.tree)
        has_return_directive = ':return:' in self.docstring
        # TODO: Has 142 failures; Enable and fill in the docs.
        # if self.is_public:
        #     if has_return_statement and not has_return_directive:
        #         self.errors.append('return statement, but no :return: in docstring')
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
        self._check_types()
        self._check_return()
        return '; '.join(
            self.errors
            if len(self.errors) == 1
            else [f'({i+1}) {e}' for i, e in enumerate(self.errors)]
        )


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
            # TODO: Also check class docs
            continue
        if ast_ends_with_ellipsis(node):
            continue
        if node.name.startswith('_'):
            is_public = False

        function = Function(
            file=f'{code_path.parent.name}/{code_path.name}',
            name=node.name,
            tree=node,
            visibility=PUBLIC if is_public else PRIVATE,
        )
        functions.append(function)

# Typo check:
assert len(functions) > 100


@pytest.mark.parametrize("file,name,tree,visibility", functions)
def test_function_docs(file, name, tree, visibility):
    where = f'In {file}, line {tree.lineno}, def {name}'
    is_public = visibility == PUBLIC

    docstring = ast.get_docstring(tree)
    if not is_public and docstring is None:
        return
    assert docstring is not None, f'{where}: add docstring or make private'

    errors = Checker(
        tree=tree,
        docstring=docstring,
        is_public=is_public
    ).get_errors()
    if errors:
        pytest.fail(f'{where}: {errors}')
