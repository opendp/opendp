import ctypes
import json
import re


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
    pass

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
                backtace = c_char_p_to_str(err_contents.backtrace)
                self.error_free(err)
                raise OdpException(variant, message, backtace)
        return unwrap

class OpenDP:

    def __init__(self, lib_path):
        lib = ctypes.cdll.LoadLibrary(lib_path)
        Mod.initialize(lib, "opendp_core__")
        self.core = Mod(lib, "opendp_core__")
        self.data = Mod(lib, "opendp_data__")
        self.meas = Mod(lib, "opendp_meas__")
        self.trans = Mod(lib, "opendp_trans__")
        print("Initialized OpenDP Library")

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
