import ctypes
import os
import sys
from typing import Optional, Any

# list all acceptable alternative types for each default type
ATOM_EQUIVALENCE_CLASSES = {
    'i32': ['u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64'],
    'f64': ['f32', 'f64'],
    'bool': ['bool']
}


def _get_lib_dir() -> str:
    lib_dir = os.environ.get("OPENDP_LIB_DIR", os.path.join(os.path.dirname(os.path.abspath(__file__)), "lib"))
    if not os.path.exists(lib_dir):
        raise ValueError("Unable to find lib directory. Consider setting OPENDP_LIB_DIR to a valid directory.")
    return lib_dir


def _get_lib_name() -> str:
    platform_to_name = {
        "darwin": "libopendp_ffi.dylib",
        "linux": "libopendp_ffi.so",
        "win32": "opendp_ffi.dll",
    }
    if sys.platform not in platform_to_name:
        raise Exception("Platform not supported", sys.platform)
    return platform_to_name[sys.platform]


lib_path = os.path.join(_get_lib_dir(), _get_lib_name())
lib = ctypes.cdll.LoadLibrary(lib_path)


class FfiSlice(ctypes.Structure):
    _fields_ = [
        ("ptr", ctypes.c_void_p),
        ("len", ctypes.c_size_t),
    ]


class FfiObject(ctypes.Structure):
    pass  # Opaque struct


class FfiMeasurement(ctypes.Structure):
    pass  # Opaque struct


class FfiTransformation(ctypes.Structure):
    pass  # Opaque struct


class BoolPtr(ctypes.POINTER(ctypes.c_bool)):
    _type_ = ctypes.c_bool


class FfiObjectPtr(ctypes.POINTER(FfiObject)):
    _type_ = FfiObject


class FfiSlicePtr(ctypes.POINTER(FfiSlice)):
    _type_ = FfiSlice


class FfiMeasurementPtr(ctypes.POINTER(FfiMeasurement)):
    _type_ = FfiMeasurement

    def __call__(self, arg, *, type_name=None):
        from opendp.v1.core import measurement_invoke
        # TODO: route type_name into measurement_invoke
        return measurement_invoke(self, arg)

    def check(self, d_in, d_out, *, d_in_type_name=None, d_out_type_name=None, debug=False):
        from opendp.v1.convert import py_to_object
        from opendp.v1.core import measurement_check
        from opendp.v1.data import bool_free
        d_in = py_to_object(d_in, d_in_type_name)
        d_out = py_to_object(d_out, d_out_type_name)

        def _check():
            return measurement_check(self, d_in, d_out)

        if debug:
            return _check()

        try:
            return _check()
        except OdpException as err:
            if err.variant == "RelationDebug":
                return False
            raise


class FfiTransformationPtr(ctypes.POINTER(FfiTransformation)):
    _type_ = FfiTransformation

    def __call__(self, arg, *, type_name=None):
        from opendp.v1.convert import py_to_object, object_to_py
        from opendp.v1.core import transformation_invoke
        arg = py_to_object(arg, type_name)
        res = transformation_invoke(self, arg)
        return object_to_py(res)

    def __rshift__(self, other: "FfiMeasurementPtr"):
        if isinstance(other, FfiMeasurementPtr):
            from opendp.v1.core import make_chain_mt
            return make_chain_mt(other, self)

        if isinstance(other, FfiTransformationPtr):
            from opendp.v1.core import make_chain_tt
            return make_chain_tt(other, self)



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


class UnknownTypeException(Exception):
    pass


class OdpException(Exception):
    def __init__(self, variant: str, message: str = None, inner_traceback: str = None):
        self.variant = variant
        self.message = message
        self.inner_traceback = inner_traceback

    def __str__(self) -> str:
        response = self.variant
        if self.message:
            response += f'("{self.message}")'
        if self.inner_traceback:
            response += "\n" + '\n'.join('\t' + line for line in self.inner_traceback.split('\n'))
        return response


def str_to_c_char_p(s: Optional[str]) -> Optional[bytes]:
    return None if s is None else s.encode("utf-8")


def c_char_p_to_str(s: Optional[bytes]) -> Optional[str]:
    return None if s is None else s.decode("utf-8")


def unwrap(result, type_) -> Any:
    from opendp.v1.core import error_free

    if not isinstance(result, FfiResult):
        return result

    if result.tag == 0:
        return ctypes.cast(result.payload.Ok, type_)

    err = result.payload.Err
    err_contents = err.contents
    variant = c_char_p_to_str(err_contents.variant)
    message = c_char_p_to_str(err_contents.message)
    backtrace = c_char_p_to_str(err_contents.backtrace)

    if not error_free(err):
        raise OdpException("Failed to free error.")

    raise OdpException(variant, message, backtrace)

