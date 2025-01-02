use polars::prelude::*;
use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::polars::literal_value_of;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `strptime` expression.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The str.strptime expression
pub fn make_expr_strptime<M: OuterMetric>(
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

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    if strptime_options.format.is_none() {
        return fallible!(MakeTransformation, "format must be specified; otherwise Polars will attempt to infer the format from the data, resulting in an unstable transformation");
    }

    if matches!(to_type, DataType::Time) && !strptime_options.exact {
        return fallible!(
            MakeTransformation,
            "non-exact not implemented for Time data type"
        );
    }

    // Strict casting makes the transformation unstable: errors tell you things about private data.
    // Could throw an error if strict, but it is the default, so for ease-of-use it is forced to be non-strict.
    // It is also ok for overflow to wraparound (but may be undesirable for users).
    strptime_options.strict = false;

    // never raise on error
    let ambiguous = lit(match literal_value_of::<String>(ambiguous) {
        Ok(Some(a)) if ["earliest", "latest"].contains(&a.as_str()) => a,
        _ => "null".to_string(),
    });

    let mut output_domain = middle_domain.clone();
    let series_domain = &mut output_domain.column;

    // check input and output types
    if series_domain.dtype() != DataType::String {
        return fallible!(
            MakeTransformation,
            "str.strptime input dtype must be String, found {}",
            series_domain.dtype()
        );
    }

    if matches!(to_type, DataType::Datetime(TimeUnit::Nanoseconds, _)) {
        // Nanoseconds are not supported due to this issue:
        // https://github.com/pola-rs/polars/issues/19928
        return fallible!(MakeMeasurement, "Nanoseconds are not currently supported due to potential panics when parsing inputs. Please open an issue on the OpenDP repository if you would find this functionality useful. Otherwise, consider parsing into micro- or millisecond datetimes instead.");
    }

    if !matches!(
        to_type,
        DataType::Time
            | DataType::Datetime(TimeUnit::Microseconds | TimeUnit::Milliseconds, _)
            | DataType::Date
    ) {
        return fallible!(
            MakeTransformation,
            "str.strptime output dtype must be Time, micro- or milli-second Datetime or Date, found {}",
            to_type
        );
    }

    series_domain.set_dtype(to_type.clone())?;
    series_domain.nullable = true;

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
