import importlib
import json
import builtins
from functools import wraps

import opendp.prelude as dp


__all__ = ["enable_logging"]
LOGGED_CLASSES = (
    dp.Transformation,
    dp.Measurement,
    dp.Function,
    dp.Domain,
    dp.Metric,
    dp.Measure,
    dp.PartialConstructor,
)
WRAPPED_MODULES = [
    "transformations",
    "measurements",
    "combinators",
    "domains",
    "metrics",
    "measures",
    "prelude",
]


def wrap_func(f, module_name):
    @wraps(f)
    def wrapper(*args, **kwargs):
        chain = f(*args, **kwargs)
        if isinstance(chain, LOGGED_CLASSES):
            chain.log = {  # type: ignore[union-attr]
                "_type": "constructor",
                "func": f.__name__,
                "module": module_name,
            }
            args and chain.log.setdefault("args", args)  # type: ignore[union-attr]
            kwargs and chain.log.setdefault("kwargs", kwargs)  # type: ignore[union-attr]
        return chain

    return wrapper


def to_ast(item):
    if isinstance(item, LOGGED_CLASSES):
        if not hasattr(item, "log"):  # pragma: no cover
            msg = "invoke `enable_logging()` before constructing your measurement"
            raise ValueError(msg)

        return to_ast(item.log)  # type: ignore[union-attr]
    if isinstance(item, tuple):
        return [to_ast(e) for e in item]
    if isinstance(item, list):
        return {"_type": "list", "_items": [to_ast(e) for e in item]}
    if isinstance(item, dict):
        return {key: to_ast(value) for key, value in item.items()}
    if isinstance(item, (dp.RuntimeType, type)):
        return str(dp.RuntimeType.parse(item))
    return item


def to_json(chain, *args, **kwargs):
    return json.dumps(
        # TODO: Include OpenDP version
        # https://github.com/opendp/opendp/issues/2103
        {"ast": chain.to_ast()}, *args, **kwargs
    )



def decode_ast(obj):
    if isinstance(obj, dict):
        if obj.get("_type") == "type":
            return getattr(builtins, dp.RuntimeType.parse(obj["name"]))  # type: ignore[arg-type]

        if obj.get("_type") == "list":
            return [decode_ast(i) for i in obj["_items"]]

        if obj.get("_type") == "constructor":
            module = importlib.import_module(f"opendp.{obj['module']}")
            constructor = getattr(module, obj["func"])

            return constructor(
                *decode_ast(obj.get("args", ())),
                **decode_ast(obj.get("kwargs", {}))
            )
        
        if obj.get("_type") == "partial_chain":
            return decode_ast(obj["lhs"]) >> decode_ast(obj["rhs"])
    
        return {k: decode_ast(v) for k, v in obj.items()}

    if isinstance(obj, list):
        return tuple(decode_ast(i) for i in obj)

    return obj


def make_load_json(parse_str: str):
    return make_load_ast(json.loads(parse_str))

def make_load_ast(obj, force=False):
    # TODO: Reenable when we can get the OpenDP version:
    # https://github.com/opendp/opendp/issues/2103
    #
    # if obj["version"] != OPENDP_VERSION and not force:
    #     raise ValueError(
    #         f"OpenDP version in parsed object ({obj['version']}) does not match the current installation ({OPENDP_VERSION}). Set `force=True` to try to load anyways."
    #     )

    return decode_ast(obj["ast"])



def enable_logging():
    for name in WRAPPED_MODULES:
        module = importlib.import_module(f"opendp.{name}")
        for f in dir(module):
            is_constructor = f.startswith("make_") or f.startswith("then_")
            is_elem = any(f.endswith(s) for s in ["domain", "distance", "divergence"])
            if is_constructor or is_elem:
                module.__dict__[f] = wrap_func(getattr(module, f), name)

    for cls in LOGGED_CLASSES:
        cls.to_ast = to_ast  # type: ignore[union-attr]
        cls.to_json = to_json  # type: ignore[union-attr]

    trans_shift_inner = dp.Transformation.__rshift__

    @wraps(trans_shift_inner)
    def trans_shift_outer(lhs: dp.Transformation, rhs):
        chain = trans_shift_inner(lhs, rhs)
        if isinstance(rhs, dp.PartialConstructor) and hasattr(lhs, 'log') and hasattr(rhs, 'log'):
            chain.log = {
                "_type": "partial_chain",
                "lhs": lhs.log,  # type: ignore[attr-defined]
                "rhs": rhs.log,  # type: ignore[attr-defined]
            }
        return chain

    dp.Transformation.__rshift__ = trans_shift_outer  # type: ignore[method-assign,assignment]

    # only run once
    enable_logging.__code__ = (lambda: None).__code__