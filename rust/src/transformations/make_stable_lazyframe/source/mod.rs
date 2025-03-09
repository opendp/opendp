use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::DslPlanDomain;
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
    input_domain: DslPlanDomain,
    input_metric: M,
    plan: DslPlan,
) -> Fallible<Transformation<DslPlanDomain, DslPlanDomain, M, M>>
where
    M: UnboundedMetric + 'static,
    (DslPlanDomain, M): MetricSpace,
{
    let DslPlan::DataFrameScan {
        df: _, // DO NOT TOUCH THE DATA. Touching the data will degrade any downstream stability or privacy guarantees.
        schema,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected dataframe scan");
    };

    if &input_domain.schema() != schema.as_ref() {
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
