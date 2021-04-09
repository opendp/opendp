import ctypes
import json
import os
import re
import sys


def str_to_c_char_p(s):
    return s.encode("utf-8") if s is not None else None

def c_char_p_to_str(s):
    return s.decode("utf-8") if s is not None else None

def u32_p(i):
    return ctypes.byref(ctypes.c_uint32(i))

def i32_p(i):
    return ctypes.byref(ctypes.c_int32(i))

def i64_p(i):
    return ctypes.byref(ctypes.c_int64(i))

def f32_p(f):
    return ctypes.byref(ctypes.c_float(f))

def f64_p(f):
    return ctypes.byref(ctypes.c_double(f))


class FfiSlice(ctypes.Structure):
    _fields_ = [
        ("ptr", ctypes.c_void_p),
        ("len", ctypes.c_size_t),
    ]


class FfiObject(ctypes.Structure):
    pass # Opaque struct

class FfiMeasurement(ctypes.Structure):
    pass # Opaque struct

class FfiTransformation(ctypes.Structure):
    pass # Opaque struct

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


class OdpException(Exception):
    def __init__(self, variant, message=None, inner_traceback=None):
        self.variant = variant
        self.message = message
        self.inner_traceback = inner_traceback

    def __str__(self):
        response = self.variant
        if self.message:
            response += f"({self.message})"
        if self.inner_traceback:
            response += "\n" + '\n'.join('\t' + line for line in self.inner_traceback.split('\n'))
        return response


def wrap_in_ffislice(ptr, len_):
    return ctypes.byref(FfiSlice(ctypes.cast(ptr, ctypes.c_void_p), len_))


def scalar_to_raw(val, type_name):
    return wrap_in_ffislice(ctypes.byref(OpenDP.ATOM_MAP[type_name](val)), 1)


def raw_to_scalar(raw, type_name):
    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(OpenDP.ATOM_MAP[type_name])).contents.value


def string_to_raw(val):
    return wrap_in_ffislice(ctypes.c_char_p(val.encode()), len(val) + 1)


def raw_to_string(raw):
    return ctypes.cast(raw.contents.ptr, ctypes.c_char_p).value.decode()


def vector_to_raw(val, type_name):
    inner_type_name = type_name[4:-1]
    if not isinstance(val, list):
        raise OdpException(f"Cannot cast a non-list type to a vector")

    if inner_type_name not in OpenDP.ATOM_MAP:
        raise OdpException(f"Members must be one of {OpenDP.ATOM_MAP.keys()}")

    if val:
        # check that actual type can be represented by the inner_type_name
        equivalence_class = OpenDP.ATOM_EQUIVALENCE_CLASSES[OpenDP.infer_ffi_type_name(val[0])]
        if inner_type_name not in equivalence_class:
            raise OdpException("Data cannot be represented by the suggested type_name")

    array = (OpenDP.ATOM_MAP[inner_type_name] * len(val))(*val)
    return wrap_in_ffislice(array, len(val))


def raw_to_vector(raw, type_name):
    inner_type_name = type_name[4:-1]
    return ctypes.cast(raw.contents.ptr, ctypes.POINTER(OpenDP.ATOM_MAP[inner_type_name]))[0:raw.contents.len]


def tuple_to_raw(val, type_name):
    inner_type_names = [i.strip() for i in type_name[1:-1].split(",")]
    if not isinstance(val, tuple):
        raise OdpException("Cannot cast a non-tuple type to a tuple")
    # TODO: temporary check
    if len(inner_type_names) != 2:
        return OdpException("Only 2-tuples are currently supported.")
    # TODO: temporary check
    if len(set(inner_type_names)) > 1:
        return OdpException("Only homogeneously-typed tuples are currently supported.")

    if len(inner_type_names) != len(val):
        return OdpException("type_name members must have same length as tuple")

    if any(t not in OpenDP.ATOM_MAP for t in inner_type_names):
        return OdpException(f"Tuple members must be one of {OpenDP.ATOM_MAP.keys()}")

    # check that actual type can be represented by the inner_type_name
    for v, inner_type_name in zip(val, inner_type_names):
        equivalence_class = OpenDP.ATOM_EQUIVALENCE_CLASSES[OpenDP.infer_ffi_type_name(v)]
        if inner_type_name not in equivalence_class:
            raise OdpException("Data cannot be represented by the suggested type_name")

    ptr_data = (ctypes.cast(ctypes.pointer(OpenDP.ATOM_MAP[name](v)), ctypes.c_void_p) for v, name in zip(val, inner_type_names))
    array = (ctypes.c_void_p * len(val))(*ptr_data)
    return wrap_in_ffislice(ctypes.byref(array), len(val))


def raw_to_tuple(raw: FfiSlice, type_name: str):
    inner_type_names = [i.strip() for i in type_name[1:-1].split(",")]
    # typed pointer
    void_array_ptr = ctypes.cast(raw.contents.ptr, ctypes.POINTER(ctypes.c_void_p))
    # list of void*
    ptr_data = void_array_ptr[0:raw.contents.len]
    # tuple of instances of python types
    return tuple(ctypes.cast(void_p, ctypes.POINTER(OpenDP.ATOM_MAP[name])).contents.value
                 for void_p, name in zip(ptr_data, inner_type_names))


class Mod:

    name_to_type = {
        "void": None,
        "void *": ctypes.c_void_p,
        "const void *": ctypes.c_void_p,
        "int": ctypes.c_int,
        "int8_t": ctypes.c_int8,
        "int16_t": ctypes.c_int16,
        "int32_t": ctypes.c_int32,
        "int64_t": ctypes.c_int64,
        "unsigned int": ctypes.c_uint,
        "uint8_t": ctypes.c_uint8,
        "uint16_t": ctypes.c_uint16,
        "uint32_t": ctypes.c_uint32,
        "uint64_t": ctypes.c_uint64,
        "float": ctypes.c_float,
        "double": ctypes.c_double,
        "char *": ctypes.c_char_p,
        "const char *": ctypes.c_char_p,
        "bool": ctypes.c_bool,
        "FfiSlice *": ctypes.POINTER(FfiSlice),
        "const FfiSlice *": ctypes.POINTER(FfiSlice),
        "bool *": ctypes.POINTER(ctypes.c_bool),  # FIXME: I don't think we want this? It's for error-able bools
        "FfiObject *": ctypes.POINTER(FfiObject),
        "const FfiObject *": ctypes.POINTER(FfiObject),
        "FfiMeasurement *": ctypes.POINTER(FfiMeasurement),
        "const FfiMeasurement *": ctypes.POINTER(FfiMeasurement),
        "FfiTransformation *": ctypes.POINTER(FfiTransformation),
        "const FfiTransformation *": ctypes.POINTER(FfiTransformation),
        "FfiError *": ctypes.POINTER(FfiError),
        "const FfiError *": ctypes.POINTER(FfiError),
        "FfiResult": FfiResult,
    }

    @classmethod
    def initialize(cls, lib, core_prefix):
        # We have a bootstrap problem, in that we need error_free() so we can wrap FfiResult functions.
        # This is a bit janky, but we just hardwire it.
        symbol = core_prefix + "error_free"
        function = lib[symbol]
        function.argtypes = [cls.get_type("FfiError *")]
        function.restype = cls.get_type("void")
        cls.error_free = function

    @classmethod
    def get_type(cls, name, allow_generic=False):
        def lookup_type(n):
            if not n in cls.name_to_type:
                raise Exception("Unknown type", n)
            return cls.name_to_type[n]
        if allow_generic:
            match = re.match("^(.+)<(.+)>$", name) if allow_generic else None
            if match:
                main_name, sub_name = match.groups()
                return lookup_type(main_name), lookup_type(sub_name)
            else:
                return lookup_type(name), None
        else:
            return lookup_type(name)

    def __init__(self, lib, prefix="ffi__"):
        self.lib = lib
        self.prefix = prefix
        self._bootstrap()

    def _bootstrap(self):
        spec = { "name": "bootstrap", "args": [], "ret": "const char *" }
        _name, bootstrap = self._get_function(spec)
        spec_json = bootstrap().decode("ascii")
        spec = json.loads(spec_json)
        self._load(spec)

    def _load(self, spec):
        for function_spec in spec["functions"]:
            name, function = self._get_function(function_spec)
            self.__setattr__(name, function)

    def _get_function(self, spec):
        name = spec["name"]
        symbol = self.prefix + name
        function = self.lib[symbol]
        function.argtypes = [self.get_type(arg[0]) for arg in spec.get("args", [])]
        main_restype, sub_restype = self.get_type(spec.get("ret", "void"), allow_generic=True)
        function.restype = main_restype
        if sub_restype:
            if main_restype != self.get_type("FfiResult"):
                raise Exception("Bogus generic", spec.get("ret", "void"))
            function = self._make_unwrap(function, sub_restype)
        return name, function

    def _make_unwrap(self, function, type_):
        def unwrap(*args):
            res = function(*args)
            if res.tag == 0:
                return ctypes.cast(res.payload.Ok, type_)
            else:
                err = res.payload.Err
                err_contents = err.contents
                variant = c_char_p_to_str(err_contents.variant)
                message = c_char_p_to_str(err_contents.message)
                backtrace = c_char_p_to_str(err_contents.backtrace)
                self.error_free(err)
                raise OdpException(variant, message, backtrace)
        return unwrap


class OpenDP:

    @classmethod
    def _get_lib_dir(cls):
        # Use environment variable to allow running without installation.
        dir = os.environ.get("OPENDP_LIB_DIR")
        if not dir:
            package_dir = os.path.dirname(os.path.abspath(__file__))
            # TODO: Have separate sub-dirs under lib for each platform to avoid name collisions?
            dir = os.path.join(package_dir, "lib")
        return dir

    @classmethod
    def _get_lib_name(cls):
        platform_to_name = {
            "darwin": "libopendp_ffi.dylib",
            "linux": "libopendp_ffi.so",
            "win32": "opendp_ffi.dll",
        }
        if sys.platform not in platform_to_name:
            raise Exception("Platform not supported", sys.platform)
        return platform_to_name[sys.platform]

    @classmethod
    def _get_lib_path(cls):
        dir = cls._get_lib_dir()
        name = cls._get_lib_name()
        return os.path.join(dir, name)

    def __init__(self, lib_path=None):
        lib_path = lib_path or self._get_lib_path()
        lib = ctypes.cdll.LoadLibrary(lib_path)
        Mod.initialize(lib, "opendp_core__")
        self.core = Mod(lib, "opendp_core__")
        self.data = Mod(lib, "opendp_data__")
        self.meas = Mod(lib, "opendp_meas__")
        self.trans = Mod(lib, "opendp_trans__")
        # print("Initialized OpenDP Library")

    def make_chain_tt_multi(self, *transformations):
        if not transformations:
            raise OdpException("Must have at least one Transformation")
        elif len(transformations) == 1:
            return transformations[0]
        else:
            return self.make_chain_tt_multi(*transformations[:-2], self.core.make_chain_tt(transformations[-2], transformations[-1]))

    def to_str(self, data):
        string = self.data.to_string(data)
        return c_char_p_to_str(string)

    @staticmethod
    def get_first(arr):
        return arr[0] if arr else 0

    ATOM_MAP = {
        'f32': ctypes.c_float,
        'f64': ctypes.c_double,
        'u8': ctypes.c_uint8,
        'u16': ctypes.c_uint16,
        'u32': ctypes.c_uint32,
        'u64': ctypes.c_uint64,
        'i8': ctypes.c_int8,
        'i16': ctypes.c_int16,
        'i32': ctypes.c_int32,
        'i64': ctypes.c_int64,
        'bool': ctypes.c_bool,
    }

    # list all acceptable alternative types for each default type
    ATOM_EQUIVALENCE_CLASSES = {
        'i32': ['u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64'],
        'f64': ['f32', 'f64'],
        'bool': ['bool']
    }

    @staticmethod
    def infer_ffi_type_name(val):
        if isinstance(val, int):
            return "i32"
        elif isinstance(val, float):
            return "f64"
        elif isinstance(val, str):
            return "String"
        elif isinstance(val, bool):
            return "bool"
        elif isinstance(val, list):
            return f"Vec<{OpenDP.infer_ffi_type_name(OpenDP.get_first(val))}>"
        elif isinstance(val, tuple):
            return f"({','.join(map(OpenDP.infer_ffi_type_name, val))})"

        raise Exception("Unknown type", type(val))

    def to_raw(self, val, type_name):
        if type_name in self.ATOM_MAP:
            return scalar_to_raw(val, type_name)

        if type_name.startswith("Vec<") and type_name.endswith('>'):
            return vector_to_raw(val, type_name)

        if type_name.startswith('(') and type_name.endswith(')'):
            return tuple_to_raw(val, type_name)

        if type_name == "String":
            return string_to_raw(val)

        raise Exception("Unknown type name", type_name)

    def py_to_obj(self, val, type_arg=None):
        if type_arg:
            type_name = type_arg[1:-1]
        else:
            type_name = self.infer_ffi_type_name(val)

        raw = self.to_raw(val, type_name)
        return self.data.object_new(f"<{type_name}>".encode(), raw)

    def from_raw(self, raw, type_name):
        if type_name in self.ATOM_MAP:
            return raw_to_scalar(raw, type_name)

        if type_name.startswith("Vec<") and type_name.endswith('>'):
            return raw_to_vector(raw, type_name)

        if type_name.startswith('(') and type_name.endswith(')'):
            return raw_to_tuple(raw, type_name)

        if type_name == "String":
            return raw_to_string(raw)

        raise Exception("Unknown type name", type_name)

    def obj_to_py(self, obj):
        type_name = self.data.object_type(obj).value.decode()
        raw = self.data.object_as_raw(obj)
        try:
            return self.from_raw(raw, type_name)
        except Exception as err:
            # print("MASKED ERROR:", err)
            # print("using string fallback")
            raise err
            # If we fail, resort to string representation.
            #TODO: Remove this fallback once we have composition and/or tuples sorted out.
            return self.data.to_string(obj).decode()

    def measurement_invoke(self, measurement, arg, *, type_name=None):
        arg = self.py_to_obj(arg, type_name)
        res = self.core.measurement_invoke(measurement, arg)
        return self.obj_to_py(res)

    def transformation_invoke(self, transformation, arg, *, type_name=None):
        arg = self.py_to_obj(arg, type_name)
        res = self.core.transformation_invoke(transformation, arg)
        return self.obj_to_py(res)

    def measurement_check(self, measurement, d_in, d_out, *, d_in_type_arg=None, d_out_type_arg=None, debug=False):
        d_in = self.py_to_obj(d_in, d_in_type_arg and d_in_type_arg[1:-1])
        d_out = self.py_to_obj(d_out, d_out_type_arg and d_out_type_arg[1:-1])
        if debug:
            return self.core.measurement_check(measurement, d_in, d_out)
        else:
            try:
                return self.core.measurement_check(measurement, d_in, d_out)
            except OdpException as err:
                if err.variant == "RelationDebug":
                    return False
                else:
                    raise err
