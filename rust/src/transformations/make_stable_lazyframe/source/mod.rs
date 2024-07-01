use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::LogicalPlanDomain;
use crate::error::*;
use crate::transformations::traits::UnboundedMetric;
use polars::prelude::*;

/// Placeholder transformation for creating a stable LazyFrame source.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_stable_source<M: Metric>(
    input_domain: LogicalPlanDomain,
    input_metric: M,
    plan: LogicalPlan,
) -> Fallible<Transformation<LogicalPlanDomain, LogicalPlanDomain, M, M>>
where
    M: UnboundedMetric + 'static,
    (LogicalPlanDomain, M): MetricSpace,
{
    let LogicalPlan::DataFrameScan {
        df: _, // DO NOT TOUCH THE DATA. Touching the data will degrade any downstream stability or privacy guarantees.
        projection,
        selection,
        schema,
        ..
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected Aggregate logical plan");
    };

    if projection.is_some() || selection.is_some() {
        return fallible!(MakeTransformation, "Lazyframe must not be optimized. Wait to optimize until after making a private lazyframe.");
    }

    let domain_schema = input_domain
        .series_domains
        .iter()
        .map(|s| s.field.clone())
        .collect::<Schema>();

    if &domain_schema != schema.as_ref() {
        return fallible!(
            MakeTransformation,
            "Schema mismatch. LazyFrame schema must match the schema from the input domain."
        );
    }

    Transformation::new(
        input_domain.clone(),
        input_domain,
        Function::new(Clone::clone),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}
