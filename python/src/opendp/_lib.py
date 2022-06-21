import ctypes
import os
import sys
from typing import Optional, Any

# list all acceptable alternative types for each default type
ATOM_EQUIVALENCE_CLASSES = {
    'i32': ['u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64', 'usize'],
    'f64': ['f32', 'f64'],
    'bool': ['bool'],
    'AnyMeasurementPtr': ['AnyMeasurementPtr'],
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


class BoolPtr(ctypes.POINTER(ctypes.c_bool)):
    _type_ = ctypes.c_bool


class AnyObjectPtr(ctypes.POINTER(AnyObject)):
    _type_ = AnyObject

    def __del__(self):
        from opendp._data import object_free
        object_free(self)


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

    # rust stack traces follow from here:
    raise OpenDPException(variant, message, backtrace)
