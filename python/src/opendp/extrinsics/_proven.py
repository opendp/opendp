import os
import re
import inspect
from opendp._data import make_proof_link

proof_doc_re = re.compile(r'\[\(Proof Document\)\]\(([^)]+)\)')


def proven(function):
    """Decorator for functions that have an associated proof document.
    Locates the proof document and edits the docstring with a link.
    """

    for match in proof_doc_re.finditer(function.__doc__):
        a, b = match.span(1)

        # extract the path to the proof document
        matched_path = function.__doc__[a:b]
        source_dir = os.path.dirname(inspect.getfile(function))
        absolute_proof_path = os.path.abspath(
            os.path.join(source_dir, matched_path))

        # split the path at the extrinsics directory
        extrinsics_path = os.path.dirname(__file__)
        relative_proof_path = os.path.relpath(
            absolute_proof_path, extrinsics_path)

        # create the link
        proof_url = make_proof_link(
            extrinsics_path, 
            relative_path=relative_proof_path, 
            repo_path="python/src/opendp/extrinsics")

        # replace the path with the link
        function.__doc__ = function.__doc__[:a] + proof_url + function.__doc__[b:]

    return function


if __name__ == "__main__":

    dummy_path = os.path.join(os.path.dirname(__file__), 'dummy.tex')
    open(dummy_path, 'w').close()

    @proven
    def make_test(a, b):
        """A dummy function for testing the proven decorator.

        [(Proof Document)](dummy.tex))]

        :param a: The first parameter.
        :param b: The second parameter."""
        _ = a, b

    print(make_test.__doc__)
    os.remove(dummy_path)
