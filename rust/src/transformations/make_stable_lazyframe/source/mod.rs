use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::DslPlanDomain;
use crate::error::*;
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
) -> Fallible<Transformation<DslPlanDomain, M, DslPlanDomain, M>>
where
    (DslPlanDomain, M): MetricSpace,
    M::Distance: 'static + Clone,
{
    let DslPlan::DataFrameScan {
        df: _, // DO NOT TOUCH THE DATA. Touching the data will degrade any downstream stability or privacy guarantees.
        ..
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected dataframe scan");
    };

    Transformation::new(
        input_domain.clone(),
        input_metric.clone(),
        input_domain,
        input_metric,
        Function::new(Clone::clone),
        StabilityMap::new(Clone::clone),
    )
}
