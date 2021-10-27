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
    _fields_ = [
        ("array", ctypes.POINTER(ArrowArray)),
        ("schema", ctypes.POINTER(ArrowSchema))
    ]

lib_path = "../../rust/target/debug/libopendp_ffi.dylib"
lib = ctypes.cdll.LoadLibrary(lib_path)

arrow_alloc = lib.opendp__arrow_alloc
arrow_alloc.argtypes = []
arrow_alloc.restype = ctypes.POINTER(ArrowArraySchema)

arrow_identity = lib.opendp__arrow_identity
arrow_identity.argtypes = [ctypes.POINTER(ArrowArray), ctypes.POINTER(ArrowSchema)]
arrow_identity.restype = ctypes.POINTER(ArrowArraySchema)

arrow_sort = lib.opendp__arrow_sort
arrow_sort.argtypes = [ctypes.POINTER(ArrowArray), ctypes.POINTER(ArrowSchema), ctypes.c_bool]
arrow_sort.restype = ctypes.POINTER(ArrowArraySchema)

arrow_sum = lib.opendp__arrow_sum
arrow_sum.argtypes = [ctypes.POINTER(ArrowArray), ctypes.POINTER(ArrowSchema), ctypes.POINTER(ArrowArray), ctypes.POINTER(ArrowSchema), ctypes.c_bool]
arrow_sum.restype = ctypes.POINTER(ArrowArraySchema)


def to_ffi(array):
    # ffi_array = ArrowArray()
    # ffi_schema = ArrowSchema()
    ffi_array_schema = arrow_alloc()
    ffi_array = ffi_array_schema.contents.array.contents
    ffi_schema = ffi_array_schema.contents.schema.contents
    print(hex(ctypes.addressof(ffi_array)), hex(ctypes.addressof(ffi_schema)))
    array._export_to_c(ctypes.addressof(ffi_array), ctypes.addressof(ffi_schema))
    return ffi_array, ffi_schema

def _ptr_to_int(ptr):
    vptr = ctypes.cast(ptr, ctypes.c_void_p)
    value = vptr.value
    return value if value is not None else 0

def from_ffi(ffi_array_schema):
    ffi_array = ffi_array_schema.contents.array
    ffi_schema = ffi_array_schema.contents.schema
    return pa.Array._import_from_c(_ptr_to_int(ffi_array), _ptr_to_int(ffi_schema))

def do_identity(dry=False):
    in0 = pa.array([1, 2, None, 4])
    print(f"python: in0 = {in0}")

    in0_array, in0_schema = to_ffi(in0)
    out_array_schema = arrow_identity(ctypes.byref(in0_array), ctypes.byref(in0_schema), dry)

    if bool(out_array_schema):
        out = from_ffi(out_array_schema)
        print(f"python: out = {out}")


def do_sort(dry=False):
    in0 = pa.array([1, 2, None, 4])
    print(f"python: in0 = {in0}")

    in0_array, in0_schema = to_ffi(in0)
    out_array_schema = arrow_sort(ctypes.byref(in0_array), ctypes.byref(in0_schema), dry)

    if bool(out_array_schema):
        out = from_ffi(out_array_schema)
        print(f"python: out = {out}")


def do_sum(dry=False):
    in0 = pa.array([1, 2, None, 4])
    in1 = pa.array([1, 2, 3, None])
    print(f"python: in0 = {in0}, in1 = {in1}")

    in0_array, in0_schema = to_ffi(in0)
    in1_array, in1_schema = to_ffi(in1)
    # print(f"python: in0_array = {in0_array}, in0_schema = {in0_schema}, in1_array = {in1_array}, in1_schema = {in1_schema}")

    # out_array_schema = arrow_sum(ctypes.byref(in0_array), ctypes.byref(in0_schema), ctypes.byref(in1_array), ctypes.byref(in1_schema), dry)
    arrow_sum(ctypes.byref(in0_array), ctypes.byref(in0_schema), ctypes.byref(in1_array), ctypes.byref(in1_schema), dry)

    # if bool(out_array_schema):
    #     # print(f"python: out_array = {out_array_schema.contents.array}, out_schema = {out_array_schema.contents.schema}")
    #
    #     out = from_ffi(out_array_schema)
    #     print(f"python: out = {out}")


if __name__ == "__main__":
    # do_identity(True)
    # do_identity(False)
    # do_sort(True)
    # do_sort(False)
    # do_sum(True)
    do_sum(False)
    print("YEAH")
