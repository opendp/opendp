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
use rmpv::decode::value_ref::BorrowRead;
use rmpv::ValueRef;
#[cfg(test)]
use rmpv::Value;

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
    "Python",            // PlanCallback::Python (including nested function options)
    "RenameAlias",       // callback applied to a column name
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

// These are callback slots in other (otherwise admissible) constructs.
// Null callbacks are fine, but a non-null callback must never reach Polars.
const FORBIDDEN_NON_NULL_FIELDS: &[&str] = &["apply", "callback", "scan_fn", "schema_fn"];

fn invalid(message: &str) -> crate::error::Error {
    err!(
        FFI,
        "Unsafe or unsupported serialized Polars plan: {}",
        message
    )
}

/// Validate a versioned serialized DslPlan *before* Polars deserializes it.
///
/// Requiring the exact header prevents a new Polars schema or minor version
/// from silently introducing a new Python-object deserialization path. This
/// check cannot be bypassed by POLARS_SKIP_DSL_HASH_VERIFICATION.
pub(crate) fn validate_serialized_dslplan(raw: &[u8]) -> Fallible<()> {
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
    Ok(())
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
        return Err(err!(FFI, "Unsupported serialized Polars plan node: {}", name));
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
                if FORBIDDEN_VARIANTS.contains(&name) {
                    return Err(err!(
                        FFI,
                        "Serialized Polars plan contains a forbidden construct: {}",
                        name
                    ));
                }
                if FORBIDDEN_NON_NULL_FIELDS.contains(&name) && !matches!(child, ValueRef::Nil) {
                    return Err(err!(
                        FFI,
                        "Serialized Polars plan contains a callback in field {}",
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
            let bad = root_with(Value::Array(vec![Value::Map(vec![(
                Value::String((*name).into()),
                Value::Nil,
            )])]));
            assert!(
                validate_serialized_dslplan(&encoded(&bad)).is_err(),
                "{name}"
            );
        }
        let bad = root_with(Value::Map(vec![(
            Value::String("apply".into()),
            Value::Array(vec![Value::Bool(true)]),
        )]));
        assert!(validate_serialized_dslplan(&encoded(&bad)).is_err());
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
