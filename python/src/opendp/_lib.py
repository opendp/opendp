import ctypes
import os
from pathlib import Path
import re
from typing import Optional, Any
import importlib


# list all acceptable alternative types for each default type
ATOM_EQUIVALENCE_CLASSES: dict[str, list[str]] = {
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
        lib_dir_file_names = [p for p in lib_dir.iterdir() if p.suffix in {".so", ".dylib", ".dll"}]
        if len(lib_dir_file_names) != 1:
            raise Exception(f"Expected exactly one binary to be present. Got: {lib_dir_file_names}")
        
        lib_path = lib_dir / lib_dir_file_names[0]
        try:
            return ctypes.cdll.LoadLibrary(str(lib_path)), lib_path
        except Exception as e:
            raise Exception("Unable to load OpenDP shared library", lib_path, e)

    elif os.environ.get('OPENDP_HEADLESS', "false") != "false":
        return None, None

    else:
        raise ValueError("Unable to find lib directory. Consider setting OPENDP_LIB_DIR to a valid directory.")
    

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
    except ImportError:
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


pl = import_optional_dependency("polars", raise_error=False)
if pl is not None:
    @pl.api.register_expr_namespace("dp")
    class DPNamespace(object):
        def __init__(self, expr):
            self.expr = expr

        def laplace(self, scale=None):
            """Add Laplace noise to the expression.

            If scale is None it is filled by `global_scale` in :py:func:`opendp.measurement.make_private_lazyframe`.

            :param scale: Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2). 
            """
            scale = float("nan") if scale is None else scale
            return pl.plugins.register_plugin_function(
                plugin_path=lib_path,
                function_name="laplace",
                kwargs={"scale": scale},
                args=self.expr,
                is_elementwise=True,
            )

        def sum(self, bounds, scale=None):
            """Compute the differentially private sum.

            If scale is None it is filled by `global_scale` in :py:func:`opendp.measurement.make_private_lazyframe`.

            :param bounds: The bounds of the input data.
            :param scale: Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2). 
            """
            return self.expr.clip(*bounds).sum().dp.laplace(scale)
        
        
        def mean(self, bounds, scale=None):
            """Compute the differentially private mean.

            The amount of noise to be added to the sum is determined by the scale.
            If scale is None it is filled by `global_scale` in :py:func:`opendp.measurement.make_private_lazyframe`.

            :param bounds: The bounds of the input data.
            :param scale: Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2). 
            """
            return self.expr.dp.sum(bounds, scale) / pl.len()


        def _discrete_quantile_score(self, alpha, candidates):
            """Score the utility of each candidate for representing the true quantile.
            
            Candidates closer to the true quantile are assigned scores closer to zero.
            Lower scores are better.
            
            :param alpha: a value in $[0, 1]$. Choose 0.5 for median
            :param candidates: Set of possible quantiles to evaluate the utility of.
            """
            return pl.plugins.register_plugin_function(
                plugin_path=lib_path,
                function_name="discrete_quantile_score",
                kwargs={"alpha": alpha, "candidates": candidates},
                args=self.expr,
                returns_scalar=True,
            )

        def _report_noisy_max_gumbel(self, optimize, scale=None):
            """Report the argmax or argmin after adding Gumbel noise.
            
            The scale calibrates the level of entropy when selecting an index.
            
            :param optimize: Distinguish between argmax and argmin.
            :param scale: Noise scale parameter for the Gumbel distribution.
            """
            return pl.plugins.register_plugin_function(
                plugin_path=lib_path,
                function_name="report_noisy_max_gumbel",
                kwargs={"optimize": optimize, "scale": scale},
                args=self.expr,
                is_elementwise=True,
            )
        
        def _index_candidates(self, candidates):
            """Index into a candidate set. 

            Typically used after `rnm_gumbel` to map selected indices to candidates.
            
            :param candidates: The values that each selected index corresponds to.
            """
            return pl.plugins.register_plugin_function(
                plugin_path=lib_path,
                function_name="index_candidates",
                kwargs={"candidates": candidates},
                args=self.expr,
                is_elementwise=True,
            )

    
        def quantile(self, alpha, candidates, scale=None):
            """Compute a differentially private quantile.
            
            The scale calibrates the level of entropy when selecting a candidate.
            
            :param alpha: a value in $[0, 1]$. Choose 0.5 for median
            :param candidates: Potential quantiles to select from.
            :param scale: How much noise to add to the scores of candidate.
            """
            dq_score = self.expr.dp._discrete_quantile_score(alpha, candidates)
            noisy_idx = dq_score.dp._report_noisy_max_gumbel("min", scale)
            return noisy_idx.dp._index_candidates(candidates)

        
        def median(self, candidates, scale=None):
            """Compute a differentially private median.
            
            The scale calibrates the level of entropy when selecting a candidate.
            
            :param candidates: Potential quantiles to select from.
            :param scale: How much noise to add to the scores of candidate.
            """
            return self.expr.dp.quantile(0.5, candidates, scale)

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
    _dependencies: dict[Any, Any] = {}  # TODO: Tighten this

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
        except (ImportError, TypeError):
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass

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
        raise OpenDPException("Failed to free error.")

    # Rust stack traces follow from here:
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
    except importlib.metadata.PackageNotFoundError:
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