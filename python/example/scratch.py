import ctypes
import pyarrow as pa

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
func = lib.opendp__test_arrow
func.argtypes = [ctypes.POINTER(ArrowArray), ctypes.POINTER(ArrowSchema)]
func.restype = ctypes.POINTER(ArrowArraySchema)
print(func)


array = pa.array([1, 2, None, 4])
print(array)

ffi_array = ArrowArray()
ffi_schema = ArrowSchema()
array._export_to_c(ctypes.addressof(ffi_array), ctypes.addressof(ffi_schema))

res = func(ctypes.byref(ffi_array), ctypes.byref(ffi_schema))
print(res)
# res = pa.Array._import_from_c(ctypes.addressof(res.contents.array.contents), ctypes.addressof(res.contents.schema.contents))
# print(res)
print("YEAH")
