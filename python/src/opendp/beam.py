import apache_beam
import ctypes
import tempfile
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *


c_data_t = ctypes.py_object

map_method_type = ctypes.CFUNCTYPE(ctypes.c_void_p, c_data_t, ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p)
take_method_type = ctypes.CFUNCTYPE(ctypes.c_void_p, c_data_t, ctypes.c_char_p)

class ExternalRuntime(ctypes.Structure):
    _fields_ = [
        ("map_method", map_method_type),
        ("take_method", take_method_type),
    ]


def _get_ctype(T):
    c_type = {
        "bool": ctypes.c_uint8,  # Must be synchronized with c_bool in crate::ffi::util
        "char": ctypes.c_char,
        "i8": ctypes.c_int8,
        "i16": ctypes.c_int16,
        "i32": ctypes.c_int32,
        "i64": ctypes.c_int64,
        "u8": ctypes.c_uint8,
        "u16": ctypes.c_uint16,
        "u32": ctypes.c_uint32,
        "u64": ctypes.c_uint64,
        "f32": ctypes.c_float,
        "f64": ctypes.c_double,
        "String": ctypes.c_char_p,  # TODO
        "AnyObject": ctypes.c_void_p,  # TODO
    }.get(T)
    if c_type is None:
        raise UnknownTypeException(T)
    return c_type


def _call_closure_1(c_closure, xp, yp):
    lib_function = lib.opendp_beam__call_closure_1
    lib_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
    lib_function.restype = FfiResult
    _res = unwrap(lib_function(c_closure, xp, yp), ctypes.c_void_p)


def _wrap_closure(c_closure, T, U):
    T_ctype = _get_ctype(T)
    U_ctype = _get_ctype(U)

    def f(x):
        print(f"Python closure({x})")
        x = T_ctype(x)
        y = U_ctype()
        xp, yp = ctypes.byref(x), ctypes.byref(y)
        print("calling Rust closure", c_closure, xp, yp)
        _call_closure_1(c_closure, xp, yp)
        return y.value

    return f


def map_method(c_collection, c_closure, c_T, c_U):
    print(f"python map f", c_collection, c_closure, c_T, c_U)
    try:
        # 1. Convert args to Python types
        # c_collection is already a Python object thanks to ctypes.py_object
        py_collection = c_collection
        # ctypes converts these to binary strings
        # FIXME: Proper conversion, this will leak.
        T = c_T.decode("utf-8")
        U = c_U.decode("utf-8")
        f = _wrap_closure(c_closure, T, U)

        # 2. map the Python function/OpenDP closure
        py_out = py_collection | "OpenDP Map" >> apache_beam.Map(f)

        # 3. convert back to an AnyObject
        c_out = py_to_c(py_out, c_type=AnyObjectPtr, type_name=Collection[U])
        # don't free c_out, because we are giving ownership to Rust
        c_out.__class__ = ctypes.POINTER(AnyObject)

        # 4. pack up into an FfiResult
        lib.ffiresult_ok.argtypes = [ctypes.c_void_p]
        lib.ffiresult_ok.restype = ctypes.c_void_p
        return lib.ffiresult_ok(ctypes.addressof(c_out.contents))

    except Exception:
        import traceback
        lib.ffiresult_err.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        lib.ffiresult_err.restype = ctypes.c_void_p
        return lib.ffiresult_err(
            ctypes.c_char_p(f"Continued stack trace from Exception in Python reverse callback".encode()),
            ctypes.c_char_p(traceback.format_exc().encode()),
        )

c_map_method = map_method_type(map_method)


def take_method(pcollection, T):
    print("python take")
    try:
        with tempfile.TemporaryDirectory() as out_dir:
            file_path_prefix=f"{out_dir}/take"
            coder = apache_beam.coders.PickleCoder()
            (
                    pcollection
                    | "Combine" >> apache_beam.combiners.ToList()
                    | "Write" >> apache_beam.io.WriteToText(file_path_prefix=file_path_prefix, num_shards=1, coder=coder)
            )
            pcollection.pipeline.run().wait_until_finish()
            with open(f"{file_path_prefix}-00000-of-00001", "rb") as f:
                encoded = f.read()
        taken = coder.decode(encoded)
        print("OUT", taken)
        return taken

    except Exception:
        import traceback
        lib.ffiresult_err.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        lib.ffiresult_err.restype = ctypes.c_void_p
        return lib.ffiresult_err(
            ctypes.c_char_p(f"Continued stack trace from Exception in Python reverse callback".encode()),
            ctypes.c_char_p(traceback.format_exc().encode()),
        )


c_take_method = take_method_type(take_method)


def _pcollection_to_data(pcollection):
    return ctypes.py_object(pcollection)


def _data_to_pcollection(data):
    return data.value


def _pcollection_to_obj(val: apache_beam.PCollection, type_name: RuntimeType) -> AnyObjectPtr:
    assert type_name.origin == 'Collection'
    assert len(type_name.args) == 1, "Collection only has one generic argument"
    inner_type_name = type_name.args[0]

    # Standardize type arguments.
    T = inner_type_name

    # Convert arguments to c types.
    c_data = _pcollection_to_data(val)
    c_runtime = ExternalRuntime(map_method=c_map_method, take_method=c_take_method)
    c_T = py_to_c(T, c_type=ctypes.c_char_p)

    # Call library function.
    lib_function = lib.opendp_beam__new_collection
    lib_function.argtypes = [ExternalRuntime, c_data_t, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = unwrap(lib_function(c_runtime, c_data, c_T), AnyObjectPtr)

    return output


def _obj_to_pcollection(obj: AnyObjectPtr, type_name: RuntimeType) -> apache_beam.PCollection:
    assert type_name.origin == 'Collection'
    assert len(type_name.args) == 1, "Collection only has one generic argument"
    inner_type_name = type_name.args[0]

    # No type arguments to standardize.

    # Convert arguments to c types.
    c_T = py_to_c(inner_type_name, ctypes.c_char_p)
    # don't free obj, because we are giving ownership to Rust
    obj.__class__ = ctypes.POINTER(AnyObject)

    # Call library function.
    lib_function = lib.opendp_beam__get_data
    lib_function.argtypes = [AnyObjectPtr, ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = _data_to_pcollection(unwrap(lib_function(obj, c_T), ctypes.py_object))

    return output




# EXAMPLE CONSTRUCTOR
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
