use polars::chunked_array::cast::CastOptions;
use polars::prelude::*;
use polars_plan::dsl::Expr;

use super::StableExpr;
use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, VectorDomain, WildExprDomain};
use crate::error::*;
use crate::transformations::make_stable_expr::{L01InfDistance, LpDistance, UnboundedMetric};

#[cfg(test)]
mod test;

/// Make a Transformation that casts a measurement output to int 64.
/// Casting measurements to i64 before noise is added can enable negative values.
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
/// * `expr` - The input measurement to be cast
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_cast_measurement_to_i64<MI, const P: usize, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: MI,
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
                "make_cast_measurement_to_i64 only supports literal dtype"
            )
        })?
        .clone();

    if to_type_dtype != DataType::Int64 {
        return fallible!(
            MakeTransformation,
            "make_cast_measurement_to_i64 cast expects target dtype Int64, found {}",
            to_type_dtype
        );
    }

    if matches!(options, CastOptions::Strict) {
        options = CastOptions::NonStrict;
    }

    // This recursively stabilizes len/count/n_unique/count_null.
    // The child emits ExprDomain under LpDistance<P, f64>.
    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;

    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let active_series = &mut output_domain.column;

    active_series.set_element_domain(AtomDomain::<i64>::default());

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
