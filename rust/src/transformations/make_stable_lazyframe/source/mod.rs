use std::collections::HashSet;

use crate::core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{
    strip_table_markers, table_name_from_schema_or_domain, Database, DatabaseDomain,
    DslPlanDomain,
};
use crate::error::*;
use crate::metrics::{Bound, Bounds, DatabaseIdDistance, FrameDistance, PolarsMetric, SymmetricIdDistance};

use super::{
    StableDatabaseDslPlan, database_metric, table_metric,
    filter::make_chain_filter,
    group_by::make_chain_group_by,
    h_stack::make_chain_h_stack,
    select::make_chain_select,
    truncate::match_truncations,
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
    let DslPlan::DataFrameScan { df, schema } = &plan else {
        return fallible!(MakeTransformation, "Expected dataframe scan");
    };

    let table_name = table_name_from_schema_or_domain(schema, &input_domain)?;
    let output_domain = input_domain.get(&table_name)?.cast_carrier::<DslPlan>();
    let output_metric = FrameDistance(table_metric(&input_metric, &table_name));

    let active_exprs = output_metric
        .0
        .active_id_sites()
        .iter()
        .flat_map(|site| site.exprs.iter().cloned())
        .collect::<HashSet<_>>();

    let stripped_df = strip_table_markers(df)?;
    if !output_domain.cast_carrier::<DataFrame>().member(&stripped_df)? {
        return fallible!(
            MakeTransformation,
            "dataframe scan for table {} is not a member of the declared table domain",
            table_name
        );
    }

    Transformation::new(
        input_domain,
        input_metric,
        output_domain,
        output_metric,
        Function::new_fallible(move |arg: &Database| {
            arg.get(&table_name)
                .cloned()
                .map(|lf| lf.logical_plan)
                .ok_or_else(|| err!(FailedFunction, "missing table in database input: {}", table_name))
        }),
        StabilityMap::new_fallible(move |d_in: &u32| {
            if active_exprs.is_empty() {
                return Ok(Bounds::from(*d_in));
            }

            Ok(Bounds::from(*d_in).with_bound(
                Bound::by(active_exprs.iter().cloned().collect::<Vec<_>>())
                    .with_num_groups(*d_in)
                    .with_per_group(1),
            ))
        }),
    )
}

pub fn make_stable_database_metric_source<M: crate::transformations::traits::UnboundedMetric + crate::metrics::PolarsMetric>(
    input_domain: DatabaseDomain,
    input_metric: FrameDistance<M>,
    plan: DslPlan,
) -> Fallible<Transformation<DatabaseDomain, FrameDistance<M>, DslPlanDomain, FrameDistance<M>>>
where
    (DatabaseDomain, FrameDistance<M>): MetricSpace,
    (DslPlanDomain, FrameDistance<M>): MetricSpace,
{
    let DslPlan::DataFrameScan { schema, .. } = &plan else {
        return fallible!(MakeTransformation, "Expected dataframe scan");
    };
    let table_name = table_name_from_schema_or_domain(schema, &input_domain)?;
    let output_domain = input_domain.get(&table_name)?.cast_carrier::<DslPlan>();

    Transformation::new(
        input_domain,
        input_metric.clone(),
        output_domain,
        input_metric,
        Function::new_fallible(move |arg: &Database| {
            arg.get(&table_name)
                .cloned()
                .map(|lf| lf.logical_plan)
                .ok_or_else(|| err!(FailedFunction, "missing table in database input: {}", table_name))
        }),
        StabilityMap::new(Clone::clone),
    )
}

impl<M> StableDatabaseDslPlan<FrameDistance<M>, FrameDistance<M>> for DslPlan
where
    M: crate::transformations::traits::UnboundedMetric + crate::metrics::PolarsMetric,
{
    fn make_stable_database(
        self,
        input_domain: DatabaseDomain,
        input_metric: FrameDistance<M>,
    ) -> Fallible<Transformation<DatabaseDomain, FrameDistance<M>, DslPlanDomain, FrameDistance<M>>>
    {
        match self {
            DslPlan::IR { dsl, .. } => (*dsl).clone().make_stable_database(input_domain, input_metric),
            plan @ DslPlan::DataFrameScan { .. } => make_stable_database_metric_source(input_domain, input_metric, plan),
            DslPlan::Filter { input, predicate } => {
                let t_prior = input
                    .as_ref()
                    .clone()
                    .make_stable_database(input_domain, input_metric)?;
                make_chain_filter(t_prior, predicate)
            }
            DslPlan::HStack { input, exprs, options } => {
                let t_prior = input
                    .as_ref()
                    .clone()
                    .make_stable_database(input_domain, input_metric)?;
                make_chain_h_stack(t_prior, exprs, options)
            }
            DslPlan::Select { input, expr, options } => {
                let t_prior = input
                    .as_ref()
                    .clone()
                    .make_stable_database(input_domain, input_metric)?;
                make_chain_select(t_prior, expr, options)
            }
            DslPlan::GroupBy {
                input,
                keys,
                predicates,
                aggs,
                apply,
                maintain_order,
                options,
                ..
            } => {
                let t_prior = input
                    .as_ref()
                    .clone()
                    .make_stable_database(input_domain, input_metric)?;
                make_chain_group_by(t_prior, keys, predicates, aggs, apply, maintain_order, options)
            }
            dsl => match dsl.describe() {
                Ok(describe) => fallible!(
                    MakeTransformation,
                    "A database-aware step in your query is not recognized at this time: {:?}. {:?}If you would like to see this supported, please file an issue.",
                    describe,
                    crate::polars::get_disabled_features_message()
                ),
                Err(e) => fallible!(
                    MakeTransformation,
                    "A database-aware step in your query is not recognized at this time, and the step cannot be identified due to the following error: {}. {:?}",
                    e,
                    crate::polars::get_disabled_features_message()
                ),
            },
        }
    }
}

impl StableDatabaseDslPlan<DatabaseIdDistance, FrameDistance<SymmetricIdDistance>> for DslPlan {
    fn make_stable_database(
        self,
        input_domain: DatabaseDomain,
        input_metric: DatabaseIdDistance,
    ) -> Fallible<
        Transformation<
            DatabaseDomain,
            DatabaseIdDistance,
            DslPlanDomain,
            FrameDistance<SymmetricIdDistance>,
        >,
    > {
        let plan = match self {
            DslPlan::IR { dsl, .. } => return (*dsl).clone().make_stable_database(input_domain, input_metric),
            plan => plan,
        };

        if let Some(identifier) = crate::metrics::unique_id_expr(
            &database_metric(&input_metric).active_id_sites(),
        )? {
            if !match_truncations(plan.clone(), &identifier)?.1.is_empty() {
                return fallible!(
                    MakeTransformation,
                    "joins are only supported before truncation; convert to event-level stability after truncation"
                );
            }
        }

        match plan {
            plan @ DslPlan::DataFrameScan { .. } => {
                make_stable_database_source(input_domain, input_metric, plan)
            }
            DslPlan::Filter { input, predicate } => {
                let t_prior = input
                    .as_ref()
                    .clone()
                    .make_stable_database(input_domain, input_metric)?;
                make_chain_filter(t_prior, predicate)
            }
            DslPlan::HStack { input, exprs, options } => {
                let t_prior = input
                    .as_ref()
                    .clone()
                    .make_stable_database(input_domain, input_metric)?;
                make_chain_h_stack(t_prior, exprs, options)
            }
            DslPlan::Select { input, expr, options } => {
                let t_prior = input
                    .as_ref()
                    .clone()
                    .make_stable_database(input_domain, input_metric)?;
                make_chain_select(t_prior, expr, options)
            }
            DslPlan::GroupBy {
                input,
                keys,
                predicates,
                aggs,
                apply,
                maintain_order,
                options,
                ..
            } => {
                let t_prior = input
                    .as_ref()
                    .clone()
                    .make_stable_database(input_domain, input_metric)?;
                make_chain_group_by(t_prior, keys, predicates, aggs, apply, maintain_order, options)
            }
            plan @ DslPlan::Join { .. } => super::join::make_stable_database_join(input_domain, input_metric, plan),
            dsl => match dsl.describe() {
                Ok(describe) => fallible!(
                    MakeTransformation,
                    "A database-aware step in your query is not recognized at this time: {:?}. {:?}If you would like to see this supported, please file an issue.",
                    describe,
                    crate::polars::get_disabled_features_message()
                ),
                Err(e) => fallible!(
                    MakeTransformation,
                    "A database-aware step in your query is not recognized at this time, and the step cannot be identified due to the following error: {}. {:?}",
                    e,
                    crate::polars::get_disabled_features_message()
                ),
            },
        }
    }
}
