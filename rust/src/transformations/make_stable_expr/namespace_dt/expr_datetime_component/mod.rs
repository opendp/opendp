use std::collections::{BTreeSet, HashMap};

use polars::prelude::*;
use polars_plan::dsl::Expr;
use polars_plan::prelude::{ApplyOptions, FunctionOptions};

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric, SeriesDomain};
use crate::error::*;
use crate::polars::ExprFunction;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns an expression that extracts a datetime component.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The dt expression
pub fn make_expr_datetime_component<M: OuterMetric>(
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
        function: FunctionExpr::TemporalExpr(temporal_function),
        ..
    } = expr
    else {
        return fallible!(MakeTransformation, "expected dt expression");
    };

    let Some((to_dtype, max_num_partitions)) = match_datetime_component(temporal_function.clone())
    else {
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

    let in_dtype = middle_domain.active_series()?.field.dtype.clone();
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
    let series_domain = output_domain.active_series_mut()?;
    let name = series_domain.field.name.to_string();

    *series_domain = SeriesDomain::new_from_field(Field::new(name.as_str(), to_dtype.clone()))?;
    let margin_id = BTreeSet::from([name.to_string()]);
    let mut out_margin = output_domain.frame_domain.get_margin(margin_id.clone());
    out_margin.max_num_partitions = max_num_partitions;
    output_domain.frame_domain.margins = HashMap::from([(margin_id, out_margin)]);

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            output_domain,
            Function::then_expr(move |expr| Expr::Function {
                input: vec![expr],
                function: FunctionExpr::TemporalExpr(temporal_function.clone()),
                options: FunctionOptions {
                    collect_groups: ApplyOptions::ElementWise,
                    ..Default::default()
                },
            }),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}

/// # Proof Definition
/// Returns the data type and optionally the maximum number of unique values of the output
/// when applying `temporal_function` to a temporal data type.
/// Returns None if the temporal function is not retrieving a datetime component.
pub(super) fn match_datetime_component(
    temporal_function: TemporalFunction,
) -> Option<(DataType, Option<u32>)> {
    use TemporalFunction::*;
    Some(match temporal_function {
        Millennium => (DataType::UInt32, None),
        Century => (DataType::Int32, None),
        Year => (DataType::Int32, None),
        IsoYear => (DataType::Int32, None),
        Quarter => (DataType::Int8, Some(4)),
        Month => (DataType::Int8, Some(12)),
        Week => (DataType::Int8, Some(53)),
        WeekDay => (DataType::Int8, Some(7)),
        Day => (DataType::Int8, Some(31)),
        OrdinalDay => (DataType::Int16, Some(366)),
        Hour => (DataType::Int8, Some(24)),
        Minute => (DataType::Int8, Some(60)),
        Second => (DataType::Int8, Some(60)),
        Millisecond => (DataType::Int32, None),
        Microsecond => (DataType::Int32, None),
        Nanosecond => (DataType::Int32, None),
        _ => return None,
    })
}
