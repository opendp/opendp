import ctypes
import json


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
    }

    @classmethod
    def get_type(cls, name):
        if not name in cls.name_to_type:
            raise Exception(f"Unknown type {name}")
        return cls.name_to_type[name]

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
        function.restype = self.get_type(spec.get("ret", "void"))
        return name, function

class OpenDP:

    def __init__(self, lib_path):
        lib = ctypes.cdll.LoadLibrary(lib_path)
        self.lib = lib
        self.core = Mod(lib, "opendp_core__")
        self.data = Mod(lib, "opendp_data__")
        self.meas = Mod(lib, "opendp_meas__")
        self.trans = Mod(lib, "opendp_trans__")
        print("Initialized OpenDP Library")

    def str_to_c_char_p(self, s):
        return s.encode("utf-8")

    def c_char_p_to_str(self, s):
        return s.decode("utf-8")

    def i32_p(self, i):
        return ctypes.byref(ctypes.c_int32(i))

    def i64_p(self, i):
        return ctypes.byref(ctypes.c_int64(i))

    def f32_p(self, f):
        return ctypes.byref(ctypes.c_float(f))

    def f64_p(self, f):
        return ctypes.byref(ctypes.c_double(f))

    def make_chain_tt_multi(self, *transformations):
        if not transformations:
            raise Exception
        elif len(transformations) == 1:
            return transformations[0]
        else:
            return self.make_chain_tt_multi(*transformations[:-2], self.core.make_chain_tt(transformations[-2], transformations[-1]))

    def to_str(self, data):
        string = self.data.to_string(data)
        return self.c_char_p_to_str(string)
