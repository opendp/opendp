use std::sync::Arc;

use polars::chunked_array::cast::CastOptions;
use polars::prelude::*;
use polars_plan::dsl::Expr;

use super::{StableExpr, cast::is_integer_widening_cast};
use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{L01InfDistance, LpDistance};
use crate::transformations::traits::UnboundedMetric;

#[cfg(test)]
mod test;

/// Make a Transformation that casts an aggregation output to int 64.
/// Casting aggregations to i64 before noise is added can enable negative values.
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
/// * `expr` - The input measurement to be cast
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_cast_aggregation<MI, const P: usize>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, L01InfDistance<MI>, ExprDomain, LpDistance<P, f64>>>
where
    MI: 'static + UnboundedMetric,
    (WildExprDomain, L01InfDistance<MI>): MetricSpace,
    (ExprDomain, LpDistance<P, f64>): MetricSpace,
    Expr: StableExpr<L01InfDistance<MI>, LpDistance<P, f64>>,
{
    let Expr::Cast {
        expr: input,
        dtype: to_type,
        mut options,
    } = expr
    else {
        return fallible!(MakeTransformation, "expected cast expression");
    };

    let to_type_dtype = to_type
        .as_literal()
        .ok_or_else(|| {
            err!(
                MakeTransformation,
                "make_cast_aggregation only supports literal dtype"
            )
        })?
        .clone();

    if matches!(options, CastOptions::Strict) {
        options = CastOptions::NonStrict;
    }

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;

    let (middle_domain, middle_metric) = t_prior.output_space();

    let from_dtype = middle_domain.column.dtype();
    if !is_integer_widening_cast(&from_dtype, &to_type_dtype) {
        return fallible!(
            MakeTransformation,
            "cast from {} to {} cannot preserve all possible values",
            from_dtype,
            to_type_dtype
        );
    }

    let mut output_domain = middle_domain.clone();
    output_domain.column.set_dtype(to_type_dtype.clone())?;

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            middle_metric.clone(),
            output_domain,
            middle_metric,
            Function::then_expr(move |expr| Expr::Cast {
                expr: Arc::new(expr),
                dtype: to_type.clone(),
                options,
            }),
            StabilityMap::new(Clone::clone),
        )?
}
