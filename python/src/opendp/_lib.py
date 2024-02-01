import ctypes
import os
import platform
import re
import sys
from typing import Dict, List, Optional, Any
import re


# list all acceptable alternative types for each default type
ATOM_EQUIVALENCE_CLASSES: Dict[str, List[str]] = {
    'i32': ['u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64', 'usize'],
    'f64': ['f32', 'f64'],
    'bool': ['bool'],
    'AnyMeasurementPtr': ['AnyMeasurementPtr', 'AnyMeasurement'],
    'AnyTransformationPtr': ['AnyTransformationPtr'],
    'ExtrinsicObject': ['ExtrinsicObject']
}


def _load_library():
    lib_dir = os.environ.get("OPENDP_LIB_DIR", os.path.join(os.path.dirname(os.path.abspath(__file__)), "lib"))
    if not os.path.exists(lib_dir): # pragma: no cover
        # fall back to default location of binaries in a developer install
        build_dir = 'debug' if os.environ.get('OPENDP_TEST_RELEASE', "false") == "false" else 'release'
        lib_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), *['..'] * 3, 'rust', 'target', build_dir)

    if os.path.exists(lib_dir):
        # Mapping of Python platform to library name format
        platform_to_name_template = {
            "darwin": "libopendp{}.dylib",
            "linux": "libopendp{}.so",
            "win32": "opendp{}.dll",
        }
        # Mapping of Python platform/machine to Rust architecture.
        platform_machine_to_architecture = {
            ("win32", "AMD64"): "x86_64",
            # No way to build this yet ("win32", "ARM64"): "aarch64",
            ("darwin", "x86_64"): "x86_64",
            ("darwin", "arm64"): "aarch64",
            ("linux", "x86_64"): "x86_64",
            ("linux", "aarch64"): "aarch64",
        }

        name_template = platform_to_name_template.get(sys.platform)
        if name_template is None:
            raise Exception("Platform not supported", sys.platform)
        architecture = platform_machine_to_architecture.get((sys.platform, platform.machine()))
        if architecture is None:
            raise Exception("Machine not supported", sys.platform, platform.machine())

        def get_lib_path(name_template, architecture):
            suffix = f"-{architecture}" if architecture is not None else ""
            name = name_template.format(suffix)
            return os.path.join(lib_dir, name)

        # First try name with architecture
        lib_path = get_lib_path(name_template, architecture)
        if not os.path.exists(lib_path):
            # Fall back to name without architecture (happens on darwin, which has fat binaries, and developer installs)
            lib_path = get_lib_path(name_template, None)

        try:
            return ctypes.cdll.LoadLibrary(lib_path)
        except Exception as e:
            raise Exception("Unable to load OpenDP shared library", lib_path, e)

    elif os.environ.get('OPENDP_HEADLESS', "false") != "false":
        return None

    else:
        raise ValueError("Unable to find lib directory. Consider setting OPENDP_LIB_DIR to a valid directory.")


lib = _load_library()


np_csprng: "np.random.Generator" = None # type: ignore[assignment]
try:
    import numpy as np  # type: ignore[import-not-found]
    from randomgen import UserBitGenerator  # type: ignore[import]

    buffer_len = 1024
    buffer = np.empty(buffer_len, dtype=np.uint64)
    buffer_ptr = ctypes.cast(buffer.ctypes.data, ctypes.POINTER(ctypes.c_uint8))
    buffer_pos = buffer_len

    def next_raw(_voidp):
        global buffer_pos
        if buffer_len == buffer_pos:
            from opendp._data import fill_bytes

            # there are 8x as many u8s as there are u64s
            if not fill_bytes(buffer_ptr, buffer_len * 8): # pragma: no cover
                from opendp.mod import OpenDPException
                raise OpenDPException("FailedFunction", "Failed to sample from CSPRNG")
            buffer_pos = 0

        out = buffer[buffer_pos]
        buffer_pos += 1
        return int(out)

    np_csprng = np.random.Generator(bit_generator=UserBitGenerator(next_raw))

except ImportError:  # pragma: no cover
    pass

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
        except (ImportError, TypeError):
            # ImportError: sys.meta_path is None, Python is likely shutting down
            pass


class AnyQueryable(ctypes.Structure):
    pass  # Opaque struct


class FfiSlicePtr(ctypes.POINTER(FfiSlice)): # type: ignore[misc]
    _type_ = FfiSlice
    _dependencies: Dict[Any, Any] = {}  # TODO: Tighten this

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
        try: # pragma: no cover
            from opendp._data import extrinsic_object_free
            extrinsic_object_free(self)
        except (ImportError, TypeError):
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass

# def _str_to_c_char_p(s: Optional[str]) -> Optional[bytes]:
#     return s and s.encode("utf-8")
def _c_char_p_to_str(s: Optional[bytes]) -> Optional[str]:
    if s is not None:
        return s.decode("utf-8")
    return None # pragma: no cover


def unwrap(result, type_) -> Any:
    from opendp.core import _error_free
    from opendp.mod import OpenDPException

    if not isinstance(result, FfiResult):
        return result # pragma: no cover

    if result.tag == 0:
        return ctypes.cast(result.payload.Ok, type_)

    err = result.payload.Err
    err_contents = err.contents
    variant = _c_char_p_to_str(err_contents.variant) or "Core"
    message = _c_char_p_to_str(err_contents.message)
    backtrace = _c_char_p_to_str(err_contents.backtrace)

    if not _error_free(err):
        raise OpenDPException("Failed to free error.")

    # Rust stack traces follow from here:
    raise OpenDPException(variant, message, backtrace)


def versioned(function):
    """Decorator to update version numbers in docstrings.
    This is shown in the help(*) and Sphinx documentation (like docs.opendp.org)."""

    version = get_opendp_version()
    channel = get_channel(version)

    if channel != "dev": # pragma: no cover
        # docs.rs keeps all releases, so we can use the full version.
        function.__doc__ = function.__doc__.replace(
            "https://docs.rs/opendp/latest/", f"https://docs.rs/opendp/{version}/"
        )

        docs_ref = get_docs_ref(version)
        function.__doc__ = function.__doc__.replace(
            "https://docs.opendp.org/en/latest/",
            f"https://docs.opendp.org/en/{docs_ref}/",
        )

    return function


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


        # split the path at the extrinsics directory
        extrinsics_path = os.path.join(os.path.dirname(__file__), "_extrinsics")
        relative_proof_path = os.path.relpath(absolute_proof_path, extrinsics_path)

        # create the link
        proof_url = make_proof_link(
            extrinsics_path,
            relative_path=relative_proof_path,
            repo_path="python/src/opendp/_extrinsics",
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
        raise ValueError(f"{absolute_path} does not exist!")

    # link to the pdf, not the tex
    relative_path = relative_path.replace(".tex", ".pdf")

    # link from sphinx and rustdoc to latex
    sphinx_port = os.environ.get("OPENDP_SPHINX_PORT", None)
    if sphinx_port is not None:
        proof_uri = f"http://localhost:{sphinx_port}"  # pragma: no cover

    else:
        # find the docs uri
        docs_uri = os.environ.get("OPENDP_REMOTE_SPHINX_URI", "https://docs.opendp.org")

        # find the version
        version = get_opendp_version()
        docs_ref = get_docs_ref(version)

        proof_uri = f"{docs_uri}/en/{docs_ref}"

    return f"{proof_uri}/proofs/{repo_path}/{relative_path}"


def get_opendp_version():
    import sys

    if sys.version_info >= (3, 8):
        import importlib.metadata

        try:
            return unmangle_py_version(importlib.metadata.version("opendp"))
        except importlib.metadata.PackageNotFoundError:
            return get_opendp_version_from_file()
    else: # pragma: no cover
        import pkg_resources

        try:
            return unmangle_py_version(pkg_resources.get_distribution("opendp").version)
        except pkg_resources.DistributionNotFound:
            return get_opendp_version_from_file()


def unmangle_py_version(py_version):
    '''
    Python mangles pre-release versions like "X.Y.Z-nightly.NNN.M" into "X.Y.ZaNNN00M", but the docs use
    the original format, so we need to unmangle for links to work.
    There are more variations possible, but we only need to handle X.Y.Z-dev0, X.Y.Z-aNNN00M, X.Y.Z-bNNN00M, X.Y.Z
    
    >>> unmangle_py_version('0.9.0.dev0')
    '0.9.0-dev'
    >>> unmangle_py_version('other')
    'other'
    '''
    if py_version.endswith(".dev0"):
        return f"{py_version[:-5]}-dev"
    match = re.match(r"^(\d+\.\d+\.\d+)(?:([ab])(\d{8})(\d{3}))?$", py_version)
    if match: # pragma: no cover
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
    version_file = os.path.join(os.path.dirname(os.path.abspath(__file__)), *['..'] * 3, 'VERSION')
    return open(version_file, 'r').read().strip()


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
