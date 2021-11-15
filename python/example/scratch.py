import ctypes
import pyarrow as pa
import sys

ARROW_FLAG_DICTIONARY_ORDERED = 1
ARROW_FLAG_NULLABLE = 2
ARROW_FLAG_MAP_KEYS_SORTED = 4

# struct ArrowSchema {
#   // Array type description
#   const char* format;
#   const char* name;
#   const char* metadata;
#   int64_t flags;
#   int64_t n_children;
#   struct ArrowSchema** children;
#   struct ArrowSchema* dictionary;
#   // Release callback
#   void (*release)(struct ArrowSchema*);
#   // Opaque producer-specific data
#   void* private_data;
# };
class ArrowSchema(ctypes.Structure):
    pass
    # def __del__(self):
    #     print("ArrowSchema del")
ArrowSchema._fields_ = [
    ("format", ctypes.c_char_p),
    ("name", ctypes.c_char_p),
    ("metadata", ctypes.c_char_p),
    ("flags", ctypes.c_int64),
    ("n_children", ctypes.c_int64),
    ("children", ctypes.POINTER(ctypes.POINTER(ArrowSchema))),
    ("dictionary", ctypes.POINTER(ArrowSchema)),
    ("release", ctypes.POINTER(ctypes.CFUNCTYPE(None, ctypes.POINTER(ArrowSchema)))),
    ("private_data", ctypes.c_void_p)
]

# struct ArrowArray {
#   // Array data description
#   int64_t length;
#   int64_t null_count;
#   int64_t offset;
#   int64_t n_buffers;
#   int64_t n_children;
#   const void** buffers;
#   struct ArrowArray** children;
#   struct ArrowArray* dictionary;
#   // Release callback
#   void (*release)(struct ArrowArray*);
#   // Opaque producer-specific data
#   void* private_data;
# };
class ArrowArray(ctypes.Structure):
    pass
    # def __del__(self):
    #     print("ArrowArray del")
ArrowArray._fields_ = [
    ("length", ctypes.c_int64),
    ("null_count", ctypes.c_int64),
    ("offset", ctypes.c_int64),
    ("n_buffers", ctypes.c_int64),
    ("n_children", ctypes.c_int64),
    ("buffers", ctypes.POINTER(ctypes.c_void_p)),
    ("children", ctypes.POINTER(ctypes.POINTER(ArrowArray))),
    ("dictionary", ctypes.POINTER(ArrowArray)),
    ("release", ctypes.POINTER(ctypes.CFUNCTYPE(None, ctypes.POINTER(ArrowArray)))),
    ("private_data", ctypes.c_void_p)
]

class ArrowArraySchema(ctypes.Structure):
    pass
    # def __del__(self):
    #     print("ArrowArraySchema del")
    _fields_ = [
        ("array", ctypes.POINTER(ArrowArray)),
        ("schema", ctypes.POINTER(ArrowSchema))
    ]

lib_path = "../../rust/target/debug/libopendp_ffi.dylib"
lib = ctypes.cdll.LoadLibrary(lib_path)

arrow_identity = lib.opendp__arrow_identity
arrow_identity.argtypes = [ctypes.POINTER(ArrowArray), ctypes.POINTER(ArrowSchema)]
arrow_identity.restype = ctypes.POINTER(ArrowArraySchema)

arrow_identity_param = lib.opendp__arrow_identity_param
arrow_identity_param.argtypes = [ctypes.POINTER(ArrowArray), ctypes.POINTER(ArrowSchema), ctypes.c_bool, ctypes.c_bool]
arrow_identity_param.restype = ctypes.POINTER(ArrowArraySchema)

arrow_sort = lib.opendp__arrow_sort
arrow_sort.argtypes = [ctypes.POINTER(ArrowArray), ctypes.POINTER(ArrowSchema)]
arrow_sort.restype = ctypes.POINTER(ArrowArraySchema)

arrow_add = lib.opendp__arrow_add
arrow_add.argtypes = [ctypes.POINTER(ArrowArray), ctypes.POINTER(ArrowSchema), ctypes.POINTER(ArrowArray), ctypes.POINTER(ArrowSchema)]
arrow_add.restype = ctypes.POINTER(ArrowArraySchema)

arrow_free = lib.opendp__arrow_free
arrow_free.argtypes = [ctypes.POINTER(ArrowArraySchema)]
arrow_free.restype = None


def to_ffi(array):
    ffi_array = ArrowArray()
    ffi_schema = ArrowSchema()
    array._export_to_c(ctypes.addressof(ffi_array), ctypes.addressof(ffi_schema))
    ffi_array.release = None
    ffi_schema.release = None
    return ffi_array, ffi_schema

def _ptr_to_int(ptr):
    vptr = ctypes.cast(ptr, ctypes.c_void_p)
    value = vptr.value
    return value if value is not None else 0

def from_ffi(ffi_array_schema):
    if not bool(ffi_array_schema):
        return None
    parse = True
    if not parse:
        return None
    copy = True
    ffi_array = ffi_array_schema.contents.array
    ffi_schema = ffi_array_schema.contents.schema
    if copy:
        py_ffi_array = ArrowArray()
        ctypes.pointer(py_ffi_array)[0] = ffi_array.contents
        py_ffi_schema = ArrowSchema()
        ctypes.pointer(py_ffi_schema)[0] = ffi_schema.contents
        ffi_array.contents.release = None
        ffi_schema.contents.release = None
        arrow_free(ffi_array_schema)
        return pa.Array._import_from_c(ctypes.addressof(py_ffi_array), ctypes.addressof(py_ffi_schema))
    else:
        return pa.Array._import_from_c(_ptr_to_int(ffi_array), _ptr_to_int(ffi_schema))

def do_identity():
    create_array = True
    call = True
    parse = True and create_array
    gen = False
    if create_array:
        arg = pa.array([1, 2, None, 4])
        print(f"python: arg = {arg}")

        ffi_arg_array, ffi_arg_schema = to_ffi(arg)
        ffi_arg_array_ptr, ffi_arg_schema_ptr = ctypes.byref(ffi_arg_array), ctypes.byref(ffi_arg_schema)
    else:
        ffi_arg_array_ptr, ffi_arg_schema_ptr = None, None
    if call:
        # ffi_res = arrow_identity(ffi_arg_array_ptr, ffi_arg_schema_ptr)
        ffi_res = arrow_identity_param(ffi_arg_array_ptr, ffi_arg_schema_ptr, parse, gen)
    else:
        ffi_res = None

    res = from_ffi(ffi_res)
    print(f"python: res = {res}")


def do_sort():
    arg = pa.array([1, 2, None, 4])
    print(f"python: arg = {arg}")

    ffi_arg_array, ffi_arg_schema = to_ffi(arg)
    ffi_res = arrow_sort(ctypes.byref(ffi_arg_array), ctypes.byref(ffi_arg_schema))

    res = from_ffi(ffi_res)
    print(f"python: res = {res}")


def do_add():
    arg0 = pa.array([1, 2, None, 4])
    arg1 = pa.array([1, 2, 3, None])
    print(f"python: arg0 = {arg0}, arg1 = {arg1}")

    ffi_arg0_array, ffi_arg0_schema = to_ffi(arg0)
    ffi_arg1_array, ffi_arg1_schema = to_ffi(arg1)
    ffi_res = arrow_add(ctypes.byref(ffi_arg0_array), ctypes.byref(ffi_arg0_schema), ctypes.byref(ffi_arg1_array), ctypes.byref(ffi_arg1_schema))

    res = from_ffi(ffi_res)
    print(f"python: res = {res}")


if __name__ == "__main__":
    for _ in range(100):
        do_identity()
    # do_sort()
    # do_add()
    print("YEAH")
