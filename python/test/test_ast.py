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


def check_directive_order(docstring):
    '''
    >>> check_directive_order("""
    ...     :param abc: The input
    ...     :paran xyz: Typo!
    ... """)
    'Unknown directives: :paran'
    
    >>> check_directive_order("""
    ...     :return: The output
    ...     :param xyz: The input
    ... """)
    'Directives :return:param are not in canonical order: (param(type)?)*(return)?(rtype)?(raises)*(example)?'
    '''
    directives = re.findall(r'^\s*(\:\w+:?)', docstring, re.MULTILINE)
    unknown_directives = set(directives) - {':param', ':rtype:', ':type', ':raises', ':example:', ':return:'}
    if unknown_directives:
        return f'Unknown directives: {", ".join(unknown_directives)}'
    
    order = ''.join(re.sub(r':$', '', d) for d in directives)
    # TODO: Has 169 failures if we require ":return" and ":rtype" together:
    # canonical_order = r'^(:param(:type)?)*(:return:rtype)?(:raises)*(:example)?$'

    # TODO: Has 139 failures if we require ":return" if ":rtype" is given:
    # (Low priority: from the type and the function name, the return may be clear enough)
    # canonical_order = r'^(:param(:type)?)*(:return(:rtype)?)?(:raises)*(:example)?$'

    # TODO: Has 28 failures if we require ":rtype" if ":return" is given:
    # canonical_order = r'^(:param(:type)?)*((:return)?:rtype)?(:raises)*(:example)?$'
    
    canonical_order = r'^(:param(:type)?)*(:return)?(:rtype)?(:raises)*(:example)?$'
    if not re.search(canonical_order, order):
        short_order = re.sub(r'[:$^]', '', canonical_order)
        return f'Directives {order} are not in canonical order: {short_order}'


def check_directive_continuity(docstring):
    '''
    >>> check_directive_continuity("""
    ...     :param xyz: The input
    ...     Surprise!
    ...     :return: The output
    ... """)
    'Found another directive after non-directive: :return: The output'
    '''
    directives_started = False
    directives_ended = False
    for line in docstring.split('\n'):
        line = line.strip()
        if not line:
            continue
        if line.startswith(':'):
            if directives_ended:
                return f'Found another directive after non-directive: {line}'
            directives_started = True
        elif directives_started:
            directives_ended = True


def check_doctest_continuity(docstring):
    '''
    >>> check_doctest_continuity("""
    ...     >>> 1 + 1
    ...     2
    ...     
    ...     >>> 2 + 2
    ...     4
    ... """)
    'Doctest should not be split by empty line above: >>> 2 + 2'

    >>> check_doctest_continuity("""
    ...     >>> assert True
    ...     
    ...     And then:
    ...     >>> print('hello!')
    ...     hello!
    ... """)
    "Doctest should have blank line above: >>> print('hello!')"
    '''
    in_code = False
    after_code = False
    in_text = False
    for line in docstring.split('\n'):
        line = line.strip()
        if line.startswith('>>>'):
            if after_code:
                return f'Doctest should not be split by empty line above: {line}'
            if in_text:
                return f'Doctest should have blank line above: {line}'
            in_code = True
            in_text = False
        elif in_code:
            if not line:
                in_code = False
                after_code = True
        elif line:
            after_code = False
            in_text = True
        else:
            in_text = False


def check_list_space(docstring):
    '''
    >>> check_list_space("""
    ...     Add a blank line after this one!
    ...     1. One thing
    ...     2. After another
    ... """)
    'Add a blank line above list that begins with: 1. One thing'

    >>> check_list_space("""
    ...     >>> 5.0 - 4.0
    ...     1.0
    ...
    ...     That should pass
    ...     1. but this should not!
    ... """)
    'Add a blank line above list that begins with: 1. but this should not!'

    >>> check_list_space("""
    ...     * This
    ...     * is fine,
    ...
    ...     but this is
    ...     * not ok!
    ... """)
    'Add a blank line above list that begins with: * not ok!'
    '''
    prev_is_text = False
    for line in docstring.split('\n'):
        line = line.strip()
        if prev_is_text and (line.startswith('1.') or line.startswith('* ')):
            return f'Add a blank line above list that begins with: {line}'
        prev_is_text = line and not (
            line.startswith('>>>')
            or line.startswith('...')
            or line.startswith('* ')
        )


class Checker():
    def __init__(self, name, tree, docstring, is_public, is_verbose):
        self.name = name
        self.tree = tree
        self.docstring = docstring
        self.is_public = is_public
        self.is_verbose = is_verbose

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
        
        if self.name == '__init__' and self.docstring:
            self.errors.append('Move docstring up to class')

    def _check_docstring(self):
        checks = [
            check_directive_order,
            check_directive_continuity,
            check_doctest_continuity,
            check_list_space,
        ]
        for check in checks:
            error = check(self.docstring)
            if error:
                self.errors.append(error)

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
        doc_type_dict = (
            # Get all documented parameters, even if there aren't types...
            {name: None for name in re.findall(r':param (\w+):', self.docstring)}
            | {
                # ... and then add types, where documented.
                # Allow either ":py:ref:" or ":ref:".
                # No reason to impose a standard if both work with the tools.
                k: re.sub(r'(?::py)?:ref:`(\w+)`', r'\1', v)
                for k, v in re.findall(r':type (\w+): *(.*)', self.docstring)
            }
        )

        ast_arg_annotation_dict = {
            arg.arg: getattr(arg, 'annotation', None)
            for arg in self.all_ast_args
        }
        ast_type_dict = {
            # "v and ..." ensures that None is passed through.
            # TODO: Has 32 failures if we require parameters with "Optional" types
            # to also have "Optional" in the  docstring.
            # k: v and ast.unparse(v)
            k: v and re.sub(r'Optional\[(.+)\]', r'\1', ast.unparse(v))
            for k, v in ast_arg_annotation_dict.items()
        }
        
        for k in doc_type_dict.keys() | ast_type_dict.keys():
            doc_type = doc_type_dict.get(k)
            ast_type = ast_type_dict.get(k)

            # TODO: Ignoring private functions, there are still 56 functions where 
            # either the documentation type or the annotation type is missing.
            if doc_type is None or ast_type is None:
                continue

            if doc_type != ast_type:
                self.errors.append(
                    f'docstring type ({doc_type}) '
                    f'!= function signature types ({ast_type}) '
                    f'for {k}'
                )

    def _check_return(self):
        has_return_statement = ast_has_return(self.tree)
        has_return_doc = ':return' in self.docstring or ':rtype' in self.docstring
        # TODO: Has 68 failures if "return" statements require either ":return" or ":rtype".
        # if self.is_public and has_return_statement and not has_return_doc:
        #     self.errors.append('return statement, but no :return or :rtype in docstring')
        if has_return_doc and not has_return_statement:
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
        if not self.errors:
            return
        if self.is_verbose:
            # split across multiple lines for readability:
            return '\n' + '\n'.join(self.errors)
        # else all on one line, though it may be truncated:
        return '; '.join(
            self.errors
            if len(self.errors) == 1
            else [f'({i+1}) {e}' for i, e in enumerate(self.errors)]
        )


PUBLIC = 'public'
PRIVATE = 'private'

class CodeObj(NamedTuple):
    file: str
    name: str
    tree: ast.AST
    visibility: str  # str rather than bool so the pytest report is more readable.


classes = []
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
        if not isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            continue
        if ast_ends_with_ellipsis(node):
            continue
        if node.name.startswith('_'):
            is_public = False

        code_obj = CodeObj(
            file=f'{code_path.parent.name}/{code_path.name}',
            name=node.name,
            tree=node,
            visibility=PUBLIC if is_public else PRIVATE,
        )
        if isinstance(node, ast.FunctionDef):
            functions.append(code_obj)
        if isinstance(node, ast.ClassDef):
            classes.append(code_obj)

# Typo check:
assert len(functions) > 100


@pytest.mark.parametrize("file,name,tree,visibility", functions)
def test_function_docs(file, name, tree, visibility, pytestconfig):
    where = f'In {file}, line {tree.lineno}, def {name}'
    is_public = visibility == PUBLIC

    docstring = ast.get_docstring(tree)
    if not is_public and docstring is None:
        return
    assert docstring is not None, f'{where}: add docstring or make private'

    errors = Checker(
        name=name,
        tree=tree,
        docstring=docstring,
        is_public=is_public,
        is_verbose=pytestconfig.getoption("verbose") > 0
    ).get_errors()
    if errors:
        pytest.fail(f'{where}: {errors}')

@pytest.mark.parametrize("file,name,tree,visibility", classes)
def test_class_docs(file, name, tree, visibility):
    where = f'In {file}, line {tree.lineno}, class {name}'
    is_public = visibility == PUBLIC

    docstring = ast.get_docstring(tree)
    if not is_public and docstring is None:
        return
    assert docstring is not None, f'{where}: add docstring or make private'