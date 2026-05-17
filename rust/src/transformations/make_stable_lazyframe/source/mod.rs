use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Database, DatabaseDomain, DslPlanDomain, table_name_from_schema_or_domain};
use crate::error::*;
use crate::metrics::{
    Bound, Bounds, DatabaseIdDistance, FrameDistance, SymmetricIdDistance, choose_owner_claim,
    expr_identifies_protected_id,
};
use polars::prelude::*;

#[cfg(test)]
mod test;

pub fn make_stable_source<M: Metric>(
    input_domain: DslPlanDomain,
    input_metric: M,
    plan: DslPlan,
) -> Fallible<Transformation<DslPlanDomain, M, DslPlanDomain, M>>
where
    (DslPlanDomain, M): MetricSpace,
    M::Distance: 'static + Clone,
{
    let DslPlan::DataFrameScan { .. } = plan else {
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

pub fn make_stable_database_source(
    input_domain: DatabaseDomain,
    input_metric: DatabaseIdDistance,
    plan: DslPlan,
) -> Fallible<
    Transformation<
        DatabaseDomain,
        DatabaseIdDistance,
        DslPlanDomain,
        FrameDistance<SymmetricIdDistance>,
    >,
> {
    let DslPlan::DataFrameScan { schema, .. } = &plan else {
        return fallible!(MakeTransformation, "Expected dataframe scan");
    };

    let table_name = table_name_from_schema_or_domain(schema, &input_domain)?;
    let output_domain = input_domain.table(&table_name)?.cast_carrier::<DslPlan>();
    let output_metric = FrameDistance(SymmetricIdDistance {
        protect: input_metric.protect.clone(),
        bindings: input_metric
            .bindings
            .get(&table_name)
            .cloned()
            .unwrap_or_default(),
        owner_claims: input_metric
            .base_owner_claims
            .get(&table_name)
            .cloned()
            .unwrap_or_default(),
    });
    let owner_claim = choose_owner_claim(
        &output_metric
            .0
            .owner_claims
            .iter()
            .filter(|claim| {
                claim.len() == 1
                    && expr_identifies_protected_id(
                        &output_metric.0.bindings,
                        &output_metric.0.protect,
                        &claim[0],
                    )
            })
            .cloned()
            .collect::<Vec<_>>(),
    );

    Transformation::new(
        input_domain,
        input_metric,
        output_domain,
        output_metric,
        Function::new_fallible(move |arg: &Database| {
            arg.get(&table_name)
                .cloned()
                .map(|lf| lf.logical_plan)
                .ok_or_else(|| {
                    err!(
                        FailedFunction,
                        "missing table in database input: {}",
                        table_name
                    )
                })
        }),
        StabilityMap::new(move |&d_in: &u32| {
            owner_claim.clone().map_or_else(
                || Bounds::from(d_in),
                |claim| {
                    let identifier = claim[0].clone();
                    Bounds::from(d_in).with_bound(
                        Bound::by([identifier])
                            .with_num_groups(d_in)
                            .with_per_group(1),
                    )
                },
            )
        }),
    )
}
