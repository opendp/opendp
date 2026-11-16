use polars::chunked_array::cast::CastOptions;
use polars::prelude::*;
use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::metrics::MicrodataMetric;

use super::{
    StableExpr,
    cast::{has_unsafe_temporal_cast, is_null_preserving_cast, is_safe_categorical_cast},
};

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `cast(dtype)` expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The cast expression
pub fn make_expr_cast<M: OuterMetric>(
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
    let Expr::Cast {
        expr: input,
        dtype: to_type,
        mut options,
    } = expr
    else {
        return fallible!(MakeTransformation, "expected cast expression");
    };

    // Strict casting makes the transformation unstable: errors tell you things about private data.
    // Could throw an error if strict, but it is the default, so for ease-of-use it is forced to be non-strict.
    // It is also ok for overflow to wraparound.
    if matches!(options, CastOptions::Strict) {
        options = CastOptions::NonStrict;
    }

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let data_column = &mut output_domain.column;

    let to_type_dtype = to_type
        .as_literal()
        .ok_or_else(|| {
            err!(
                MakeTransformation,
                "cast expression only supports literal dtype"
            )
        })?
        .clone();

    let from_dtype = data_column.dtype();
    // Array casts require separate reasoning about element nullability and
    // categorical encodings. Do not support them, including identity casts.
    if matches!(&from_dtype, DataType::Array(_, _))
        || matches!(&to_type_dtype, DataType::Array(_, _))
    {
        return fallible!(MakeTransformation, "Array casts are not supported");
    }
    // Mutable categorical mappings can make cast outputs depend on other rows,
    // expose physical category codes, or raise when the mapping fills up.
    if !is_safe_categorical_cast(&from_dtype, &to_type_dtype) {
        return fallible!(
            MakeTransformation,
            "Casting to or from mutable Categorical encodings is unsafe. Cast to String or fixed Enum instead."
        );
    }
    // Polars' non-strict timestamp parser still panics on dates outside the
    // nanosecond timestamp range. Numeric and broad-range datetime-to-Date
    // casts can also construct dates that Chrono cannot represent.
    if has_unsafe_temporal_cast(&from_dtype, &to_type_dtype) {
        return fallible!(
            MakeTransformation,
            "This temporal cast can produce data-dependent failures on private values."
        );
    }

    if from_dtype != to_type_dtype {
        data_column.set_dtype(to_type_dtype.clone())?;
        // Non-strict casts may introduce nulls, even from non-null inputs.
        data_column.nullable |= !is_null_preserving_cast(&from_dtype, &to_type_dtype);
    }

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            middle_metric.clone(),
            output_domain,
            middle_metric,
            Function::then_expr(move |expr| Expr::Cast {
                expr: Arc::new(expr),
                dtype: to_type.clone(),
                // Specify behavior for when casting fails (this is forced to be non-strict).
                options,
            }),
            StabilityMap::new(Clone::clone),
        )?
}

