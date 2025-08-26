use polars::prelude::*;
use polars_plan::dsl::Expr;
use polars_plan::plans::{DynListLiteralValue, DynLiteralValue, RangeLiteralValue};

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::metrics::MicrodataMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `replace(old, new)` expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The replace expression
pub fn make_expr_replace<M: OuterMetric>(
    input_domain: WildExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, M, ExprDomain, M>>
where
    M::InnerMetric: MicrodataMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Function {
        input,
        function: FunctionExpr::Replace,
    } = expr
    else {
        return fallible!(MakeTransformation, "expected replace expression");
    };

    let [input, old, new] = <[Expr; 3]>::try_from(input)
        .map_err(|_| err!(MakeTransformation, "replace takes an input, old and new"))?;

    let t_prior = input.make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let (Expr::Literal(old_lit), Expr::Literal(new_lit)) = (&old, &new) else {
        return fallible!(
            MakeTransformation,
            "replace: old and new must be literals, but found {:?} and {:?}",
            old,
            new
        );
    };

    let (old_len, new_len) = (literal_len(old_lit)?, literal_len(new_lit)?);
    if ![old_len, 1].contains(&new_len) {
        return fallible!(
            MakeTransformation,
            "length of `new` ({}) must match length of `old` ({}) or be broadcastable (1)",
            new_len,
            old_len
        );
    }

    let dtype = middle_domain.column.dtype();
    if matches!(dtype, DataType::Categorical(_, _)) {
        return fallible!(
            MakeTransformation,
            "replace cannot be applied to categorical data, because it may trigger a data-dependent CategoricalRemappingWarning in Polars"
        );
    }

    let old_lit = materialize_lit_list(old_lit)?;
    let new_lit = materialize_lit_list(new_lit)?;

    let (old_dtype, new_dtype) = (old_lit.get_datatype(), new_lit.get_datatype());
    if is_cast_fallible(&old_dtype, &dtype) || is_cast_fallible(&new_dtype, &dtype) {
        return fallible!(
            MakeTransformation,
            "replace: old datatype ({}) and new datatype ({}) must be consistent with the input datatype ({})",
            old_dtype,
            new_dtype,
            dtype
        );
    }

    let mut output_domain = middle_domain.clone();

    // reset domain descriptors
    output_domain.column.set_dtype(dtype)?;

    // if replacement can introduce nulls, then set nullable
    output_domain.column.nullable |= literal_is_nullable(&new_lit);

    // if old has null and new does not, then there is a non-null null replacement
    if literal_is_nullable(&old_lit) && !literal_is_nullable(&new_lit) {
        output_domain.column.nullable = false;
    }

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            middle_metric.clone(),
            output_domain,
            middle_metric,
            Function::then_expr(move |expr| Expr::Function {
                input: vec![expr.clone(), old.clone(), new.clone()],
                function: FunctionExpr::Replace,
            }),
            StabilityMap::new(Clone::clone),
        )?
}

/// # Proof Definition
/// Returns the length of a literal value.
pub(crate) fn literal_len(literal: &LiteralValue) -> Fallible<i128> {
    Ok(match literal {
        LiteralValue::Range(RangeLiteralValue { low, high, .. }) => high.saturating_sub(*low),
        LiteralValue::Series(s) => s.len() as i128,
        LiteralValue::Dyn(DynLiteralValue::List(l)) => match l {
            DynListLiteralValue::Str(pl_small_strs) => pl_small_strs.len() as i128,
            DynListLiteralValue::Int(items) => items.len() as i128,
            DynListLiteralValue::Float(items) => items.len() as i128,
            DynListLiteralValue::List(lists) => lists.len() as i128,
        },
        l if l.is_scalar() => 1,
        l => {
            return fallible!(
                MakeTransformation,
                "unrecognized literal when determining length: {l:?}"
            );
        }
    })
}

fn to_series<T: PolarsPhysicalType, N>(x: Box<[Option<N>]>) -> Series
where
    ChunkedArray<T>: NewChunkedArray<T, N>,
{
    ChunkedArray::<T>::from_iter_options("literal".into(), x.into_iter()).into_series()
}

pub(crate) fn materialize_lit_list(literal: &LiteralValue) -> Fallible<LiteralValue> {
    Ok(match literal {
        // desugar lists
        LiteralValue::Dyn(DynLiteralValue::List(list)) => {
            LiteralValue::Series(SpecialEq::new(match list.clone() {
                DynListLiteralValue::Int(list) => to_series::<Int128Type, _>(list),
                DynListLiteralValue::Float(list) => to_series::<Float64Type, _>(list),
                DynListLiteralValue::Str(list) => to_series::<StringType, _>(list),
                DynListLiteralValue::List(_) => {
                    return fallible!(MakeTransformation, "nested lists are not supported");
                }
            }))
        }
        // desugar mappings
        LiteralValue::Scalar(scalar) => {
            if let AnyValue::List(v) = scalar.value() {
                LiteralValue::Series(SpecialEq::new(v.clone()))
            } else {
                literal.clone()
            }
        }
        literal => literal.clone(),
    })
}

/// # Proof Definition
/// Returns whether a literal value contains null.
pub(crate) fn literal_is_nullable(literal: &LiteralValue) -> bool {
    match literal {
        LiteralValue::Series(new_series) => new_series.has_nulls(),
        LiteralValue::Scalar(s) => s.is_null(),
        _ => false,
    }
}

/// # Proof Definition
/// Returns whether casting is fallible between two data types.
pub(crate) fn is_cast_fallible(from: &DataType, to: &DataType) -> bool {
    if matches!(from, DataType::Null)
        || matches!(to, DataType::String | DataType::Categorical(_, _))
    {
        return false;
    }

    if let DataType::Unknown(child) = from {
        return match child {
            UnknownKind::Ufunc => true,
            UnknownKind::Int(v) => {
                return if let Ok(v) = i64::try_from(*v) {
                    AnyValue::Int64(v).cast(&to).is_null()
                } else {
                    to != &DataType::UInt64
                };
            }
            UnknownKind::Float => !to.is_float(),
            UnknownKind::Str => !to.is_string(),
            UnknownKind::Any => true,
        };
    }
    from != to
}
