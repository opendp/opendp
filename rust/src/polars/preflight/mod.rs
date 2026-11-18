//! Data-only preflight for untrusted, serialized Polars DSL plans.
//!
//! The Polars DSL is a versioned header followed by MessagePack. Polars
//! deserializes some Python callbacks eagerly; it is therefore unsafe to call
//! `DslPlan::deserialize_versioned` and then inspect the resulting plan.
//!

use polars::prelude::DslPlan;
use rmpv::ValueRef;

use crate::error::Fallible;

mod subset;

const DSL_MAGIC: &[u8] = b"DSL_VERSION";
const HEADER_LENGTH: usize = DSL_MAGIC.len() + 4 + 64;
// SECURITY: Audit supported operations when the pinned Polars version changes.

fn invalid(message: &str) -> crate::error::Error {
    err!(
        FFI,
        "Unsafe or unsupported serialized Polars plan: {}",
        message
    )
}

fn value_str<'a>(value: &'a ValueRef<'_>) -> Option<&'a str> {
    match value {
        ValueRef::String(text) => text.as_str(),
        _ => None,
    }
}

fn is_opendp_symbol(symbol: &str) -> bool {
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
            | "noisy_max"
            | "permute_plugin"
    )
}

fn is_opendp_library(lib: &str) -> bool {
    std::path::Path::new(lib)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("libopendp.") || name.starts_with("opendp."))
}

fn plugin_field<'a, 'v>(
    fields: &'v [(ValueRef<'a>, ValueRef<'a>)],
    name: &str,
) -> Option<&'v ValueRef<'a>> {
    fields
        .iter()
        .find(|(key, _)| value_str(key) == Some(name))
        .map(|(_, value)| value)
}

fn rebind_plugin<'a>(
    plugin: &mut ValueRef<'a>,
    trusted_lib: &'a str,
    trusted: bool,
) -> Fallible<bool> {
    let ValueRef::Map(fields) = plugin else {
        return if trusted {
            Ok(false)
        } else {
            Err(invalid("invalid FfiPlugin"))
        };
    };
    let lib = plugin_field(fields, "lib").and_then(value_str);
    let symbol = plugin_field(fields, "symbol").and_then(value_str);

    if !trusted {
        if fields.len() != 4 || lib.is_none() {
            return Err(invalid("invalid FfiPlugin fields"));
        }
        let symbol = symbol.ok_or_else(|| invalid("missing plugin symbol"))?;
        if !is_opendp_symbol(symbol) {
            return Err(invalid("unsupported native plugin"));
        }
        match plugin_field(fields, "kwargs") {
            Some(ValueRef::Binary(bytes)) if bytes.is_empty() => {}
            Some(ValueRef::Array(values)) if values.is_empty() => {}
            _ => return Err(invalid("plugin keyword arguments are forbidden")),
        }
    } else if !lib.is_some_and(is_opendp_library) || !symbol.is_some_and(is_opendp_symbol) {
        return Ok(false);
    }

    let path = fields
        .iter_mut()
        .find(|(key, _)| value_str(key) == Some("lib"))
        .map(|(_, value)| value)
        .ok_or_else(|| invalid("missing plugin library"))?;
    *path = ValueRef::from(trusted_lib);
    Ok(true)
}

// Trusted plans have no operation restrictions, but still need OpenDP plugins
// rebound. Untrusted plans use the restricted traversal in subset.rs instead.
fn rebind_trusted<'a>(value: &mut ValueRef<'a>, trusted_lib: &'a str) -> Fallible<bool> {
    match value {
        ValueRef::Map(fields) => {
            if fields.len() == 1 && value_str(&fields[0].0) == Some("FfiPlugin") {
                return rebind_plugin(&mut fields[0].1, trusted_lib, true);
            }
            let mut rewritten = false;
            for (_, child) in fields {
                rewritten |= rebind_trusted(child, trusted_lib)?;
            }
            Ok(rewritten)
        }
        ValueRef::Array(values) => {
            let mut rewritten = false;
            for child in values {
                rewritten |= rebind_trusted(child, trusted_lib)?;
            }
            Ok(rewritten)
        }
        _ => Ok(false),
    }
}

/// Rebind OpenDP plugins, rejecting unsafe constructs unless the plan is trusted.
pub fn sanitize_serialized_dslplan(
    raw: &[u8],
    trusted_lib: &str,
    trusted: bool,
) -> Fallible<Vec<u8>> {
    if trusted_lib.is_empty() {
        return Err(invalid("server plugin library path is not configured"));
    }
    let (value, rewritten) = inspect_serialized_dslplan(raw, trusted_lib, trusted)?;
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
    trusted: bool,
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
    let mut value = rmpv::decode::read_value_ref(&mut payload)
        .map_err(|e| err!(FFI, "Malformed Polars MessagePack: {}", e))?;
    if !payload.is_empty() {
        return Err(invalid("trailing data"));
    }

    let rewritten = if trusted {
        rebind_trusted(&mut value, trusted_lib)?
    } else {
        subset::check_operations(&mut value, trusted_lib)?
    };
    Ok((value, rewritten))
}

#[cfg(test)]
mod tests;
