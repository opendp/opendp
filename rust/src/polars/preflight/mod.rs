//! Data-only preflight for untrusted, serialized Polars DSL plans.
//!
//! The Polars DSL is a versioned header followed by MessagePack. Polars
//! deserializes some Python callbacks eagerly; it is therefore unsafe to call
//! `DslPlan::deserialize_versioned` and then inspect the resulting plan.
//!

use std::collections::HashSet;

use polars::prelude::DslPlan;
use rmpv::ValueRef;

use crate::error::Fallible;

const DSL_MAGIC: &[u8] = b"DSL_VERSION";
const HEADER_LENGTH: usize = DSL_MAGIC.len() + 4 + 64;
const MAX_NESTING: usize = 128;
// Reject executable callbacks and external I/O before Polars deserializes.
const FORBIDDEN_VARIANTS: &[&str] = &[
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
    "PythonScan",
    "PipeWithSchema",
    "Sink",
    "SinkMultiple",
    "Scan",
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

/// Validate a plan and replace approved plugin library paths with a trusted path.
///
/// File scans are rejected; public sources must be in memory.
pub fn sanitize_serialized_dslplan(
    raw: &[u8],
    trusted_lib: &str,
) -> Fallible<Vec<u8>> {
    if trusted_lib.is_empty() {
        return Err(invalid("server plugin library path is not configured"));
    }
    let (value, rewritten) = inspect_serialized_dslplan(raw, trusted_lib)?;
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
    trusted_lib: &'a str,
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
                let major =
                    u16::from_le_bytes([header[version_offset], header[version_offset + 1]]);
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

    let mut payload = &raw[HEADER_LENGTH..];
    let mut value = rmpv::decode::read_value_ref_with_max_depth(&mut payload, MAX_NESTING)
        .map_err(|e| err!(FFI, "Malformed Polars MessagePack: {}", e))?;
    if !payload.is_empty() {
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

    // Check every map and array, including unreachable slots and nested nodes.
    let rewritten = check_value(&mut value, 0, trusted_lib)?;
    Ok((value, rewritten))
}

/// Inspect inert MessagePack data and rebind approved plugin paths in one pass.
fn check_value<'a>(
    value: &mut ValueRef<'a>,
    depth: usize,
    trusted_lib: &'a str,
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
                    if FORBIDDEN_VARIANTS.contains(&name) {
                        return Err(err!(
                            FFI,
                            "Serialized Polars plan contains a forbidden construct: {}",
                            name
                        ));
                    }
                    if name == "GroupBy" {
                        let ValueRef::Map(fields) = child else {
                            return Err(invalid("invalid GroupBy plan node"));
                        };
                        if fields.iter().any(|(key, value)| {
                            key.as_str() == Some("apply") && !matches!(value, ValueRef::Nil)
                        }) {
                            return Err(invalid("group-by callbacks are unsupported"));
                        }
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
mod tests;
