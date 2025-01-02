use polars::prelude::*;
use polars_plan::dsl::{Expr, FunctionExpr};

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, CategoricalDomain, ExprDomain, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `to_physical` expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The clipping expression
pub fn make_expr_to_physical<M: OuterMetric>(
    input_domain: WildExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Function {
        input, function, ..
    } = expr
    else {
        return fallible!(MakeTransformation, "expected function expression");
    };

    if !matches!(function, FunctionExpr::ToPhysical) {
        return fallible!(
            MakeTransformation,
            "expected to_physical function, found {}",
            function
        );
    };

    let n_args = input.len();
    let [input] = <[Expr; 1]>::try_from(input).map_err(|_| {
        err!(
            MakeTransformation,
            "to_physical expects 1 data argument, found {}",
            n_args
        )
    })?;

    let t_prior = input.make_stable(input_domain.clone(), input_metric.clone())?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();

    let active_series = &mut output_domain.column;

    use DataType::*;
    let in_dtype = active_series.dtype();

    // this code is written intentionally to fail or change if polars behavior changes
    match (in_dtype.clone(), in_dtype.to_physical()) {
        (in_dtype, out_dtype) if in_dtype == out_dtype => (),
        (Categorical(_, _), UInt32) => {
            let cat_domain = active_series.element_domain::<CategoricalDomain>()?;

            if cat_domain.categories().is_none() {
                return fallible!(MakeTransformation, "to_physical: to prevent potentially revealing information about row ordering, category ordering must be statically known. Convert to String first.");
            }

            active_series.set_element_domain(AtomDomain::<u32>::default());
        }
        (Date, Int32) => {
            active_series.set_element_domain(AtomDomain::<u32>::default());
        }
        (Datetime(_, _) | Time | Duration(_), Int64) => {
            active_series.set_element_domain(AtomDomain::<i64>::default());
        }
        (in_dtype, _) => {
            return fallible!(
                MakeTransformation,
                "to_physical unsupported dtype: {}",
                in_dtype
            )
        }
    };

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            output_domain,
            Function::then_expr(move |expr| expr.to_physical()),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}
