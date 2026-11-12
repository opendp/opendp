//! Executable scalar operators and their generated specification counterparts.
use super::*;

/// Common executable and specification rules for scalar expression typing.
#[verus_verify(dual_spec, open)]
#[verus_spec(returns binary_field(op, left, right))]
pub fn binary_field(
    op: BinaryOp, left: Option<Field>, right: Option<Field>,
) -> Option<Field> {
    match (left, right) {
        (Some(l), Some(r)) => {
            let valid = match (op, l.dtype, r.dtype) {
                (BinaryOp::Lt | BinaryOp::Gt, DType::Int64, DType::Int64) => true,
                (BinaryOp::Eq, DType::Int64, DType::Int64)
                    | (BinaryOp::Eq, DType::Boolean, DType::Boolean) => true,
                (BinaryOp::And, DType::Boolean, DType::Boolean) => true,
                _ => false,
            };
            if valid {
                Some(Field { dtype: DType::Boolean, nullable: l.nullable || r.nullable })
            } else { None }
        },
        _ => None,
    }
}

#[verus_verify(dual_spec, open)]
#[verus_spec(returns clip_field(value, lower, upper))]
pub fn clip_field(
    value: Option<Field>, lower: Option<Option<Field>>, upper: Option<Option<Field>>,
) -> Option<Field> {
    match value {
        Some(field) => {
            match field.dtype {
                DType::Boolean => None,
                DType::Int64 => {
                    let lower_ok = match lower {
                        None | Some(Some(Field { dtype: DType::Int64, .. })) => true,
                        _ => false,
                    };
                    let upper_ok = match upper {
                        None | Some(Some(Field { dtype: DType::Int64, .. })) => true,
                        _ => false,
                    };
                    if lower_ok && upper_ok { Some(field) } else { None }
                },
            }
        },
        _ => None,
    }
}

/// Single source of truth for scalar semantics: dual_spec generates the
/// open specification used by the row interpreter and typing proofs.
#[verus_verify(dual_spec, open)]
#[verus_spec(returns eval_binary(op, lhs, rhs))]
pub fn eval_binary(
    op: BinaryOp, lhs: Option<Value>, rhs: Option<Value>,
) -> Result<Option<Value>, ()> {
    match op {
        BinaryOp::Lt => match (lhs, rhs) {
            (Some(Value::Int(a)), Some(Value::Int(b))) =>
                Ok(Some(Value::Bool(a < b))),
            (None, _) | (_, None) => Ok(None),
            _ => Err(()),
        },
        BinaryOp::Gt => match (lhs, rhs) {
            (Some(Value::Int(a)), Some(Value::Int(b))) =>
                Ok(Some(Value::Bool(a > b))),
            (None, _) | (_, None) => Ok(None),
            _ => Err(()),
        },
        BinaryOp::Eq => match (lhs, rhs) {
            (Some(Value::Int(a)), Some(Value::Int(b))) =>
                Ok(Some(Value::Bool(a == b))),
            (Some(Value::Bool(a)), Some(Value::Bool(b))) =>
                Ok(Some(Value::Bool(a == b))),
            (None, _) | (_, None) => Ok(None),
            _ => Err(()),
        },
        BinaryOp::And => match (lhs, rhs) {
            (Some(Value::Bool(a)), Some(Value::Bool(b))) =>
                Ok(Some(Value::Bool(a && b))),
            (Some(Value::Bool(false)), None) | (None, Some(Value::Bool(false))) =>
                Ok(Some(Value::Bool(false))),
            (Some(Value::Bool(true)), None)
                | (None, Some(Value::Bool(true)))
                | (None, None) => Ok(None),
            _ => Err(()),
        },
    }
}

/// The outer Option indicates a missing bound. The inner Option is the
/// evaluated bound, which can be null. Null bounds are ignored in Polars.
#[verus_verify(dual_spec, open)]
#[verus_spec(returns eval_clip(value, lower, upper))]
pub fn eval_clip(
    value: Option<Value>,
    lower: Option<Option<Value>>,
    upper: Option<Option<Value>>,
) -> Result<Option<Value>, ()> {
    match (value, lower, upper) {
        (None, _, _) => Ok(None),
        (Some(Value::Int(_)), Some(Some(Value::Bool(_))), _)
            | (Some(Value::Int(_)), _, Some(Some(Value::Bool(_)))) => Err(()),
        (Some(Value::Int(v)), lower, upper) => {
            let low = match lower {
                Some(Some(Value::Int(n))) => Some(n),
                _ => None,
            };
            let high = match upper {
                Some(Some(Value::Int(n))) => Some(n),
                _ => None,
            };
            // Use the original input for both comparisons. Polars tests
            // establish that the lower bound takes precedence if both apply.
            let clipped = match low {
                Some(n) if v < n => n,
                _ => match high {
                    Some(n) if v > n => n,
                    _ => v,
                },
            };
            Ok(Some(Value::Int(clipped)))
        },
        _ => Err(()),
    }
}
