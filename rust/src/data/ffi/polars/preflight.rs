//! Data-only preflight for untrusted, serialized Polars DSL plans.
//!
//! The Polars 1.36.1 DSL is a versioned header followed by MessagePack. Polars
//! deserializes some Python callbacks eagerly; it is therefore unsafe to call
//! `DslPlan::deserialize_versioned` and then inspect the resulting plan.
//!
//! The preflight does not deserialize into any Polars types. It rejects known
//! executable variants/callbacks, checks for duplicate MessagePack string keys,
//! limits input size and nesting, and requires the exact DSL header produced by
//! this build. The approved bytes are passed unchanged to Polars afterwards.
//!
//! This guard is specific to versioned DslPlan serialization. The compact
//! Expr serialization format must not be passed to this validator.

use std::collections::HashSet;
use std::io::{self, Read};

use polars::prelude::DslPlan;
use rmpv::{Value, ValueRef};
use rmpv::decode::value_ref::BorrowRead;

use crate::error::Fallible;

const DSL_MAGIC: &[u8] = b"DSL_VERSION";
const HEADER_LENGTH: usize = DSL_MAGIC.len() + 4 + 64;
pub(crate) const MAX_PLAN_BYTES: usize = 64 * 1024 * 1024;
const MAX_NESTING: usize = 128;
const MAX_PARSE_READS: usize = 1_000_000;

// The borrowed-value parser avoids copying binary payloads, but still
// allocates for array/map elements. Bound its work *during* decoding so a
// small message containing millions of scalars cannot exhaust memory.
struct BoundedReader<'a> {
    remaining: &'a [u8],
    reads: usize,
}

impl Read for BoundedReader<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.reads >= MAX_PARSE_READS {
            return Err(io::Error::other("MessagePack element limit exceeded"));
        }
        self.reads += 1;
        let len = out.len().min(self.remaining.len());
        out[..len].copy_from_slice(&self.remaining[..len]);
        self.remaining = &self.remaining[len..];
        Ok(len)
    }
}

impl<'a> BorrowRead<'a> for BoundedReader<'a> {
    fn fill_buf(&self) -> &'a [u8] {
        self.remaining
    }

    fn consume(&mut self, len: usize) {
        self.remaining = &self.remaining[len..];
    }
}

// This list is intentionally broader than the set of known code-execution
// paths. False-positive rejection is preferable to running a Python callback.
// Audit ALL callback-bearing variants before updating the pinned Polars build.
const FORBIDDEN_VARIANTS: &[&str] = &[
    "PythonScan",        // eagerly deserializes scan_fn/schema_fn
    "OpaquePython",      // eagerly deserializes PythonFunction
    "AnonymousFunction", // lazily deserializes Python UDFs
    "PipeWithDtype",     // expression callback
    "PipeWithSchema",    // plan callback
    "FoldHorizontal",    // function expression callback
    "ReduceHorizontal",
    "CumFoldHorizontal",
    "CumReduceHorizontal",
    "Map",           // RollingFunction::Map callback
    "MapFieldNames", // struct-field callback
    "SubPlan",       // an independently supplied nested plan
    "Scan",          // user-controlled external sources and providers
    "Sink",          // user-controlled output destination
    "SinkMultiple",
];

// Only these Polars 1.36.1 plan node variants can cross the untrusted
// serialization boundary. Nested plan nodes must be checked as well.
const ALLOWED_PLAN_NODES: &[&str] = &[
    "Filter",
    "Cache",
    "DataFrameScan",
    "Select",
    "GroupBy",
    "Join",
    "HStack",
    "MatchToSchema",
    "MapFunction",
    "Distinct",
    "Sort",
    "Slice",
    "Union",
    "HConcat",
    "ExtContext",
    "Pivot",
    "MergeSorted",
    "IR",
];

fn invalid(message: &str) -> crate::error::Error {
    err!(
        FFI,
        "Unsafe or unsupported serialized Polars plan: {}",
        message
    )
}

// Only client-side OpenDP shims may be submitted. The values mirror the
// FunctionFlags set by Python's register_plugin_function in Polars 1.36.1.
fn shim_flags(symbol: &str) -> Option<u64> {
    Some(match symbol {
        "noise" => 192, // ROW_SEPARABLE | LENGTH_PRESERVING
        "dp_len" | "dp_count" | "dp_null_count" | "dp_n_unique" | "dp_frame_len" => 136,
        "dp_sum" | "dp_mean" | "dp_quantile" | "dp_median" => 8,
        _ => return None,
    })
}

fn check_ffi_plugin(plugin: &ValueRef<'_>) -> Fallible<()> {
    let ValueRef::Map(fields) = plugin else {
        return Err(invalid("invalid FfiPlugin"));
    };
    let field = |name: &str| {
        fields
            .iter()
            .find(|(key, _)| key.as_str() == Some(name))
            .map(|(_, value)| value)
    };
    if fields.len() != 4 || field("lib").and_then(ValueRef::as_str).is_none() {
        return Err(invalid("invalid FfiPlugin fields"));
    }
    let symbol = field("symbol")
        .and_then(ValueRef::as_str)
        .ok_or_else(|| invalid("missing plugin symbol"))?;
    let expected = shim_flags(symbol).ok_or_else(|| invalid("unsupported native plugin"))?;
    match field("kwargs") {
        Some(ValueRef::Binary([])) | Some(ValueRef::Array(ref empty)) if empty.is_empty() => {},
        _ => return Err(invalid("plugin keyword arguments are forbidden")),
    }
    let Some(ValueRef::Map(options)) = field("flags") else {
        return Err(invalid("invalid plugin options"));
    };
    let option = |name: &str| {
        options
            .iter()
            .find(|(key, _)| key.as_str() == Some(name))
            .map(|(_, value)| value)
    };
    if options.len() != 2
        || !matches!(option("check_lengths"), Some(ValueRef::Boolean(true)))
        || option("flags").and_then(ValueRef::as_u64) != Some(expected)
    {
        return Err(invalid("untrusted plugin execution flags"));
    }
    Ok(())
}

/// Validate a versioned serialized DslPlan *before* Polars deserializes it.
///
/// Requiring the exact header prevents a new Polars schema or minor version
/// from silently introducing a new Python-object deserialization path. This
/// check cannot be bypassed by POLARS_SKIP_DSL_HASH_VERIFICATION.
pub(crate) fn validate_serialized_dslplan(raw: &[u8]) -> Fallible<()> {
    inspect_serialized_dslplan(raw).map(|_| ())
}

// Rebind plugin paths in MessagePack, before Polars can load a shared library
// during expression/schema reconstruction. No client-provided library path is
// carried into the reconstructed plan.
pub(crate) fn sanitize_serialized_dslplan(raw: &[u8], trusted_lib: &str) -> Fallible<Vec<u8>> {
    if trusted_lib.is_empty() {
        return Err(invalid("server plugin library path is not configured"));
    }
    let mut value = inspect_serialized_dslplan(raw)?.to_owned();
    if !rebind_plugins(&mut value, trusted_lib) {
        return Ok(raw.to_vec());
    }
    let mut output = raw[..HEADER_LENGTH].to_vec();
    rmpv::encode::write_value(&mut output, &value)
        .map_err(|e| err!(FFI, "Unable to rewrite Polars plan: {}", e))?;
    Ok(output)
}

fn rebind_plugins(value: &mut Value, trusted_lib: &str) -> bool {
    match value {
        Value::Map(fields) => {
            let mut rewritten = false;
            for (key, child) in fields {
                if key.as_str() == Some("FfiPlugin") {
                    let Value::Map(plugin) = child else { unreachable!() };
                    let (_, path) = plugin
                        .iter_mut()
                        .find(|(name, _)| name.as_str() == Some("lib"))
                        .expect("validated plugin has lib");
                    *path = Value::String(trusted_lib.into());
                    rewritten = true;
                } else {
                    rewritten |= rebind_plugins(child, trusted_lib);
                }
            }
            rewritten
        }
        Value::Array(values) => values
            .iter_mut()
            .fold(false, |changed, child| rebind_plugins(child, trusted_lib) | changed),
        _ => false,
    }
}

fn inspect_serialized_dslplan(raw: &[u8]) -> Fallible<ValueRef<'_>> {
    if raw.len() > MAX_PLAN_BYTES {
        return Err(invalid("plan exceeds the size limit"));
    }
    if raw.len() < HEADER_LENGTH || !raw.starts_with(DSL_MAGIC) {
        return Err(invalid("not a versioned Polars DslPlan"));
    }

    // Generate the expected header from *this* compiled Polars build. No
    // handwritten schema hash and no dependence on an environment override.
    let mut reference = Vec::new();
    DslPlan::default()
        .serialize_versioned(&mut reference, Default::default())
        .map_err(|e| err!(FFI, "Unable to determine Polars DSL version: {}", e))?;
    if raw[..HEADER_LENGTH] != reference[..HEADER_LENGTH] {
        return Err(invalid("Polars DSL version or schema hash mismatch"));
    }

    let payload = &raw[HEADER_LENGTH..];
    let mut reader = BoundedReader {
        remaining: payload,
        reads: 0,
    };
    let value = rmpv::decode::read_value_ref_with_max_depth(&mut reader, MAX_NESTING)
        .map_err(|e| err!(FFI, "Malformed Polars MessagePack: {}", e))?;
    if !reader.remaining.is_empty() {
        return Err(invalid("trailing data"));
    }

    // Polars' versioned DSL encodes SerializableDslPlan as a struct-map.
    // Refuse formats that do not match this expected top-level structure.
    let ValueRef::Map(root) = &value else {
        return Err(invalid("expected a serialized plan map"));
    };
    let mut fields = HashSet::new();
    for (key, _) in root {
        let Some(key) = key.as_str() else {
            return Err(invalid("unexpected non-string root field"));
        };
        if !fields.insert(key) {
            return Err(invalid("duplicate root field"));
        }
    }
    if fields != HashSet::from(["root", "dataframes", "dsl_plans"]) {
        return Err(invalid("unrecognized serialized plan structure"));
    }

    check_value(&value, 0)?;

    // SlotMap encodes each node as {value: Option<Node>, version: u32}.
    // Checking each enum tag against an allowlist is stronger than trying to
    // anticipate every future executable plan variant.
    let nodes = root
        .iter()
        .find(|(key, _)| key.as_str() == Some("dsl_plans"))
        .map(|(_, value)| value)
        .ok_or_else(|| invalid("missing plan nodes"))?;
    let ValueRef::Array(slots) = nodes else {
        return Err(invalid("invalid plan node collection"));
    };
    for slot in slots {
        let ValueRef::Map(fields) = slot else {
            return Err(invalid("invalid plan slot"));
        };
        let node = fields
            .iter()
            .find(|(key, _)| key.as_str() == Some("value"))
            .map(|(_, value)| value)
            .ok_or_else(|| invalid("missing plan slot value"))?;
        if !matches!(node, ValueRef::Nil) {
            check_plan_node(node)?;
        }
    }
    Ok(value)
}

/// Check the serialized plan variant without constructing any Polars objects.
fn check_plan_node(node: &ValueRef<'_>) -> Fallible<()> {
    let ValueRef::Map(variant) = node else {
        return Err(invalid("invalid plan node"));
    };
    if variant.len() != 1 {
        return Err(invalid("expected a single plan node variant"));
    }
    let (tag, payload) = &variant[0];
    let Some(name) = tag.as_str() else {
        return Err(invalid("non-string plan node variant"));
    };
    if !ALLOWED_PLAN_NODES.contains(&name) {
        return Err(err!(
            FFI,
            "Unsupported serialized Polars plan node: {}",
            name
        ));
    }
    if name == "GroupBy" {
        let ValueRef::Map(fields) = payload else {
            return Err(invalid("invalid GroupBy plan node"));
        };
        if fields
            .iter()
            .any(|(key, value)| key.as_str() == Some("apply") && !matches!(value, ValueRef::Nil))
        {
            return Err(invalid("group-by callbacks are unsupported"));
        }
    }
    if matches!(name, "Union" | "HConcat" | "ExtContext") {
        let ValueRef::Map(fields) = payload else {
            return Err(invalid("invalid nested plan node"));
        };
        let nested = if name == "ExtContext" {
            "contexts"
        } else {
            "inputs"
        };
        let children = fields
            .iter()
            .find(|(key, _)| key.as_str() == Some(nested))
            .map(|(_, value)| value)
            .ok_or_else(|| invalid("missing nested plan nodes"))?;
        let ValueRef::Array(children) = children else {
            return Err(invalid("invalid nested plan nodes"));
        };
        for child in children {
            check_plan_node(child)?;
        }
    }
    Ok(())
}

/// Traverse inert MessagePack data. Do not instantiate any Polars/Serde types.
fn check_value(value: &ValueRef<'_>, depth: usize) -> Fallible<()> {
    if depth > MAX_NESTING {
        return Err(invalid("plan nesting limit exceeded"));
    }
    match value {
        ValueRef::Map(pairs) => {
            let mut seen = HashSet::new();
            for (key, child) in pairs {
                // Versioned Polars DSL uses string-keyed struct maps. Reject
                // alternate key encodings rather than allowing an enum tag to
                // evade inspection.
                let Some(name) = key.as_str() else {
                    return Err(invalid("non-string MessagePack map field"));
                };
                if !seen.insert(name) {
                    return Err(invalid("duplicate MessagePack map field"));
                }
                if name == "FfiPlugin" {
                    check_ffi_plugin(child)?;
                }
                if FORBIDDEN_VARIANTS.contains(&name) {
                    return Err(err!(
                        FFI,
                        "Serialized Polars plan contains a forbidden construct: {}",
                        name
                    ));
                }
                check_value(child, depth + 1)?;
            }
        }
        ValueRef::Array(values) => {
            for child in values {
                check_value(child, depth + 1)?;
            }
        }
        ValueRef::Ext(_, _) => return Err(invalid("MessagePack extensions are unsupported")),
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(body: &Value) -> Vec<u8> {
        let mut header = Vec::new();
        DslPlan::default()
            .serialize_versioned(&mut header, Default::default())
            .unwrap();
        header.truncate(HEADER_LENGTH);
        rmpv::encode::write_value(&mut header, body).unwrap();
        header
    }

    fn root_with(value: Value) -> Value {
        let string = |s: &str| Value::String(s.into());
        Value::Map(vec![
            (string("root"), Value::Nil),
            (string("dataframes"), Value::Nil),
            (string("dsl_plans"), value),
        ])
    }

    fn slot(variant: &str) -> Value {
        Value::Map(vec![
            (
                Value::String("value".into()),
                Value::Map(vec![(Value::String(variant.into()), Value::Nil)]),
            ),
            (Value::String("version".into()), Value::from(1)),
        ])
    }

    #[test]
    fn inspects_every_node_behind_ir_wrapper() {
        use std::sync::Arc;

        let wrapped = DslPlan::IR {
            dsl: Arc::new(DslPlan::default()),
            version: 0,
            node: None,
        };
        let mut serialized = Vec::new();
        wrapped
            .serialize_versioned(&mut serialized, Default::default())
            .unwrap();
        validate_serialized_dslplan(&serialized).unwrap();

        // Polars deserializes every slot, not just nodes reachable from the
        // root. An IR wrapper must not conceal an eagerly-unpickled slot.
        let mut payload = rmpv::decode::read_value(
            &mut std::io::Cursor::new(&serialized[HEADER_LENGTH..]),
        )
        .unwrap();
        let Value::Map(root) = &mut payload else {
            panic!("expected struct map");
        };
        let Value::Array(slots) = &mut root
            .iter_mut()
            .find(|(name, _)| name.as_str() == Some("dsl_plans"))
            .unwrap()
            .1
        else {
            panic!("expected node slot map");
        };
        slots.push(slot("PythonScan"));
        assert!(validate_serialized_dslplan(&encoded(&payload)).is_err());
    }

    #[test]
    fn accepts_native_versioned_plan() {
        let mut bytes = Vec::new();
        DslPlan::default()
            .serialize_versioned(&mut bytes, Default::default())
            .unwrap();
        validate_serialized_dslplan(&bytes).unwrap();
    }

    #[test]
    fn refuses_python_and_callback_variants_before_polars_deserialization() {
        for name in FORBIDDEN_VARIANTS {
            let bad = root_with(Value::Array(vec![slot(name)]));
            assert!(
                validate_serialized_dslplan(&encoded(&bad)).is_err(),
                "{name}"
            );
        }
        let bad = root_with(Value::Array(vec![Value::Map(vec![
            (
                Value::String("value".into()),
                Value::Map(vec![(
                    Value::String("GroupBy".into()),
                    Value::Map(vec![(
                        Value::String("apply".into()),
                        Value::Bool(true),
                    )]),
                )]),
            ),
            (Value::String("version".into()), Value::from(1)),
        ])]));
        let error = validate_serialized_dslplan(&encoded(&bad)).unwrap_err();
        assert!(format!("{error:?}").contains("group-by callbacks"));
    }

    #[test]
    fn refuses_bad_header_duplicates_trailing_bytes_and_unknown_structure() {
        let mut bytes = Vec::new();
        DslPlan::default()
            .serialize_versioned(&mut bytes, Default::default())
            .unwrap();
        let mut bad_header = bytes.clone();
        bad_header[DSL_MAGIC.len()] ^= 1;
        assert!(validate_serialized_dslplan(&bad_header).is_err());

        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(validate_serialized_dslplan(&trailing).is_err());

        let duplicate = Value::Map(vec![
            (Value::String("root".into()), Value::Nil),
            (Value::String("root".into()), Value::Nil),
            (Value::String("dataframes".into()), Value::Nil),
            (Value::String("dsl_plans".into()), Value::Nil),
        ]);
        assert!(validate_serialized_dslplan(&encoded(&duplicate)).is_err());

        assert!(validate_serialized_dslplan(&encoded(&Value::Nil)).is_err());
    }
}
