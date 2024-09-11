use polars::chunked_array::cast::CastOptions;
use polars::prelude::*;
use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric, SeriesDomain};
use crate::error::*;
use crate::polars::ExprFunction;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `cast(dtype)` expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The clipping expression
pub fn make_expr_cast<M: OuterMetric>(
    input_domain: ExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Cast {
        expr: input,
        data_type: to_type,
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
    let series_domain = output_domain.active_series_mut()?;
    let name = series_domain.field.name.as_ref();

    // it is possible to tighten this:
    // in cases where casting will never fail, the nullable and/or nan bits can be left false
    // in the meantime, users will need to impute
    *series_domain = SeriesDomain::new_from_field(Field::new(name, to_type.clone()))?;

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            output_domain,
            Function::then_expr(move |expr| Expr::Cast {
                expr: Arc::new(expr),
                data_type: to_type.clone(),
                // Specify behavior for when casting fails (this is forced to be non-strict).
                options,
            }),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}
