import ctypes
from typing import MutableMapping
import os
from pathlib import Path
import re
from typing import Optional, Any
import importlib


# list all acceptable alternative types for each default type
ATOM_EQUIVALENCE_CLASSES: MutableMapping[str, list[str]] = {
    'i32': ['u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64', 'usize'],
    'f64': ['f32', 'f64'],
    'bool': ['bool'],
    'AnyMeasurementPtr': ['AnyMeasurementPtr', 'AnyMeasurement'],
    'AnyTransformationPtr': ['AnyTransformationPtr'],
    'ExtrinsicObject': ['ExtrinsicObject']
}


def _load_library():
    default_lib_dir = Path(__file__).absolute().parent / "lib"
    lib_dir = Path(os.environ.get("OPENDP_LIB_DIR", default_lib_dir))
    if not lib_dir.exists():
        # fall back to default location of binaries in a developer install
        build_dir = 'debug' if os.environ.get('OPENDP_TEST_RELEASE', "false") == "false" else 'release'
        lib_dir = Path(__file__).parent / ".." / ".." / ".." / 'rust' / 'target' / build_dir  # pragma: no cover

    if lib_dir.exists():
        from importlib.machinery import EXTENSION_SUFFIXES
        suffixes = EXTENSION_SUFFIXES + [".dylib"]

        lib_dir_file_names = [
            str(p.name) for p in lib_dir.iterdir() 
            if "opendp" in p.name and "derive" not in p.name
            and any(p.name.endswith(suffix) for suffix in suffixes)
        ]
        
        if len(lib_dir_file_names) != 1:
            raise Exception(f"Expected exactly one binary to be present in {lib_dir}. Got: {lib_dir_file_names}")  # pragma: no cover
        
        lib_path = lib_dir / lib_dir_file_names[0]
        try:
            return ctypes.cdll.LoadLibrary(str(lib_path)), lib_path
        except Exception as e: # pragma: no cover
            raise Exception("Unable to load OpenDP shared library", lib_path, e)

    elif os.environ.get('OPENDP_HEADLESS', "false") != "false":
        return None, None

    else:
        raise ValueError("Unable to find lib directory. Consider setting OPENDP_LIB_DIR to a valid directory.")  # pragma: no cover
    

lib, lib_path = _load_library()


install_names = {
    'sklearn': 'scikit-learn'
}


def import_optional_dependency(name, raise_error=True):
    '''
    Imports optional dependency, or explains that it is required.
    '''
    try:
        return importlib.import_module(name)
    except ImportError: # pragma: no cover
        if raise_error:
            root_name = name.split(".")[0]
            install_name = install_names.get(root_name) or root_name
            raise ImportError(f'The optional install {install_name} is required for this functionality')
        return None


_np_csprng = None
_buffer_pos = 0 # TODO: Make this into a class rather than using ad-hoc globals.
def get_np_csprng():
    global _np_csprng
    global _buffer_pos
    if _np_csprng is not None:
        return _np_csprng

    randomgen = import_optional_dependency('randomgen')
    np = import_optional_dependency('numpy')

    buffer_len = 1024
    buffer = np.empty(buffer_len, dtype=np.uint64)
    buffer_ptr = ctypes.cast(buffer.ctypes.data, ctypes.POINTER(ctypes.c_uint8))
    _buffer_pos = buffer_len

    def next_raw(_voidp):
        global _buffer_pos
        if buffer_len == _buffer_pos:
            from opendp._data import fill_bytes

            # there are 8x as many u8s as there are u64s
            if not fill_bytes(buffer_ptr, buffer_len * 8): # pragma: no cover
                from opendp.mod import OpenDPException
                raise OpenDPException("FailedFunction", "Failed to sample from CSPRNG")
            _buffer_pos = 0

        out = buffer[_buffer_pos]
        _buffer_pos += 1
        return int(out)

    _np_csprng = np.random.Generator(bit_generator=randomgen.UserBitGenerator(next_raw)) # type:ignore
    return _np_csprng


# This enables backtraces in Rust by default.
# It can be disabled by setting RUST_BACKTRACE=0.
if "RUST_BACKTRACE" not in os.environ:
    os.environ["RUST_BACKTRACE"] = "1"


class FfiSlice(ctypes.Structure):
    _fields_ = [
        ("ptr", ctypes.c_void_p),
        ("len", ctypes.c_size_t),
    ]


class AnyObject(ctypes.Structure):
    pass  # Opaque struct


class AnyMeasurement(ctypes.Structure):
    pass  # Opaque struct


class AnyTransformation(ctypes.Structure):
    pass  # Opaque struct


class AnyDomain(ctypes.Structure):
    pass  # Opaque struct


class AnyMetric(ctypes.Structure):
    pass  # Opaque struct


class AnyMeasure(ctypes.Structure):
    pass  # Opaque struct


class AnyFunction(ctypes.Structure):
    pass  # Opaque struct


class BoolPtr(ctypes.POINTER(ctypes.c_bool)): # type: ignore[misc]
    _type_ = ctypes.c_bool


class AnyObjectPtr(ctypes.POINTER(AnyObject)): # type: ignore[misc]
    _type_ = AnyObject

    def __del__(self):
        try:
            from opendp._data import object_free
            object_free(self)
        except (ImportError, TypeError): # pragma: no cover
            # ImportError: sys.meta_path is None, Python is likely shutting down
            pass


class AnyQueryable(ctypes.Structure):
    pass  # Opaque struct


class FfiSlicePtr(ctypes.POINTER(FfiSlice)): # type: ignore[misc]
    _type_ = FfiSlice
    _dependencies: MutableMapping = {}  # TODO: Tighten this

    def depends_on(self, *args):
        """Extends the memory lifetime of args to the lifetime of self."""
        FfiSlicePtr._dependencies.setdefault(id(self), []).extend(args)

    def __del__(self):
        """When self is deleted, stop keeping dependencies alive by freeing the reference."""
        FfiSlicePtr._dependencies.pop(id(self), None)


class FfiError(ctypes.Structure):
    _fields_ = [
        ("variant", ctypes.c_char_p),
        ("message", ctypes.c_char_p),
        ("backtrace", ctypes.c_char_p),
    ]


class FfiResultPayload(ctypes.Union):
    _fields_ = [
        ("Ok", ctypes.c_void_p),
        ("Err", ctypes.POINTER(FfiError)),
    ]


class FfiResult(ctypes.Structure):
    _fields_ = [
        ("tag", ctypes.c_uint32),
        ("payload", FfiResultPayload),
    ]


class ExtrinsicObject(ctypes.Structure):
    _fields_ = [
        ("ptr", ctypes.py_object),
        ("count", ctypes.CFUNCTYPE(ctypes.c_bool, ctypes.py_object, ctypes.c_bool))
    ]


class ExtrinsicObjectPtr(ctypes.POINTER(ExtrinsicObject)): # type: ignore[misc]
    _type_ = ExtrinsicObject

    def __del__(self):
        try:
            from opendp._data import extrinsic_object_free
            extrinsic_object_free(self)
        except (ImportError, TypeError): # pragma: no cover
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass


# The output type cannot be an `ctypes.POINTER(FfiResult)` due to:
#   https://bugs.python.org/issue5710#msg85731
#                                 (output         , input       )
CallbackFnValue = ctypes.CFUNCTYPE(ctypes.c_void_p, AnyObjectPtr)

class CallbackFn(ctypes.Structure):
    _fields_ = [
        ("callback", CallbackFnValue),
        ("lifeline", ExtrinsicObject)
    ]

class CallbackFnPtr(ctypes.POINTER(CallbackFn)): # type: ignore[misc]
    _type_ = CallbackFn

# def _str_to_c_char_p(s: Optional[str]) -> Optional[bytes]:
#     return s and s.encode("utf-8")
def _c_char_p_to_str(s: Optional[bytes]) -> Optional[str]:
    ''''''
    if s is not None:
        return s.decode("utf-8")
    # TODO: Unused: would it indicate a problem if we did start hitting this?
    return None # pragma: no cover


def unwrap(result, type_) -> Any:
    from opendp.core import _error_free
    from opendp.mod import OpenDPException

    if not isinstance(result, FfiResult):
        # TODO: Unused: would it indicate a problem if we did start hitting this?
        return result # pragma: no cover

    if result.tag == 0:
        return ctypes.cast(result.payload.Ok, type_)

    err = result.payload.Err
    err_contents = err.contents
    variant = _c_char_p_to_str(err_contents.variant) or "Core"
    message = _c_char_p_to_str(err_contents.message)
    backtrace = _c_char_p_to_str(err_contents.backtrace)

    if not _error_free(err):
        raise OpenDPException("Failed to free error.")  # pragma: no cover

    # Rust stack traces follow from here:
    pl = import_optional_dependency('polars', raise_error=False)
    if pl is not None:
        from opendp.mod import _EXPECTED_POLARS_VERSION
        if 'polars' in str(message).lower() and pl.__version__ != _EXPECTED_POLARS_VERSION:
            message = f'Installed python polars version ({pl.__version__}) != expected version ({_EXPECTED_POLARS_VERSION}). {message}' # pragma: no cover
    raise OpenDPException(variant, message, backtrace)


proof_doc_re = re.compile(r"\[\(Proof Document\)\]\(([^)]+)\)")


def proven(function):
    """Decorator for functions that have an associated proof document.
    Locates the proof document and edits the docstring with a link.
    """
    import inspect

    for match in proof_doc_re.finditer(function.__doc__):
        a, b = match.span(1)

        # extract the path to the proof document
        matched_path = function.__doc__[a:b]
        source_dir = os.path.dirname(inspect.getfile(function))
        absolute_proof_path = os.path.abspath(os.path.join(source_dir, matched_path))

        # split the path at the extras directory
        extras_path = os.path.join(os.path.dirname(__file__), "extras")
        relative_proof_path = os.path.relpath(absolute_proof_path, extras_path)

        # create the link
        proof_url = make_proof_link(
            extras_path,
            relative_path=relative_proof_path,
            repo_path="python/src/opendp/extras",
        )

        # replace the path with the link
        function.__doc__ = function.__doc__[:a] + proof_url + function.__doc__[b:]

    return function


def make_proof_link(
    source_dir,
    relative_path,
    repo_path,
) -> str:
    # construct absolute path
    absolute_path = os.path.join(source_dir, relative_path)

    if not os.path.exists(absolute_path):
        raise ValueError(f"{absolute_path} does not exist!")  # pragma: no cover

    # link to the pdf, not the tex
    relative_path = relative_path.replace(".tex", ".pdf")

    # link from sphinx and rustdoc to latex
    sphinx_port = os.environ.get("OPENDP_SPHINX_PORT", None)
    if sphinx_port is not None:
        proof_uri = f"http://localhost:{sphinx_port}" # pragma: no cover

    else:
        # find the docs uri
        docs_uri = os.environ.get("OPENDP_REMOTE_SPHINX_URI", "https://docs.opendp.org")

        # find the version
        version = get_opendp_version()
        docs_ref = get_docs_ref(version)

        proof_uri = f"{docs_uri}/en/{docs_ref}"

    return f"{proof_uri}/proofs/{repo_path}/{relative_path}"


def get_opendp_version():
    import importlib.metadata

    try:
        return unmangle_py_version(importlib.metadata.version("opendp"))
    except importlib.metadata.PackageNotFoundError: # pragma: no cover
        return get_opendp_version_from_file()


def unmangle_py_version(py_version):
    '''
    Python mangles pre-release versions like "X.Y.Z-nightly.NNN.M" into "X.Y.ZaNNN00M", but the docs use
    the original format, so we need to unmangle for links to work.
    There are more variations possible, but we only need to handle X.Y.Z-dev0, X.Y.Z-aNNN00M, X.Y.Z-bNNN00M, X.Y.Z
    
    >>> unmangle_py_version('0.9.0')
    '0.9.0'
    >>> unmangle_py_version('0.9.0.dev0')
    '0.9.0-dev'
    >>> unmangle_py_version('0.9.0a11111111001')
    '0.9.0-nightly.11111111.1'
    >>> unmangle_py_version('0.9.0b11111111001')
    '0.9.0-beta.11111111.1'
    >>> unmangle_py_version('other')
    'other'
    '''
    if py_version.endswith(".dev0"):
        return f"{py_version[:-5]}-dev"
    match = re.match(r"^(\d+\.\d+\.\d+)(?:([ab])(\d{8})(\d{3}))?$", py_version)
    if match:
        base = match.group(1)
        py_tag = match.group(2)
        if not py_tag:
            return base
        channel = "nightly" if py_tag == "a" else "beta"
        date = match.group(3)
        counter = int(match.group(4))
        return f"{base}-{channel}.{date}.{counter}"
    return py_version


def get_opendp_version_from_file():
    '''
    If the package isn't installed (eg when we're building docs), we can't get the version from metadata,
    so fall back to the version file.

    >>> import re
    >>> assert re.match(r'\\d+\\.\\d+\\.\\d+', get_opendp_version_from_file())
    '''
    version_path = Path(__file__).parent.parent.parent.parent / 'VERSION'
    return version_path.read_text().strip()


def get_docs_ref(version):
    '''
    >>> get_docs_ref('0.0.0')
    'v0.0.0'
    >>> get_docs_ref('0.0.0-dev')
    'latest'
    >>> get_docs_ref('0.0.0-nightly')
    'nightly'
    >>> get_docs_ref('0.0.0-surprise')
    'unknown'
    '''
    channel = get_channel(version)
    if channel == "stable":
        return f"v{version}"  # For stable, we have tags.
    elif channel == "dev":
        return "latest"  # Will be replaced by the @versioned decorator.
    else:
        return channel  # For beta & nightly, we don't have tags, just a single branch.


def get_channel(version):
    match = re.match(r"^(\d+\.\d+\.\d+)(?:-(dev|nightly|beta)(?:\.(.+))?)?$", version)
    if match:
        channel = match.group(2)
        return channel or "stable"
    return "unknown"

def indent(text):
    '''
    Indents the lines after the first line of a multiline string.
    Used for nested reprs.

    >>> print(indent('object(\\nfield = 123)'))
    object(
        field = 123)
    '''
    lines = text.split('\n')
    first, rest = lines[0], lines[1:]
    spaces = ' ' * 4
    indented_rest = [f'\n{spaces}{line}' for line in rest]
    return first + ''.join(indented_rest)