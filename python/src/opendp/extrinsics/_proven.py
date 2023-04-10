import os
from opendp._data import make_proof_link


def proven(original_function=None, *, proof_path=None):
    """Decorator for functions that have an associated proof document.
    Locates the proof document and edits the docstring with a link.

    :param original_function: The function to decorate.
    :param proof_path: The path to the proof document _in the extrinsics module_.
    """
    def _proven(function):
        nonlocal proof_path
        if proof_path is None:
            proof_path = _find_unique(
                function.__name__ + ".tex", 
                os.path.dirname(os.path.abspath(__file__)))
        
        function.__doc__ = _insert_proof_into_docstring(function.__doc__, proof_path)
        return function

    if original_function:
        return _proven(original_function)

    return _proven

def _find_unique(name, path):
    """Find a unique file in a directory tree. Raise an error if there are multiple matches."""
    match = None
    for root, _, files in os.walk(path):
        if name in files:
            if match is not None:
                raise ValueError(f"More than one file named {name}. Please specify `proof_path = \"/relative/path/from/extrinsics/to/{name}\"`.")
            match = os.path.relpath(os.path.join(root, name), path)
    return match

def _split_docstring(docstring):
    """Split a docstring into a description and a list of parameters."""
    description = []
    params = []
    found_param = False
    for line in docstring:
        found_param |= line.startswith(":")
        if found_param:
            params.append(line)
        else:
            description.append(line)
    return description, params


def _insert_proof_into_docstring(docstring, proof_path):
    """Insert a link to the proof document into the docstring."""
    source_dir = os.path.dirname(os.path.abspath(__file__))

    proof_url = make_proof_link(source_dir, relative_path=proof_path, repo_path="python/src/opendp/extrinsics")
    proof_link = f"\n    [(Proof Document)]({proof_url})"

    description, params = _split_docstring(docstring.splitlines())

    proof_index = next((i for i, line in enumerate(
        description) if "# Proof Definition" in line), None)
    
    if proof_index is None:
        description.append("    # Proof Definition")
        proof_index = len(description)
    
    description.insert(proof_index, proof_link)
    return '\n'.join(description + params)