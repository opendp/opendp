import ctypes
import sys

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
class FFI_ArrowSchema(ctypes.Structure):
    pass
FFI_ArrowSchema._fields_ = [
    ("format", ctypes.c_char_p),
    ("name", ctypes.c_char_p),
    ("metadata", ctypes.c_char_p),
    ("flags", ctypes.c_int64),
    ("n_children", ctypes.c_int64),
    ("children", ctypes.POINTER(ctypes.POINTER(FFI_ArrowSchema))),
    ("dictionary", ctypes.POINTER(FFI_ArrowSchema)),
    ("release", ctypes.POINTER(ctypes.CFUNCTYPE(None, ctypes.POINTER(FFI_ArrowSchema)))),
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
class FFI_ArrowArray(ctypes.Structure):
    def __str__(self):
        return "{}: {{{}}}".format(self.__class__.__name__,
                                   ", ".join(["{}: {}".format(field[0],
                                                              getattr(self,
                                                                      field[0]))
                                              for field in self._fields_]))
FFI_ArrowArray._fields_ = [
    ("length", ctypes.c_int64),
    ("null_count", ctypes.c_int64),
    ("offset", ctypes.c_int64),
    ("n_buffers", ctypes.c_int64),
    ("n_children", ctypes.c_int64),
    ("buffers", ctypes.POINTER(ctypes.c_void_p)),
    ("children", ctypes.POINTER(ctypes.POINTER(FFI_ArrowArray))),
    ("dictionary", ctypes.POINTER(FFI_ArrowArray)),
    ("release", ctypes.POINTER(ctypes.CFUNCTYPE(None, ctypes.POINTER(FFI_ArrowArray)))),
    ("private_data", ctypes.c_void_p)
]

class FFI_Array(ctypes.Structure):
    _fields_ = [
        ("array", ctypes.POINTER(FFI_ArrowArray)),
        ("schema", ctypes.POINTER(FFI_ArrowSchema))
    ]
    @classmethod
    def new_empty(cls):
        array = cls()
        array.array = ctypes.pointer(FFI_ArrowArray())
        array.schema = ctypes.pointer(FFI_ArrowSchema())
        return array

AllocCBType = ctypes.CFUNCTYPE(None, ctypes.POINTER(FFI_Array))


lib_path = "../../rust/target/debug/libopendp_ffi.dylib"
lib = ctypes.cdll.LoadLibrary(lib_path)

arrow_init = lib.opendp__arrow_init
arrow_init.argtypes = [AllocCBType]
arrow_init.restype = None

arrow_identity = lib.opendp__arrow_identity
arrow_identity.argtypes = [FFI_Array, FFI_Array]
arrow_identity.restype = None

arrow_identity_cb = lib.opendp__arrow_identity_cb
arrow_identity_cb.argtypes = [FFI_Array]
arrow_identity_cb.restype = FFI_Array

arrow_sort = lib.opendp__arrow_sort
arrow_sort.argtypes = [FFI_Array, FFI_Array]
arrow_sort.restype = None

arrow_add = lib.opendp__arrow_add
arrow_add.argtypes = [FFI_Array, FFI_Array, FFI_Array]
arrow_add.restype = None


arrow_consume = lib.opendp__arrow_consume
arrow_consume.argtypes = [FFI_Array]
arrow_consume.restype = None

arrow_produce = lib.opendp__arrow_produce
arrow_produce.argtypes = [FFI_Array]
arrow_produce.restype = None



arrow_alloc = lib.opendp__arrow_alloc
arrow_alloc.argtypes = []
arrow_alloc.restype = FFI_Array

arrow_free = lib.opendp__arrow_free
arrow_free.argtypes = [FFI_Array]
arrow_free.restype = None

_leaks = []

def alloc(ffi_array_ptr):
    global _leaks
    print(f"python: alloc()")
    ffi_array = ffi_array_ptr.contents
    arrow_array = FFI_ArrowArray()
    arrow_schema = FFI_ArrowSchema()
    _leaks += [arrow_array, arrow_schema]
    ffi_array.array = ctypes.pointer(arrow_array)
    ffi_array.schema = ctypes.pointer(arrow_schema)

alloc_cb = AllocCBType(alloc)


def init():
    arrow_init(alloc_cb)


def _ptr_to_int(ptr):
    vptr = ctypes.cast(ptr, ctypes.c_void_p)
    value = vptr.value
    return value if value is not None else 0


def to_ffi(array):
    ffi_array = arrow_alloc()
    array._export_to_c(_ptr_to_int(ffi_array.array), _ptr_to_int(ffi_array.schema))
    return ffi_array


def from_ffi(ffi_array):
    return pa.Array._import_from_c(_ptr_to_int(ffi_array.array), _ptr_to_int(ffi_array.schema))


def do_identity():
    arg = pa.array([1, 2, None, 4])
    print(f"python: arg = {arg}")

    ffi_arg = to_ffi(arg)
    ffi_res = FFI_Array.new_empty()
    arrow_identity(ffi_arg, ffi_res)

    res = from_ffi(ffi_res)
    print(f"python: res = {res}")
    arrow_free(ffi_arg)
    # print(f"python: ffi_arg.array = {ctypes.cast(ffi_arg.array.contents.release, ctypes.c_void_p)}")
    # ffi_arg.array.contents.release = ctypes.cast(ctypes.c_void_p(123), ctypes.POINTER(ctypes.CFUNCTYPE(None, ctypes.POINTER(FFI_ArrowArray))))
    # # ffi_arg.array.contents.release = None
    # print(f"python: ffi_arg.array = {ctypes.cast(ffi_arg.array.contents.release, ctypes.c_void_p)}")


def do_identity_cb():
    arg = pa.array([1, 2, None, 4])
    print(f"python: arg = {arg}")

    ffi_arg = to_ffi(arg)
    print(f"python: did to_ffi")
    ffi_res = arrow_identity_cb(ffi_arg)
    print(f"python: did identity_cb")

    res = from_ffi(ffi_res)
    print(f"python: res = {res}")


def do_sort():
    arg = pa.array([1, 2, None, 4])
    print(f"python: arg = {arg}")

    ffi_arg = to_ffi(arg)
    ffi_res = FFI_Array.new_empty()
    arrow_sort(ffi_arg, ffi_res)

    res = from_ffi(ffi_res)
    print(f"python: res = {res}")


def do_add():
    arg0 = pa.array([1, 2, None, 4])
    arg1 = pa.array([1, 2, 3, None])
    print(f"python: arg0 = {arg0}, arg1 = {arg1}")

    ffi_arg0 = to_ffi(arg0)
    ffi_arg1 = to_ffi(arg1)
    ffi_res = FFI_Array.new_empty()
    arrow_add(ffi_arg0, ffi_arg1, ffi_res)

    res = from_ffi(ffi_res)
    print(f"python: res = {res}")


def do_consume():
    arg = pa.array([1, 2, None, 4] * 10000)

    ffi_arg = to_ffi(arg)
    arrow_consume(ffi_arg)
    # print(f"python: arg = {arg}")
    # print(f"python: DONE")


def do_produce():
    ffi_res = FFI_Array.new_empty()
    arrow_produce(ffi_res)

    res = from_ffi(ffi_res)
    # print(f"python: res = {res}")
    # arrow_free(ffi_res)


if __name__ == "__main__":
    init()
    for i in range(200000):
    # for i in range(2):
        if (i + 1) % 1000 == 0:
            print(i + 1)
        # do_identity()
        # do_sort()
        # do_add()
        # do_consume()
        do_produce()
    print("YEAH")
    import time
    time.sleep(10)
