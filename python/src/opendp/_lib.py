import ctypes
import os
import sys
from typing import Optional, Any
import re


# list all acceptable alternative types for each default type
ATOM_EQUIVALENCE_CLASSES = {
    'i32': ['u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64', 'usize'],
    'f64': ['f32', 'f64'],
    'bool': ['bool'],
    'AnyMeasurementPtr': ['AnyMeasurementPtr', 'AnyMeasurement'],
    'AnyTransformationPtr': ['AnyTransformationPtr'],
}

lib_dir = os.environ.get("OPENDP_LIB_DIR", os.path.join(os.path.dirname(os.path.abspath(__file__)), "lib"))
if not os.path.exists(lib_dir):
    # fall back to default location of binaries in a developer install
    build_dir = 'debug' if os.environ.get('OPENDP_TEST_RELEASE', "false") == "false" else 'release'
    lib_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), *['..'] * 3, 'rust', 'target', build_dir)

if os.path.exists(lib_dir):
    platform_to_name = {
        "darwin": "libopendp.dylib",
        "linux": "libopendp.so",
        "win32": "opendp.dll",
    }
    if sys.platform not in platform_to_name:
        raise Exception("Platform not supported", sys.platform)
    lib_name = platform_to_name[sys.platform]

    lib = ctypes.cdll.LoadLibrary(os.path.join(lib_dir, lib_name))

elif os.environ.get('OPENDP_HEADLESS', "false") != "false":
    lib = None

else:
    raise ValueError("Unable to find lib directory. Consider setting OPENDP_LIB_DIR to a valid directory.")


# This enables backtraces in Rust by default.
# It can be disabled by setting RUST_BACKTRACE=0.
# Binary searches disable backtraces for performance reasons.
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


class BoolPtr(ctypes.POINTER(ctypes.c_bool)):
    _type_ = ctypes.c_bool


class AnyObjectPtr(ctypes.POINTER(AnyObject)):
    _type_ = AnyObject

    def __del__(self):
        from opendp._data import object_free
        object_free(self)


class AnyQueryable(ctypes.Structure):
    pass  # Opaque struct


class FfiSlicePtr(ctypes.POINTER(FfiSlice)):
    _type_ = FfiSlice
    _dependencies = {}

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


# def _str_to_c_char_p(s: Optional[str]) -> Optional[bytes]:
#     return s and s.encode("utf-8")
def _c_char_p_to_str(s: Optional[bytes]) -> Optional[str]:
    return s and s.decode("utf-8")


def unwrap(result, type_) -> Any:
    from opendp.core import _error_free
    from opendp.mod import OpenDPException

    if not isinstance(result, FfiResult):
        return result

    if result.tag == 0:
        return ctypes.cast(result.payload.Ok, type_)

    err = result.payload.Err
    err_contents = err.contents
    variant = _c_char_p_to_str(err_contents.variant)
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

    if version != "0.0.0+development":
        function.__doc__ = function.__doc__.replace(
            "https://docs.rs/opendp/latest/", f"https://docs.rs/opendp/{version}/"
        )

        function.__doc__ = function.__doc__.replace(
            "https://docs.opendp.org/en/latest/",
            f"https://docs.opendp.org/en/v{version}/",
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
        extrinsics_path = os.path.join(os.path.dirname(__file__), "extrinsics")
        relative_proof_path = os.path.relpath(absolute_proof_path, extrinsics_path)

        # create the link
        proof_url = make_proof_link(
            extrinsics_path,
            relative_path=relative_proof_path,
            repo_path="python/src/opendp/extrinsics",
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
        proof_uri = f"http://localhost:{sphinx_port}"

    else:
        # find the docs uri
        docs_uri = os.environ.get("OPENDP_REMOTE_SPHINX_URI", "https://docs.opendp.org")

        # find the version
        version = get_opendp_version()
        version_segment = "latest" if version == "0.0.0+development" else "v" + version

        proof_uri = f"{docs_uri}/en/{version_segment}"

    return f"{proof_uri}/proofs/{repo_path}/{relative_path}"


def get_opendp_version():
    import sys

    if sys.version_info >= (3, 8):
        import importlib.metadata

        try:
            return unmangle_py_version(importlib.metadata.version("opendp"))
        except importlib.metadata.PackageNotFoundError:
            return get_opendp_version_from_file()
    else:
        import pkg_resources

        try:
            return unmangle_py_version(pkg_resources.get_distribution("opendp").version)
        except pkg_resources.DistributionNotFound:
            return get_opendp_version_from_file()


def unmangle_py_version(version):
    # Python mangles pre-release versions like "X.Y.Z-rc.N" into "X.Y.ZrcN", but the docs use the correct format,
    # so we need to unmangle so the links will work.
    match = re.match(r"^(\d+\.\d+\.\d+)rc(\d+)$", version)
    if match:
        version = f"{match.group(1)}-rc.{match.group(2)}"
    return version


def get_opendp_version_from_file():
    # If the package isn't installed (eg when we're building docs), we can't get the version from metadata,
    # so fall back to the version file.
    version_file = os.path.join(os.path.dirname(os.path.abspath(__file__)), *['..'] * 3, 'VERSION')
    return open(version_file, 'r').read().strip()
