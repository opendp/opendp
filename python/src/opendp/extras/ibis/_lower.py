"""Restricted Polars-to-Ibis lowering for OpenDP deterministic release heads.

The Rust splitter supplies the *analyzed* deterministic heads. This module is
not a general Polars translator, a privacy validator, or an execution engine.
Unsupported operations must fail rather than silently change the computation.
"""

import json
import warnings
from operator import add, and_, eq, ge, gt, le, lt, mul, ne, or_, sub, truediv

import ibis
import ibis.expr.types as ir


_BINARY = {
    "Plus": add,
    "Minus": sub,
    "Multiply": mul,
    "TrueDivide": truediv,
    "Divide": truediv,
    "Eq": eq,
    "NotEq": ne,
    "Lt": lt,
    "LtEq": le,
    "Gt": gt,
    "GtEq": ge,
    "And": and_,
    "Or": or_,
}

_NUMERIC = {
    "Boolean": "boolean",
    "Int8": "int8",
    "Int16": "int16",
    "Int32": "int32",
    "Int64": "int64",
    "UInt8": "uint8",
    "UInt16": "uint16",
    "UInt32": "uint32",
    "UInt64": "uint64",
    "Float32": "float32",
    "Float64": "float64",
}


def _node(node):
    if isinstance(node, str):
        return node, None
    if not isinstance(node, dict) or len(node) != 1:
        raise NotImplementedError(f"Unsupported Polars node: {node!r}")
    return next(iter(node.items()))


def _fields(payload):
    if not isinstance(payload, dict):
        raise NotImplementedError(f"Expected a Polars plan object: {payload!r}")
    return payload


def _literal(payload):
    kind, value = _node(payload)
    if kind == "Dyn":
        dtype, value = _node(value)
        if dtype == "Int":
            # Keep constants as expressions, even when they are used on the left.
            return ibis.literal(value, type="int64")
        if dtype == "Float":
            return ibis.literal(value, type="float64")
    elif kind == "Scalar":
        dtype, value = _node(value)
        if dtype in _NUMERIC:
            # Casting a Float64 literal forces SQL to round to binary32.
            if dtype == "Float32":
                return ibis.literal(value, type="float64").cast("float32")
            return ibis.literal(value, type=_NUMERIC[dtype])
        if dtype == "String":
            return ibis.literal(value)
        if dtype == "Null":
            return ibis.null() if value == "Null" else ibis.null().cast(
                _type_name(value)
            )
    raise NotImplementedError(f"Unsupported Polars literal: {payload!r}")


def _type_name(name):
    if name in _NUMERIC:
        return _NUMERIC[name]
    if name in {"String", "Boolean", "Binary"}:
        return {"String": "string", "Boolean": "boolean", "Binary": "binary"}[name]
    raise NotImplementedError(f"Unsupported Polars dtype: {name!r}")


def _cast(value, dtype, options):
    if options not in ("Strict", "NonStrict"):
        raise NotImplementedError(f"Unsupported cast options: {options!r}")
    target = ibis.dtype(_type_name(dtype))
    source = value.type()
    if options == "NonStrict" and source != target:
        # SQL TRY_CAST is backend-dependent; an error or fallback can disclose
        # properties of private data. Only the identity cast is accepted.
        raise NotImplementedError("Non-strict dtype-changing casts are unsupported")
    if source.is_floating() and target.is_integer():
        # SQL may round where Polars truncates toward zero.
        raise NotImplementedError("Float-to-integer casts are unsupported")
    if source.is_integer() and target.is_integer():
        if not (
            target.bounds.lower <= source.bounds.lower
            and source.bounds.upper <= target.bounds.upper
        ):
            raise NotImplementedError("Narrowing integer casts are unsupported")
    return value.cast(target)


def _value(expr, table):
    tag, payload = _node(expr)
    if tag == "Column":
        if payload not in table.columns:
            raise ValueError(f"Column {payload!r} is not in the input relation")
        return table[payload]
    if tag == "Literal":
        return _literal(payload)
    if tag == "Alias":
        return _value(payload[0], table)
    if tag == "Len":
        return table.count().cast("uint32")
    if tag == "Agg":
        return _value(payload, table)
    if tag == "Count":
        if isinstance(payload, list):
            payload = {"input": payload[0], "include_nulls": payload[1]}
        info = _fields(payload)
        value = _value(info["input"], table)
        return (
            table.count() if info["include_nulls"] else value.count()
        ).cast("uint32")
    if tag == "Cast":
        info = _fields(payload)
        dtype_tag, dtype = _node(info["dtype"])
        if dtype_tag != "Literal":
            raise NotImplementedError("Non-literal cast targets are unsupported")
        return _cast(_value(info["expr"], table), dtype, info["options"])
    if tag == "Sum":
        value = _value(payload, table)
        dtype = value.type()
        # Polars promotes small signed/unsigned integer sums to Int64 and
        # Boolean sums to UInt32; other sums retain their logical dtype.
        if dtype.is_boolean():
            output = ibis.dtype("uint32")
        elif dtype.is_integer() and dtype.nbytes < 4:
            output = ibis.dtype("int64")
        else:
            output = dtype
        return value.sum().fill_null(ibis.literal(0, type=output)).cast(output)
    if tag == "Mean":
        value = _value(payload, table)
        dtype = value.type()
        # Wider accumulation is permitted, but the continuation consumes
        # the Polars output dtype. This is not a bitwise equivalence claim.
        result = value.cast("float64").mean()
        return result.cast("float32") if dtype.is_float32() else result
    if tag in ("Min", "Max"):
        info = _fields(payload)
        if info.get("propagate_nans") is not False:
            raise NotImplementedError("NaN-propagating extrema are unsupported")
        value = _value(info["input"], table)
        return value.min() if tag == "Min" else value.max()
    if tag in ("Var", "Std"):
        if not isinstance(payload, list) or len(payload) != 2 or payload[1] != 1:
            raise NotImplementedError(f"Unsupported {tag} options")
        value = _value(payload[0], table)
        result = value.cast("float64")
        result = result.var() if tag == "Var" else result.std()
        return result.cast("float32") if value.type().is_float32() else result
    if tag == "BinaryExpr":
        info = _fields(payload)
        if set(info) != {"left", "op", "right"} or info["op"] not in _BINARY:
            raise NotImplementedError(f"Unsupported binary operator: {info!r}")
        return _BINARY[info["op"]](
            _value(info["left"], table), _value(info["right"], table)
        )
    if tag == "Ternary":
        info = _fields(payload)
        if set(info) != {"predicate", "truthy", "falsy"}:
            raise NotImplementedError(f"Unsupported ternary: {info!r}")
        return _value(info["predicate"], table).ifelse(
            _value(info["truthy"], table), _value(info["falsy"], table)
        )
    if tag == "Function":
        info = _fields(payload)
        function = info["function"]
        args = info["input"]
        if function == "FillNull" and len(args) == 2:
            return _value(args[0], table).fill_null(_value(args[1], table))
        if isinstance(function, dict) and "Clip" in function and len(args) == 3:
            opts = function["Clip"]
            if opts.get("has_min") is True and opts.get("has_max") is True:
                return _value(args[0], table).clip(
                    _value(args[1], table), _value(args[2], table)
                )
        if isinstance(function, dict) and "Boolean" in function and len(args) == 1:
            op = function["Boolean"]
            val = _value(args[0], table)
            if op == "IsNull":
                return val.isnull()
            if op == "IsNotNull":
                return val.notnull()
            if op == "IsNan":
                return val.isnan()
            if op == "IsNotNan":
                return ~val.isnan()
            if op == "Not":
                return ~val
        if function == "Negate" and len(args) == 1:
            return -_value(args[0], table)
    raise NotImplementedError(f"Unsupported Polars value operation: {tag} {payload!r}")


def _name(expr):
    tag, payload = _node(expr)
    if tag == "Alias":
        return payload[1]
    if tag == "Column":
        return payload
    if tag == "Len":
        return "len"
    if tag == "Literal":
        return "literal"
    if tag == "Agg":
        return _name(payload)
    if tag in ("Sum", "Mean", "Min", "Max"):
        return _name(payload["input"] if isinstance(payload, dict) else payload)
    if tag == "Count":
        return _name(payload[0] if isinstance(payload, list) else payload["input"])
    if tag == "Cast":
        return _name(payload["expr"])
    if tag in ("Var", "Std"):
        return _name(payload[0])
    if tag == "Function":
        return _name(payload["input"][0])
    if tag == "Ternary":
        return _name(payload["truthy"])
    if tag == "BinaryExpr":
        # Polars uses the left expression's name unless it is a literal.
        child = payload["left"]
        return _name(payload["right"] if _node(child)[0] == "Literal" else child)
    raise NotImplementedError(f"Unsupported output naming: {tag}")


def _named(exprs, table):
    out = {}
    for expr in exprs:
        tag, payload = _node(expr)
        if tag == "Selector" and isinstance(payload, dict):
            sel, opts = _node(payload)
            if sel == "ByName" and opts.get("strict") is True:
                for name in opts["names"]:
                    if name not in table.columns or name in out:
                        raise ValueError(f"Missing or duplicate column: {name}")
                    out[name] = table[name]
                continue
        name = _name(expr)
        if name in out:
            raise ValueError(f"Duplicate output column: {name}")
        out[name] = _value(expr, table)
    return out


def _plan(plan, table):
    tag, payload = _node(plan)
    if tag == "IR":
        if set(payload) != {"dsl", "version"}:
            raise NotImplementedError("Unsupported cached IR")
        return _plan(payload["dsl"], table)
    if tag == "DataFrameScan":
        fields = payload["schema"]["fields"]
        expected = {name: _type_name(dtype) for name, dtype in fields.items()}
        actual = {name: str(dtype) for name, dtype in table.schema().items()}
        if expected != actual:
            raise ValueError(
                f"Bound database schema does not match Polars scan: {expected} != {actual}"
            )
        return table
    if tag == "Select":
        source = _plan(payload["input"], table)
        values = _named(payload["expr"], source)
        if not values:
            raise NotImplementedError("Empty projections are unsupported")
        scalars = [value for value in values.values() if isinstance(value, ir.Scalar)]
        if len(scalars) == len(values):
            return source.aggregate(**values)
        if any(value.op().relations for value in scalars):
            raise NotImplementedError(
                "Mixed reductions and row-wise projections are unsupported"
            )
        return source.select(**values)
    if tag == "HStack":
        source = _plan(payload["input"], table)
        values = _named(payload["exprs"], source)
        if any(
            isinstance(v, ir.Scalar) and v.op().relations for v in values.values()
        ):
            raise NotImplementedError("Aggregation inside with_columns is unsupported")
        return source.mutate(**values)
    if tag == "GroupBy":
        if payload.get("apply") is not None or payload.get("maintain_order"):
            raise NotImplementedError("GroupBy callbacks and ordering are unsupported")
        if payload.get("predicates"):
            raise NotImplementedError("GroupBy predicates are unsupported")
        if any(v is not None for v in payload.get("options", {}).values()):
            raise NotImplementedError("Non-default GroupBy options are unsupported")
        source = _plan(payload["input"], table)
        keys = _named(payload["keys"], source)
        metrics = _named(payload["aggs"], source)
        if set(keys) & set(metrics):
            raise ValueError("GroupBy key and aggregate output names must be distinct")
        if not metrics or any(not isinstance(v, ir.Scalar) for v in metrics.values()):
            raise NotImplementedError("GroupBy requires scalar aggregation expressions")
        return source.group_by(**keys).aggregate(**metrics)
    if tag == "Filter":
        source = _plan(payload["input"], table)
        predicate = _value(payload["predicate"], source)
        if isinstance(predicate, ir.Scalar):
            raise NotImplementedError("Scalar filters are unsupported")
        return source.filter(predicate)
    raise NotImplementedError(f"Unsupported deterministic Polars plan: {tag}")


def convert_polars_to_ibis(query, *, table):
    """Lower an OpenDP deterministic head onto an already-bound Ibis relation.

    The splitter controls which private computations can reach this boundary.
    The lowerer does not inspect or execute the private input, and rejects all
    unrecognized plan nodes. This is *not* a validator of sensitivity bounds.
    """
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", category=UserWarning)
        plan = json.loads(query.serialize(format="json"))
    return _plan(plan, table)
