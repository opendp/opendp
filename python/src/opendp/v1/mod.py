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


class AnyObject(ctypes.Structure):
    pass  # Opaque struct


class AnyMetricDistance(ctypes.Structure):
    pass  # Opaque struct


class AnyMeasureDistance(ctypes.Structure):
    pass  # Opaque struct


class AnyMeasurement(ctypes.Structure):
    pass  # Opaque struct


class AnyTransformation(ctypes.Structure):
    pass  # Opaque struct


class BoolPtr(ctypes.POINTER(ctypes.c_bool)):
    _type_ = ctypes.c_bool


class AnyObjectPtr(ctypes.POINTER(AnyObject)):
    _type_ = AnyObject


class AnyMeasureDistancePtr(ctypes.POINTER(AnyMeasureDistance)):
    _type_ = AnyMeasureDistance


class AnyMetricDistancePtr(ctypes.POINTER(AnyMetricDistance)):
    _type_ = AnyMetricDistance


class FfiSlicePtr(ctypes.POINTER(FfiSlice)):
    _type_ = FfiSlice


class AnyMeasurementPtr(ctypes.POINTER(AnyMeasurement)):
    _type_ = AnyMeasurement

    def __call__(self, arg):
        from opendp.v1.core import measurement_invoke
        return measurement_invoke(self, arg)

    def invoke(self, arg):
        from opendp.v1.core import measurement_invoke
        return measurement_invoke(self, arg)

    def check(self, d_in, d_out, *, debug=False):
        from opendp.v1.core import measurement_check

        if debug:
            return measurement_check(self, d_in, d_out)

        try:
            return measurement_check(self, d_in, d_out)
        except OdpException as err:
            if err.variant == "RelationDebug":
                return False
            raise

    @property
    def input_distance_type(self) -> "RuntimeType":
        from opendp.v1.core import measurement_input_distance_type
        from opendp.v1.typing import RuntimeType
        return RuntimeType.parse(measurement_input_distance_type(self))

    @property
    def output_distance_type(self) -> "RuntimeType":
        from opendp.v1.typing import RuntimeType
        from opendp.v1.core import measurement_output_distance_type
        return RuntimeType.parse(measurement_output_distance_type(self))

    @property
    def input_carrier_type(self) -> "RuntimeType":
        from opendp.v1.core import measurement_input_carrier_type
        from opendp.v1.typing import RuntimeType
        return RuntimeType.parse(measurement_input_carrier_type(self))



class AnyTransformationPtr(ctypes.POINTER(AnyTransformation)):
    _type_ = AnyTransformation

    def __call__(self, arg):
        from opendp.v1.core import transformation_invoke
        return transformation_invoke(self, arg)

    def invoke(self, arg):
        from opendp.v1.core import transformation_invoke
        return transformation_invoke(self, arg)

    def __rshift__(self, other: "AnyMeasurementPtr"):
        if isinstance(other, AnyMeasurementPtr):
            from opendp.v1.core import make_chain_mt
            return make_chain_mt(other, self)

        if isinstance(other, AnyTransformationPtr):
            from opendp.v1.core import make_chain_tt
            return make_chain_tt(other, self)

        raise ValueError(f"rshift expected a measurement or transformation, got {other}")

    def check(self, d_in, d_out, *, debug=False):
        from opendp.v1.core import transformation_check

        if debug:
            return transformation_check(self, d_in, d_out)

        try:
            return transformation_check(self, d_in, d_out)
        except OdpException as err:
            if err.variant == "RelationDebug":
                return False
            raise

    @property
    def input_distance_type(self) -> "RuntimeType":
        from opendp.v1.core import transformation_input_distance_type
        from opendp.v1.typing import RuntimeType
        return RuntimeType.parse(transformation_input_distance_type(self))

    @property
    def output_distance_type(self) -> "RuntimeType":
        from opendp.v1.core import transformation_output_distance_type
        from opendp.v1.typing import RuntimeType
        return RuntimeType.parse(transformation_output_distance_type(self))
    
    @property
    def input_carrier_type(self) -> "RuntimeType":
        from opendp.v1.core import transformation_input_carrier_type
        from opendp.v1.typing import RuntimeType
        return RuntimeType.parse(transformation_input_carrier_type(self))


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

