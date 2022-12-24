import apache_beam as beam
import ctypes
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *


map_method_type = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.py_object, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p)
take_method_type = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.py_object, ctypes.c_char_p)

class ExternalRuntime(ctypes.Structure):
    _fields_ = [
        ("map_method", map_method_type),
        ("take_method", take_method_type),
        # ("map_method", ctypes.c_void_p),
        # ("take_method", ctypes.c_void_p),
        ("foo", ctypes.c_int32),
        ("bar", ctypes.c_double),
    ]


opendp_beam__call_closure_1 = lib.opendp_beam__call_closure_1
opendp_beam__call_closure_1.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
opendp_beam__call_closure_1.restype = FfiResult

opendp_beam__new_collection = lib.opendp_beam__new_collection
opendp_beam__new_collection.argtypes = [ctypes.POINTER(ExternalRuntime), ctypes.c_void_p, ctypes.c_char_p]
opendp_beam__new_collection.restype = FfiResult

opendp_beam__new_collection_methods = lib.opendp_beam__new_collection_methods
opendp_beam__new_collection_methods.argtypes = [map_method_type, take_method_type, ctypes.py_object, ctypes.c_char_p]
opendp_beam__new_collection_methods.restype = FfiResult


def make_mul(
        constant,
        T: RuntimeTypeDescriptor = None
) -> Transformation:
    assert_features("contrib")

    # Standardize type arguments.
    T = RuntimeType.parse_or_infer(type_name=T, public_example=constant)

    # Convert arguments to c types.
    c_constant = py_to_c(constant, c_type=ctypes.c_void_p, type_name=T)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_beam__make_mul
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_constant, c_T), Transformation))

    return output


def pcollection_to_data(pcollection):
    # return ctypes.pointer(ctypes.py_object(pcollection))
    return ctypes.py_object(pcollection)


def data_to_pcollection(data):
    return data.value


def map_method(data, closure, T, U):
    print(f"python map f", data, closure, T, U)
    def f(x):
        print(f"python map({x})")
        y = 0
        x = ctypes.c_int32(x)
        y = ctypes.c_int32(y)
        xp, yp = ctypes.byref(x), ctypes.byref(y)
        print("calling closure", closure, xp, yp)
        opendp_beam__call_closure_1(closure, xp, yp)
        return y.value

    try:
        # 1. convert AnyObject to Python type
        # pcollection = data_to_pcollection(data)
        pcollection = data

        # 2. invoke the user-supplied function
        ret = pcollection | "OpenDP Closure" >> beam.Map(f)

        # 3. convert back to an AnyObject
        U = U.decode("utf-8")
        ret = make_collection(ret, str(U))
        print("PYTHON", "got new collection back")
        # don't free c_out, because we are giving ownership to Rust
        ret.__class__ = ctypes.POINTER(AnyObject)

        # 4. pack up into an FfiResult
        lib.ffiresult_ok.argtypes = [ctypes.c_void_p]
        lib.ffiresult_ok.restype = ctypes.c_void_p
        return lib.ffiresult_ok(ret)

    except Exception:
        import traceback
        lib.ffiresult_err.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        lib.ffiresult_err.restype = ctypes.c_void_p
        return lib.ffiresult_err(
            ctypes.c_char_p(f"Continued stack trace from Exception in user-defined function".encode()),
            ctypes.c_char_p(traceback.format_exc().encode()),
        )

def take_method(data, T):
    print("python take")
    return []


c_map_method = map_method_type(map_method)
c_take_method = take_method_type(take_method)

def make_collection(pcollection, T: RuntimeTypeDescriptor=None):
    T = RuntimeType.parse_or_infer(type_name=T)

    data = pcollection_to_data(pcollection)

    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # runtime = ExternalRuntime(map_method=map_method, take_method=c_take_method, foo=987, bar=654.0)
    # res = opendp_beam__new_collection(map_method, take_method, data, c_T)
    res = opendp_beam__new_collection_methods(c_map_method, c_take_method, data, c_T)

    return unwrap(res, AnyObjectPtr)
