//! Data-only preflight for untrusted, serialized Polars DSL plans.
//!
//! The Polars DSL is a versioned header followed by MessagePack. Polars
//! deserializes some Python callbacks eagerly; it is therefore unsafe to call
//! `DslPlan::deserialize_versioned` and then inspect the resulting plan.
//!

use std::collections::HashSet;
use std::io::{self, Read};

use polars::prelude::DslPlan;
use rmpv::ValueRef;
use rmpv::decode::value_ref::BorrowRead;

#[cfg(test)]
use rmpv::Value;

use crate::error::Fallible;

const DSL_MAGIC: &[u8] = b"DSL_VERSION";
const HEADER_LENGTH: usize = DSL_MAGIC.len() + 4 + 64;
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

// The plan-node allowlist separately rejects Python scans and callbacks at
// the plan level. These variants embed executable code in expression trees.
const FORBIDDEN_EXPR_VARIANTS: &[&str] = &[
    "OpaquePython",
    "AnonymousFunction",
    "PipeWithDtype",
    "FoldHorizontal",
    "ReduceHorizontal",
    "CumFoldHorizontal",
    "CumReduceHorizontal",
    "Map",
    "MapFieldNames",
    "SubPlan",
];

// Reject callbacks and external I/O before Polars deserializes the plan.
const FORBIDDEN_PLAN_VARIANTS: &[&str] = &[
    "PythonScan",
    "PipeWithSchema",
    "Sink",
    "SinkMultiple",
];

fn invalid(message: &str) -> crate::error::Error {
    err!(
        FFI,
        "Unsafe or unsupported serialized Polars plan: {}",
        message
    )
}

// Only DP expressions exposed by OpenDP may be submitted. The executable
// plugin's flags and arguments are constructed later by OpenDP.
fn is_supported_plugin(symbol: &str) -> bool {
    matches!(
        symbol,
        "noise"
            | "dp_len"
            | "dp_count"
            | "dp_null_count"
            | "dp_n_unique"
            | "dp_frame_len"
            | "dp_sum"
            | "dp_mean"
            | "dp_quantile"
            | "dp_median"
    )
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
    if !is_supported_plugin(symbol) {
        return Err(invalid("unsupported native plugin"));
    }
    match field("kwargs") {
        Some(ValueRef::Binary(bytes)) if bytes.is_empty() => {}
        Some(ValueRef::Array(values)) if values.is_empty() => {}
        _ => return Err(invalid("plugin keyword arguments are forbidden")),
    }
    Ok(())
}

/// Validate a versioned serialized DslPlan *before* Polars deserializes it.
///
/// Requiring the exact header prevents a new Polars schema or minor version
/// from silently introducing a new Python-object deserialization path. This
/// check cannot be bypassed by POLARS_SKIP_DSL_HASH_VERIFICATION.
pub(crate) fn validate_serialized_dslplan(raw: &[u8]) -> Fallible<()> {
    inspect_serialized_dslplan(raw, None, &[]).map(|_| ())
}

// Rebind plugin paths in MessagePack before Polars can load a shared library.
// The borrowed value tree can be rewritten without copying the entire payload.
pub(crate) fn sanitize_serialized_dslplan(
    raw: &[u8],
    trusted_lib: &str,
    allowed_files: &[String],
) -> Fallible<Vec<u8>> {
    if trusted_lib.is_empty() {
        return Err(invalid("server plugin library path is not configured"));
    }
    let (value, rewritten) = inspect_serialized_dslplan(raw, Some(trusted_lib), allowed_files)?;
    if !rewritten {
        return Ok(raw.to_vec());
    }
    let mut output = raw[..HEADER_LENGTH].to_vec();
    rmpv::encode::write_value_ref(&mut output, &value)
        .map_err(|e| err!(FFI, "Unable to rewrite Polars plan: {}", e))?;
    Ok(output)
}

fn inspect_serialized_dslplan<'a>(
    raw: &'a [u8],
    trusted_lib: Option<&'a str>,
    allowed_files: &[String],
) -> Fallible<(ValueRef<'a>, bool)> {
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
        let version_offset = DSL_MAGIC.len();
        let version_end = version_offset + 4;
        if raw[version_offset..version_end] != reference[version_offset..version_end] {
            let version = |header: &[u8]| {
                let major = u16::from_le_bytes([header[version_offset], header[version_offset + 1]]);
                let minor =
                    u16::from_le_bytes([header[version_offset + 2], header[version_offset + 3]]);
                format!("{major}.{minor}")
            };
            return Err(err!(
                FFI,
                "Cannot deserialize Polars plan. The client's DSL version ({}) differs from the server's ({}). Use compatible Polars versions on the client and server.",
                version(raw),
                version(&reference)
            ));
        }
        return Err(err!(
            FFI,
            "Cannot deserialize Polars plan. Its serialization schema does not match the server's Polars build. Use compatible Polars versions and builds on the client and server."
        ));
    }

    let payload = &raw[HEADER_LENGTH..];
    let mut reader = BoundedReader {
        remaining: payload,
        reads: 0,
    };
    let mut value = rmpv::decode::read_value_ref_with_max_depth(&mut reader, MAX_NESTING)
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

    // SlotMap encodes each node as {value: Option<Node>, version: u32}.
    // Inspect every node, including those not reachable from the root.
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
            check_plan_node(node, allowed_files)?;
        }
    }
    let rewritten = check_value(&mut value, 0, trusted_lib)?;
    Ok((value, rewritten))
}

// Only exact local Parquet files approved by the server can be scanned.
// All other file formats, cloud sources and Python-backed scan providers are rejected.
fn check_file_scan(scan: &ValueRef<'_>, allowed_files: &[String]) -> Fallible<()> {
    if allowed_files.is_empty() {
        return Err(invalid("file scans are not allowed"));
    }
    let ValueRef::Map(fields) = scan else {
        return Err(invalid("invalid file scan"));
    };
    let field = |name: &str| {
        fields
            .iter()
            .find(|(key, _)| key.as_str() == Some(name))
            .map(|(_, value)| value)
    };
    if fields.len() != 3 {
        return Err(invalid("invalid file scan fields"));
    }

    let Some(ValueRef::Map(scan_type)) = field("scan_type") else {
        return Err(invalid("invalid scan type"));
    };
    if scan_type.len() != 1 || scan_type[0].0.as_str() != Some("Parquet") {
        return Err(invalid("only local Parquet file scans are supported"));
    }

    let Some(ValueRef::Map(options)) = field("unified_scan_args") else {
        return Err(invalid("invalid scan options"));
    };
    for name in ["cloud_options", "deletion_files"] {
        if options.iter().any(|(key, value)| {
            key.as_str() == Some(name) && !matches!(value, ValueRef::Nil)
        }) {
            return Err(invalid("external scan options are unsupported"));
        }
    }

    let Some(ValueRef::Map(sources)) = field("sources") else {
        return Err(invalid("invalid scan sources"));
    };
    if sources.len() != 1 || sources[0].0.as_str() != Some("Paths") {
        return Err(invalid("only local file paths are supported"));
    }
    let ValueRef::Array(paths) = &sources[0].1 else {
        return Err(invalid("invalid scan paths"));
    };
    if paths.is_empty() {
        return Err(invalid("empty file scan"));
    }
    for path in paths {
        let ValueRef::Map(source) = path else {
            return Err(invalid("invalid local file path"));
        };
        if source.len() != 1 || source[0].0.as_str() != Some("Local") {
            return Err(invalid("only local file paths are supported"));
        }
        let Some(path) = source[0].1.as_str() else {
            return Err(invalid("invalid local file path"));
        };
        // Reject glob expansion and path aliases. The caller supplies canonical
        // absolute paths, and each serialized path must match one of them.
        if !allowed_files.iter().any(|allowed| allowed.as_str() == path) {
            return Err(invalid("file scan path is not allowed"));
        }
    }
    Ok(())
}

/// Check the serialized plan variant without constructing any Polars objects.
fn check_plan_node(node: &ValueRef<'_>, allowed_files: &[String]) -> Fallible<()> {
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
    if FORBIDDEN_PLAN_VARIANTS.contains(&name) {
        return Err(err!(
            FFI,
            "Serialized Polars plan contains a forbidden plan node: {}",
            name
        ));
    }
    if name == "Scan" {
        return check_file_scan(payload, allowed_files);
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
            check_plan_node(child, allowed_files)?;
        }
    }
    Ok(())
}

/// Inspect inert MessagePack data and rebind approved plugin paths in one pass.
fn check_value<'a>(
    value: &mut ValueRef<'a>,
    depth: usize,
    trusted_lib: Option<&'a str>,
) -> Fallible<bool> {
    if depth > MAX_NESTING {
        return Err(invalid("plan nesting limit exceeded"));
    }
    let mut rewritten = false;
    match value {
        ValueRef::Map(pairs) => {
            let is_variant = pairs.len() == 1;
            let mut seen = HashSet::new();
            for (key, child) in pairs {
                // Reject alternate enum-key encodings and ambiguous duplicate
                // keys before Polars sees the document.
                let Some(name) = key.as_str() else {
                    return Err(invalid("non-string MessagePack map field"));
                };
                if !seen.insert(name) {
                    return Err(invalid("duplicate MessagePack map field"));
                }
                if is_variant {
                    if name == "FfiPlugin" {
                        check_ffi_plugin(child)?;
                        if let Some(trusted_lib) = trusted_lib {
                            let ValueRef::Map(plugin) = child else {
                                return Err(invalid("invalid FfiPlugin"));
                            };
                            let path = plugin
                                .iter_mut()
                                .find(|(name, _)| name.as_str() == Some("lib"))
                                .map(|(_, value)| value)
                                .ok_or_else(|| invalid("missing plugin library"))?;
                            *path = ValueRef::from(trusted_lib);
                            rewritten = true;
                        }
                    }
                    if FORBIDDEN_EXPR_VARIANTS.contains(&name) {
                        return Err(err!(
                            FFI,
                            "Serialized Polars plan contains a forbidden construct: {}",
                            name
                        ));
                    }
                }
                rewritten |= check_value(child, depth + 1, trusted_lib)?;
            }
        }
        ValueRef::Array(values) => {
            for child in values {
                rewritten |= check_value(child, depth + 1, trusted_lib)?;
            }
        }
        ValueRef::Ext(_, _) => return Err(invalid("MessagePack extensions are unsupported")),
        _ => {}
    }
    Ok(rewritten)
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
        let mut payload =
            rmpv::decode::read_value(&mut std::io::Cursor::new(&serialized[HEADER_LENGTH..]))
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
        for name in ["PythonScan", "PipeWithSchema", "Scan", "Sink"] {
            let bad = root_with(Value::Array(vec![slot(name)]));
            assert!(
                validate_serialized_dslplan(&encoded(&bad)).is_err(),
                "{name}"
            );
        }
        for name in FORBIDDEN_EXPR_VARIANTS {
            let callback = Value::Map(vec![(Value::String((*name).into()), Value::Nil)]);
            let select = Value::Map(vec![(
                Value::String("Select".into()),
                Value::Map(vec![(
                    Value::String("expr".into()),
                    Value::Array(vec![callback]),
                )]),
            )]);
            let bad = root_with(Value::Array(vec![Value::Map(vec![
                (Value::String("value".into()), select),
                (Value::String("version".into()), Value::from(1)),
            ])]));
            let error = validate_serialized_dslplan(&encoded(&bad)).unwrap_err();
            assert!(
                format!("{error:?}").contains("forbidden construct"),
                "{name}"
            );
        }
        let bad = root_with(Value::Array(vec![Value::Map(vec![
            (
                Value::String("value".into()),
                Value::Map(vec![(
                    Value::String("GroupBy".into()),
                    Value::Map(vec![(Value::String("apply".into()), Value::Bool(true))]),
                )]),
            ),
            (Value::String("version".into()), Value::from(1)),
        ])]));
        let error = validate_serialized_dslplan(&encoded(&bad)).unwrap_err();
        assert!(format!("{error:?}").contains("group-by callbacks"));
    }

    fn serialized_scan(path: &str, scan_type: &str) -> Vec<u8> {
        let string = |value: &str| Value::String(value.into());
        let node = Value::Map(vec![(
            string("Scan"),
            Value::Map(vec![
                (
                    string("sources"),
                    Value::Map(vec![(
                        string("Paths"),
                        Value::Array(vec![Value::Map(vec![(string("Local"), string(path))])]),
                    )]),
                ),
                (
                    string("unified_scan_args"),
                    Value::Map(vec![
                        (string("cloud_options"), Value::Nil),
                        (string("deletion_files"), Value::Nil),
                    ]),
                ),
                (
                    string("scan_type"),
                    Value::Map(vec![(string(scan_type), Value::Map(vec![]))]),
                ),
            ]),
        )]);
        encoded(&root_with(Value::Array(vec![Value::Map(vec![
            (string("value"), node),
            (string("version"), Value::from(1)),
        ])])))
    }

    #[test]
    fn only_approved_parquet_scans_can_be_deserialized() {
        let path = "/public/keys.parquet".to_string();
        let bytes = serialized_scan(&path, "Parquet");
        assert!(validate_serialized_dslplan(&bytes).is_err());
        assert!(sanitize_serialized_dslplan(&bytes, "opendp", &[]).is_err());
        sanitize_serialized_dslplan(&bytes, "opendp", &[path.clone()]).unwrap();

        let other = vec!["/public/other.parquet".to_string()];
        assert!(sanitize_serialized_dslplan(&bytes, "opendp", &other).is_err());

        let alias = serialized_scan("/public/../public/keys.parquet", "Parquet");
        assert!(sanitize_serialized_dslplan(&alias, "opendp", &[path.clone()]).is_err());

        let csv = serialized_scan(&path, "Csv");
        assert!(sanitize_serialized_dslplan(&csv, "opendp", &[path]).is_err());
    }

    #[test]
    fn refuses_bad_header_duplicates_trailing_bytes_and_unknown_structure() {
        let mut bytes = Vec::new();
        DslPlan::default()
            .serialize_versioned(&mut bytes, Default::default())
            .unwrap();
        let mut bad_header = bytes.clone();
        bad_header[DSL_MAGIC.len()] ^= 1;
        let version_error = validate_serialized_dslplan(&bad_header).unwrap_err();
        let version_message = format!("{version_error:?}");
        assert!(version_message.contains("client"));
        assert!(version_message.contains("server"));
        assert!(version_message.contains("Polars versions"));

        let mut bad_schema = bytes.clone();
        bad_schema[DSL_MAGIC.len() + 4] ^= 1;
        let schema_error = validate_serialized_dslplan(&bad_schema).unwrap_err();
        assert!(format!("{schema_error:?}").contains("serialization schema"));

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
