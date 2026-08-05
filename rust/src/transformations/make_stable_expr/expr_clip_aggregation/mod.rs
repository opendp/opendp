use polars_plan::dsl::{Expr, FunctionExpr};

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{L01InfDistance, LpDistance};
use crate::transformations::traits::UnboundedMetric;

use super::{StableExpr, expr_clip::extract_bounds};

#[cfg(test)]
mod test;

/// Make a Transformation that clips the output of an aggregation.
pub fn make_clip_aggregation<MI, const P: usize>(
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
    let Expr::Function {
        input, function, ..
    } = expr
    else {
        return fallible!(MakeTransformation, "expected function expression");
    };

    let FunctionExpr::Clip { has_min, has_max } = function else {
        return fallible!(MakeTransformation, "expected clip function");
    };
    if !has_min || !has_max {
        return fallible!(MakeTransformation, "Clip must have min and max");
    }

    let n_args = input.len();
    let [input, lower, upper] = <[Expr; 3]>::try_from(input).map_err(|_| {
        err!(
            MakeTransformation,
            "Clip expects 3 arguments, found {}",
            n_args
        )
    })?;

    let t_prior = input.make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let (lower, upper) = extract_bounds(lower, upper, &mut output_domain.column)?;

    t_prior
        >> Transformation::new(
            middle_domain,
            middle_metric.clone(),
            output_domain,
            middle_metric,
            Function::then_expr(move |expr| expr.clip(lower.clone(), upper.clone())),
            StabilityMap::new(Clone::clone),
        )?
}
