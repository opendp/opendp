//! Executable scalar operators and their generated specification counterparts.
use super::*;

/// Common executable and specification rules for scalar expression typing.
#[verus_verify(dual_spec, open)]
#[verus_spec(returns binary_field(op, left, right))]
pub fn binary_field(
    op: VBinaryOp, left: Option<VField>, right: Option<VField>,
) -> Option<VField> {
    match (left, right) {
        (Some(l), Some(r)) => {
            let valid = match (op, l.dtype, r.dtype) {
                (VBinaryOp::Lt | VBinaryOp::Gt, VDType::Int64, VDType::Int64) => true,
                (VBinaryOp::Eq, VDType::Int64, VDType::Int64)
                    | (VBinaryOp::Eq, VDType::Boolean, VDType::Boolean) => true,
                (VBinaryOp::And, VDType::Boolean, VDType::Boolean) => true,
                _ => false,
            };
            if valid {
                Some(VField { dtype: VDType::Boolean, nullable: l.nullable || r.nullable })
            } else { None }
        },
        _ => None,
    }
}

#[verus_verify(dual_spec, open)]
#[verus_spec(returns clip_field(value, lower, upper))]
pub fn clip_field(
    value: Option<VField>, lower: Option<Option<VField>>, upper: Option<Option<VField>>,
) -> Option<VField> {
    match value {
        Some(field) => {
            match field.dtype {
                VDType::Boolean => None,
                VDType::Int64 => {
                    let lower_ok = match lower {
                        None | Some(Some(VField { dtype: VDType::Int64, .. })) => true,
                        _ => false,
                    };
                    let upper_ok = match upper {
                        None | Some(Some(VField { dtype: VDType::Int64, .. })) => true,
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
    op: VBinaryOp, lhs: Option<VValue>, rhs: Option<VValue>,
) -> Result<Option<VValue>, ()> {
    match op {
        VBinaryOp::Lt => match (lhs, rhs) {
            (Some(VValue::Int(a)), Some(VValue::Int(b))) =>
                Ok(Some(VValue::Bool(a < b))),
            (None, _) | (_, None) => Ok(None),
            _ => Err(()),
        },
        VBinaryOp::Gt => match (lhs, rhs) {
            (Some(VValue::Int(a)), Some(VValue::Int(b))) =>
                Ok(Some(VValue::Bool(a > b))),
            (None, _) | (_, None) => Ok(None),
            _ => Err(()),
        },
        VBinaryOp::Eq => match (lhs, rhs) {
            (Some(VValue::Int(a)), Some(VValue::Int(b))) =>
                Ok(Some(VValue::Bool(a == b))),
            (Some(VValue::Bool(a)), Some(VValue::Bool(b))) =>
                Ok(Some(VValue::Bool(a == b))),
            (None, _) | (_, None) => Ok(None),
            _ => Err(()),
        },
        VBinaryOp::And => match (lhs, rhs) {
            (Some(VValue::Bool(a)), Some(VValue::Bool(b))) =>
                Ok(Some(VValue::Bool(a && b))),
            (Some(VValue::Bool(false)), None) | (None, Some(VValue::Bool(false))) =>
                Ok(Some(VValue::Bool(false))),
            (Some(VValue::Bool(true)), None)
                | (None, Some(VValue::Bool(true)))
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
    value: Option<VValue>,
    lower: Option<Option<VValue>>,
    upper: Option<Option<VValue>>,
) -> Result<Option<VValue>, ()> {
    match (value, lower, upper) {
        (None, _, _) => Ok(None),
        (Some(VValue::Int(_)), Some(Some(VValue::Bool(_))), _)
            | (Some(VValue::Int(_)), _, Some(Some(VValue::Bool(_)))) => Err(()),
        (Some(VValue::Int(v)), lower, upper) => {
            let low = match lower {
                Some(Some(VValue::Int(n))) => Some(n),
                _ => None,
            };
            let high = match upper {
                Some(Some(VValue::Int(n))) => Some(n),
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
            Ok(Some(VValue::Int(clipped)))
        },
        _ => Err(()),
    }
}
