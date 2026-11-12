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
use std::io::Cursor;

use polars::prelude::DslPlan;
use rmpv::Value;

use crate::error::Fallible;

const DSL_MAGIC: &[u8] = b"DSL_VERSION";
const HEADER_LENGTH: usize = DSL_MAGIC.len() + 4 + 64;
const MAX_PLAN_BYTES: usize = 64 * 1024 * 1024;
const MAX_NESTING: usize = 128;

// This list is intentionally broader than the set of known code-execution
// paths. False-positive rejection is preferable to running a Python callback.
// Audit ALL callback-bearing variants before updating the pinned Polars build.
const FORBIDDEN_VARIANTS: &[&str] = &[
    "PythonScan",     // eagerly deserializes scan_fn/schema_fn
    "OpaquePython",   // eagerly deserializes PythonFunction
    "AnonymousFunction", // lazily deserializes Python UDFs
    "Python",         // PlanCallback::Python (including nested function options)
    "RenameAlias",    // callback applied to a column name
    "PipeWithDtype",  // expression callback
    "PipeWithSchema", // plan callback
    "FoldHorizontal", // function expression callback
    "ReduceHorizontal",
    "CumFoldHorizontal",
    "CumReduceHorizontal",
    "Map",            // RollingFunction::Map callback
    "MapFieldNames",  // struct-field callback
    "SubPlan",        // an independently supplied nested plan
    "Scan",           // user-controlled external sources and providers
    "Sink",           // user-controlled output destination
    "SinkMultiple",
];

// These are callback slots in other (otherwise admissible) constructs.
// Null callbacks are fine, but a non-null callback must never reach Polars.
const FORBIDDEN_NON_NULL_FIELDS: &[&str] = &[
    "apply", "callback", "scan_fn", "schema_fn",
];

fn invalid(message: &str) -> crate::error::Error {
    err!(FFI, "Unsafe or unsupported serialized Polars plan: {}", message)
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
    let mut cursor = Cursor::new(payload);
    let value = rmpv::decode::read_value(&mut cursor)
        .map_err(|e| err!(FFI, "Malformed Polars MessagePack: {}", e))?;
    if cursor.position() != payload.len() as u64 {
        return Err(invalid("trailing data"));
    }

    // Polars' versioned DSL encodes SerializableDslPlan as a struct-map.
    // Refuse formats that do not match this expected top-level structure.
    let Value::Map(root) = &value else {
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

    check_value(&value, 0)
}

/// Traverse inert MessagePack data. Do not instantiate any Polars/Serde types.
fn check_value(value: &Value, depth: usize) -> Fallible<()> {
    if depth > MAX_NESTING {
        return Err(invalid("plan nesting limit exceeded"));
    }
    match value {
        Value::Map(pairs) => {
            let mut seen = HashSet::new();
            for (key, child) in pairs {
                if let Some(name) = key.as_str() {
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
                    if FORBIDDEN_NON_NULL_FIELDS.contains(&name) && !matches!(child, Value::Nil) {
                        return Err(err!(
                            FFI,
                            "Serialized Polars plan contains a callback in field {}",
                            name
                        ));
                    }
                }
                check_value(key, depth + 1)?;
                check_value(child, depth + 1)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                check_value(child, depth + 1)?;
            }
        }
        Value::String(text) => {
            // Unit enum variants may be represented as strings rather than
            // one-entry maps. This can reject benign literals with such names.
            if text.as_str().is_some_and(|s| FORBIDDEN_VARIANTS.contains(&s)) {
                return Err(invalid("forbidden unit variant or string value"));
            }
        }
        Value::Ext(_, _) => return Err(invalid("MessagePack extensions are unsupported")),
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
            assert!(validate_serialized_dslplan(&encoded(&bad)).is_err(), "{name}");
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
