use polars::prelude::*;
use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric, SeriesDomain};
use crate::error::*;
use crate::polars::{literal_value_of, ExprFunction};
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `to_date` expression.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The str.strptime expression
pub fn make_expr_strptime<M: OuterMetric>(
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
        input: inputs,
        function: FunctionExpr::StringExpr(StringFunction::Strptime(to_type, mut strptime_options)),
        ..
    } = expr
    else {
        return fallible!(MakeTransformation, "expected str.strptime expression");
    };

    let Ok([input, ambiguous]) = <&[_; 2]>::try_from(inputs.as_slice()) else {
        return fallible!(
            MakeTransformation,
            "str.strptime must have two arguments, found {}",
            inputs.len()
        );
    };

    if !matches!(
        to_type,
        DataType::Time | DataType::Datetime(_, _) | DataType::Date
    ) {
        return fallible!(
            MakeTransformation,
            "str.strptime dtype must be Time, Datetime or Date, found {}",
            to_type
        );
    }
    // Strict casting makes the transformation unstable: errors tell you things about private data.
    // Could throw an error if strict, but it is the default, so for ease-of-use it is forced to be non-strict.
    // It is also ok for overflow to wraparound.
    strptime_options.strict = false;

    if strptime_options.format.is_none() {
        return fallible!(MakeTransformation, "format must be specified; otherwise Polars will attempt to infer the format from the data, resulting in an unstable transformation");
    }

    if matches!(to_type, DataType::Time) && !strptime_options.exact {
        return fallible!(
            MakeTransformation,
            "non-exact not implemented for Time data type"
        );
    }

    // never raise on error
    let ambiguous_str = literal_value_of(ambiguous)
        .ok()
        .flatten()
        .unwrap_or("raise".to_string());
    let ambiguous = if ["earliest".to_string(), "latest".to_string()].contains(&ambiguous_str) {
        lit(ambiguous_str)
    } else {
        lit("null")
    };

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let series_domain = output_domain.active_series_mut()?;
    let name = series_domain.field.name.as_ref();

    if !series_domain.field.dtype.is_string() {
        return fallible!(
            MakeTransformation,
            "expected a string input type, got {}",
            series_domain.field.dtype
        );
    }

    *series_domain = SeriesDomain::new_from_field(Field::new(name, to_type.clone()))?;

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            output_domain,
            Function::then_expr(move |expr| {
                expr.str()
                    .strptime(to_type.clone(), strptime_options.clone(), ambiguous.clone())
            }),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}
