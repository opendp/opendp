//! Restricted operation vocabulary for untrusted serialized Polars plans.
//!
//! Inspect only enum tags and the fields containing nested expressions.
//! Polars still deserializes the DSL, and OpenDP constructors validate and
//! reconstruct the semantics and options of each operation.

use std::collections::HashSet;

use rmpv::ValueRef;

use crate::error::Fallible;

use super::{invalid, rebind_plugin, value_str};

#[derive(Clone, Copy)]
enum Role {
    Inert,
    Arena,
    Slot,
    Plan,
    Expr,
    Exprs,
    Function,
    Agg,
    Boolean,
    Range,
    Dtype,
    DtypePairs,
    FirstExpr,
    OrderBy,
}

fn variant<'a, 'v>(
    value: &'v mut ValueRef<'a>,
) -> Fallible<(String, Option<&'v mut ValueRef<'a>>)> {
    match value {
        ValueRef::String(name) => Ok((
            name.as_str()
                .ok_or_else(|| invalid("invalid operation tag"))?
                .to_owned(),
            None,
        )),
        ValueRef::Map(fields) if fields.len() == 1 => {
            let (name, body) = &mut fields[0];
            Ok((
                value_str(name)
                    .ok_or_else(|| invalid("invalid operation tag"))?
                    .to_owned(),
                Some(body),
            ))
        }
        _ => Err(invalid("expected a tagged Polars operation")),
    }
}

fn payload<'a, 'v>(value: Option<&'v mut ValueRef<'a>>) -> Fallible<&'v mut ValueRef<'a>> {
    value.ok_or_else(|| invalid("missing operation payload"))
}

fn map_fields<'a>(
    value: &mut ValueRef<'a>,
    selected: &[(&str, Role)],
    trusted_lib: &'a str,
) -> Fallible<bool> {
    let ValueRef::Map(fields) = value else {
        return Err(invalid("expected operation fields"));
    };
    for &(name, _) in selected {
        if !fields.iter().any(|(key, _)| value_str(key) == Some(name)) {
            return Err(invalid(&format!("missing operation field: {name}")));
        }
    }

    let mut seen = HashSet::new();
    let mut rewritten = false;
    for (key, child) in fields {
        let name = value_str(key).ok_or_else(|| invalid("non-string MessagePack map field"))?;
        if !seen.insert(name.to_owned()) {
            return Err(invalid("duplicate MessagePack map field"));
        }
        let role = selected
            .iter()
            .find(|(field, _)| *field == name)
            .map_or(Role::Inert, |(_, role)| *role);
        rewritten |= visit(child, role, trusted_lib)?;
    }
    Ok(rewritten)
}

fn array<'a>(value: &mut ValueRef<'a>, role: Role, trusted_lib: &'a str) -> Fallible<bool> {
    let ValueRef::Array(values) = value else {
        return Err(invalid("expected an array"));
    };
    let mut rewritten = false;
    for value in values {
        rewritten |= visit(value, role, trusted_lib)?;
    }
    Ok(rewritten)
}

fn tuple<'a>(
    value: &mut ValueRef<'a>,
    index: usize,
    role: Role,
    trusted_lib: &'a str,
) -> Fallible<bool> {
    let ValueRef::Array(values) = value else {
        return Err(invalid("expected operation arguments"));
    };
    if values.len() <= index {
        return Err(invalid("missing operation argument"));
    }
    let mut rewritten = false;
    for (i, child) in values.iter_mut().enumerate() {
        rewritten |= visit(
            child,
            if i == index { role } else { Role::Inert },
            trusted_lib,
        )?;
    }
    Ok(rewritten)
}

fn inert<'a>(value: &mut ValueRef<'a>, trusted_lib: &'a str) -> Fallible<bool> {
    match value {
        ValueRef::Map(_) => map_fields(value, &[], trusted_lib),
        ValueRef::Array(_) => array(value, Role::Inert, trusted_lib),
        ValueRef::Ext(_, _) => Err(invalid("MessagePack extensions are unsupported")),
        _ => Ok(false),
    }
}

fn optional<'a>(body: Option<&mut ValueRef<'a>>, trusted_lib: &'a str) -> Fallible<bool> {
    match body {
        Some(body) => inert(body, trusted_lib),
        None => Ok(false),
    }
}

fn plan<'a>(value: &mut ValueRef<'a>, trusted_lib: &'a str) -> Fallible<bool> {
    let (name, body) = variant(value)?;
    match name.as_str() {
        "DataFrameScan" | "IR" => optional(body, trusted_lib),
        "Select" => map_fields(payload(body)?, &[("expr", Role::Exprs)], trusted_lib),
        "HStack" => map_fields(payload(body)?, &[("exprs", Role::Exprs)], trusted_lib),
        "Filter" => map_fields(payload(body)?, &[("predicate", Role::Expr)], trusted_lib),
        "GroupBy" => {
            let body = payload(body)?;
            let ValueRef::Map(fields) = &*body else {
                return Err(invalid("invalid GroupBy plan node"));
            };
            if fields.iter().any(|(key, value)| {
                value_str(key) == Some("apply") && !matches!(value, ValueRef::Nil)
            }) {
                return Err(invalid("group-by callbacks are unsupported"));
            }
            map_fields(
                body,
                &[("keys", Role::Exprs), ("aggs", Role::Exprs)],
                trusted_lib,
            )
        }
        "Join" => map_fields(
            payload(body)?,
            &[
                ("left_on", Role::Exprs),
                ("right_on", Role::Exprs),
                ("predicates", Role::Exprs),
            ],
            trusted_lib,
        ),
        _ => Err(invalid(&format!("unsupported plan operation: {name}"))),
    }
}

fn agg<'a>(value: &mut ValueRef<'a>, trusted_lib: &'a str) -> Fallible<bool> {
    let (name, body) = variant(value)?;
    let body = payload(body)?;
    match name.as_str() {
        "Count" | "Min" | "Max" => map_fields(body, &[("input", Role::Expr)], trusted_lib),
        "Quantile" => map_fields(
            body,
            &[("expr", Role::Expr), ("quantile", Role::Expr)],
            trusted_lib,
        ),
        "Std" | "Var" => tuple(body, 0, Role::Expr, trusted_lib),
        "NUnique" | "Sum" | "Mean" | "Median" | "First" | "Last" => {
            visit(body, Role::Expr, trusted_lib)
        }
        _ => Err(invalid(&format!("unsupported aggregation: {name}"))),
    }
}

fn function<'a>(value: &mut ValueRef<'a>, trusted_lib: &'a str) -> Fallible<bool> {
    let (name, body) = variant(value)?;
    match name.as_str() {
        "FfiPlugin" => {
            let body = payload(body)?;
            let rewritten = rebind_plugin(body, trusted_lib, false)?;
            Ok(rewritten | inert(body, trusted_lib)?)
        }
        "Boolean" => visit(payload(body)?, Role::Boolean, trusted_lib),
        "Range" => visit(payload(body)?, Role::Range, trusted_lib),
        "ReplaceStrict" => map_fields(
            payload(body)?,
            &[("return_dtype", Role::Dtype)],
            trusted_lib,
        ),
        "FillNull" | "Clip" | "Cut" | "DropNulls" | "DropNans" | "ToPhysical" | "Replace"
        | "NullCount" | "AsStruct" | "Hash" | "Rank" | "Reverse" | "Random" | "Abs" | "Negate"
        | "Floor" | "Ceil" | "Round" | "Pow" => optional(body, trusted_lib),
        _ => Err(invalid(&format!("unsupported function operation: {name}"))),
    }
}

fn boolean<'a>(value: &mut ValueRef<'a>, trusted_lib: &'a str) -> Fallible<bool> {
    let (name, body) = variant(value)?;
    if matches!(
        name.as_str(),
        "Any"
            | "All"
            | "IsNull"
            | "IsNotNull"
            | "IsNan"
            | "IsNotNan"
            | "IsFinite"
            | "IsInfinite"
            | "IsBetween"
            | "IsIn"
            | "AllHorizontal"
            | "AnyHorizontal"
            | "Not"
    ) {
        optional(body, trusted_lib)
    } else {
        Err(invalid(&format!("unsupported Boolean operation: {name}")))
    }
}

fn range<'a>(value: &mut ValueRef<'a>, trusted_lib: &'a str) -> Fallible<bool> {
    let (name, body) = variant(value)?;
    match name.as_str() {
        "IntRange" | "IntRanges" => {
            map_fields(payload(body)?, &[("dtype", Role::Dtype)], trusted_lib)
        }
        _ => Err(invalid(&format!("unsupported range operation: {name}"))),
    }
}

// Dtype expressions may embed arbitrary Exprs through OfExpr. The embedded
// expression must pass the same allowlist before Polars deserializes it.
fn dtype<'a>(value: &mut ValueRef<'a>, trusted_lib: &'a str) -> Fallible<bool> {
    if matches!(value, ValueRef::Nil) {
        return Ok(false);
    }
    let (name, body) = variant(value)?;
    match name.as_str() {
        "Literal" | "SelfDtype" => optional(body, trusted_lib),
        "OfExpr" => visit(payload(body)?, Role::Expr, trusted_lib),
        "InnerDataType" => map_fields(payload(body)?, &[("input", Role::Dtype)], trusted_lib),
        "Int" | "Struct" | "WrapInArray" => tuple(payload(body)?, 0, Role::Dtype, trusted_lib),
        "WrapInList" => visit(payload(body)?, Role::Dtype, trusted_lib),
        "StructWithFields" => array(payload(body)?, Role::DtypePairs, trusted_lib),
        _ => Err(invalid(&format!("unsupported dtype operation: {name}"))),
    }
}

fn expr<'a>(value: &mut ValueRef<'a>, trusted_lib: &'a str) -> Fallible<bool> {
    let (name, body) = variant(value)?;
    match name.as_str() {
        "Column" | "Literal" | "Element" | "Len" => optional(body, trusted_lib),
        "Alias" => tuple(payload(body)?, 0, Role::Expr, trusted_lib),
        "KeepName" => visit(payload(body)?, Role::Expr, trusted_lib),
        "BinaryExpr" => map_fields(
            payload(body)?,
            &[("left", Role::Expr), ("right", Role::Expr)],
            trusted_lib,
        ),
        "Cast" => map_fields(
            payload(body)?,
            &[("expr", Role::Expr), ("dtype", Role::Dtype)],
            trusted_lib,
        ),
        "Ternary" => map_fields(
            payload(body)?,
            &[
                ("predicate", Role::Expr),
                ("truthy", Role::Expr),
                ("falsy", Role::Expr),
            ],
            trusted_lib,
        ),
        "Filter" => map_fields(
            payload(body)?,
            &[("input", Role::Expr), ("by", Role::Expr)],
            trusted_lib,
        ),
        "Function" => map_fields(
            payload(body)?,
            &[("input", Role::Exprs), ("function", Role::Function)],
            trusted_lib,
        ),
        "Agg" => visit(payload(body)?, Role::Agg, trusted_lib),
        "SortBy" => map_fields(
            payload(body)?,
            &[("expr", Role::Expr), ("by", Role::Exprs)],
            trusted_lib,
        ),
        "Over" => map_fields(
            payload(body)?,
            &[
                ("function", Role::Expr),
                ("partition_by", Role::Exprs),
                ("order_by", Role::OrderBy),
            ],
            trusted_lib,
        ),
        _ => Err(invalid(&format!(
            "unsupported expression operation: {name}"
        ))),
    }
}

fn visit<'a>(value: &mut ValueRef<'a>, role: Role, trusted_lib: &'a str) -> Fallible<bool> {
    match role {
        Role::Inert => inert(value, trusted_lib),
        Role::Arena => array(value, Role::Slot, trusted_lib),
        Role::Slot => {
            if matches!(value, ValueRef::Nil) {
                Ok(false)
            } else {
                map_fields(value, &[("value", Role::Plan)], trusted_lib)
            }
        }
        Role::Plan => {
            if matches!(value, ValueRef::Nil) {
                Ok(false)
            } else {
                plan(value, trusted_lib)
            }
        }
        Role::Expr => expr(value, trusted_lib),
        Role::Exprs => array(value, Role::Expr, trusted_lib),
        Role::Function => function(value, trusted_lib),
        Role::Agg => agg(value, trusted_lib),
        Role::Boolean => boolean(value, trusted_lib),
        Role::Range => range(value, trusted_lib),
        Role::Dtype => dtype(value, trusted_lib),
        Role::DtypePairs => tuple(value, 1, Role::Dtype, trusted_lib),
        Role::FirstExpr => tuple(value, 0, Role::Expr, trusted_lib),
        Role::OrderBy => {
            if matches!(value, ValueRef::Nil) {
                Ok(false)
            } else {
                visit(value, Role::FirstExpr, trusted_lib)
            }
        }
    }
}

/// Inspect each serialized node exactly once, including unreachable nodes,
/// without duplicating privacy validation performed by the constructors.
pub(super) fn check_operations<'a>(
    value: &mut ValueRef<'a>,
    trusted_lib: &'a str,
) -> Fallible<bool> {
    let ValueRef::Map(root) = &*value else {
        return Err(invalid("expected a serialized plan map"));
    };
    let keys: HashSet<_> = root.iter().filter_map(|(key, _)| value_str(key)).collect();
    if root.len() != 3 || keys != HashSet::from(["root", "dataframes", "dsl_plans"]) {
        return Err(invalid("unrecognized serialized plan structure"));
    }
    map_fields(value, &[("dsl_plans", Role::Arena)], trusted_lib)
}
