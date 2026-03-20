use polars::prelude::*;
use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::metrics::MicrodataMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns an expression that extracts a datetime component.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The datetime component expression
pub fn make_expr_datetime_component<M: OuterMetric>(
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
        input: inputs,
        function: FunctionExpr::TemporalExpr(temporal_function),
        ..
    } = expr
    else {
        return fallible!(MakeTransformation, "expected datetime component expression");
    };

    let Some(to_dtype) = match_datetime_component(&temporal_function) else {
        return fallible!(
            MakeTransformation,
            "expected datetime component, found {:?}",
            temporal_function
        );
    };

    let Ok([input]) = <&[_; 1]>::try_from(inputs.as_slice()) else {
        return fallible!(
            MakeTransformation,
            "{} must have one arguments, found {}",
            temporal_function,
            inputs.len()
        );
    };

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let in_dtype = middle_domain.column.dtype();
    if !matches!(
        in_dtype,
        DataType::Time | DataType::Datetime(_, _) | DataType::Date
    ) {
        return fallible!(
            MakeTransformation,
            "expected a temporal input type, got {}",
            in_dtype
        );
    }

    let mut output_domain = middle_domain.clone();
    output_domain.column.set_dtype(to_dtype)?;

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            middle_metric.clone(),
            output_domain,
            middle_metric,
            Function::then_expr(move |expr| Expr::Function {
                input: vec![expr],
                function: FunctionExpr::TemporalExpr(temporal_function.clone()),
            }),
            StabilityMap::new(Clone::clone),
        )?
}

/// # Proof Definition
/// For any choice of `temporal_function`,
/// returns the data type of the output if the input is an infallible row-by-row temporal expression,
/// otherwise returns none.
pub(super) fn match_datetime_component(temporal_function: &TemporalFunction) -> Option<DataType> {
    use TemporalFunction::*;
    Some(match temporal_function {
        Millennium => DataType::Int32,
        Century => DataType::Int32,
        Year => DataType::Int32,
        IsoYear => DataType::Int32,
        Quarter => DataType::Int8,
        Month => DataType::Int8,
        Week => DataType::Int8,
        WeekDay => DataType::Int8,
        Day => DataType::Int8,
        OrdinalDay => DataType::Int16,
        Hour => DataType::Int8,
        Minute => DataType::Int8,
        Second => DataType::Int8,
        Millisecond => DataType::Int32,
        Microsecond => DataType::Int32,
        Nanosecond => DataType::Int32,
        _ => return None,
    })
}
