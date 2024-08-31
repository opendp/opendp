use polars::prelude::*;
use polars_plan::dsl::{Expr, FunctionExpr};

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{CategoricalDomain, ExprDomain, OuterMetric};
use crate::error::*;
use crate::polars::ExprFunction;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `cut` expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The clipping expression
pub fn make_expr_cut<M: OuterMetric>(
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
    let Expr::Function {
        input, function, ..
    } = expr
    else {
        return fallible!(MakeTransformation, "expected function expression");
    };

    let FunctionExpr::Cut {
        breaks,
        labels,
        left_closed,
        include_breaks,
    } = function
    else {
        return fallible!(MakeTransformation, "expected cut function");
    };

    if include_breaks {
        return fallible!(
            MakeTransformation,
            "include_breaks in cut is not currently supported"
        );
    }

    let n_args = input.len();
    let [input] = <[Expr; 1]>::try_from(input).map_err(|_| {
        err!(
            MakeTransformation,
            "cut expects 1 data argument, found {}",
            n_args
        )
    })?;

    let t_prior = input.make_stable(input_domain.clone(), input_metric.clone())?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let categories = if let Some(labels) = &labels {
        if labels.len() != breaks.len() + 1 {
            return fallible!(
                MakeTransformation,
                "cut must have {} labels, found {} labels",
                breaks.len() + 1,
                labels.len()
            );
        }
        labels.clone()
    } else {
        compute_labels(&breaks, left_closed)?
    };
    let series_domain = output_domain.active_series_mut()?;
    series_domain.element_domain = Arc::new(CategoricalDomain::new_with_encoding(categories)?);
    series_domain.field.dtype = DataType::Categorical(None, Default::default());

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            output_domain,
            Function::then_expr(move |expr| {
                expr.cut(breaks.clone(), labels.clone(), left_closed, include_breaks)
            }),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}
