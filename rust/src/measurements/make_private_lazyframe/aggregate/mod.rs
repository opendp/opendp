use std::collections::BTreeSet;

use crate::combinators::{make_basic_composition, BasicCompositionMeasure};
use crate::core::{Function, Measurement, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprContext, ExprDomain, LogicalPlanDomain};
use crate::error::*;
use crate::measurements::make_private_expr;
use crate::metrics::PartitionDistance;
use crate::traits::InfMul;
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{DatasetMetric, StableLogicalPlan};
use make_private_expr::PrivateExpr;
use polars::prelude::*;
use polars_plan::prelude::GroupbyOptions;

/// Create a private version of an aggregate operation on a LazyFrame.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `output_measure` - The measure of the output LazyFrame.
/// * `param` - The parameter for the transformation.
/// * `plan` - The LazyFrame to transform.
pub fn make_private_aggregate<MS, MI, MO>(
    input_domain: LogicalPlanDomain,
    input_metric: MS,
    output_measure: MO,
    param: f64,
    plan: LogicalPlan,
) -> Fallible<Measurement<LogicalPlanDomain, LogicalPlan, MS, MO>>
where
    MS: 'static + DatasetMetric,
    MI: 'static + UnboundedMetric,
    MO: 'static + BasicCompositionMeasure,
    Expr: PrivateExpr<PartitionDistance<MI>, MO>,
    LogicalPlan: StableLogicalPlan<MS, MI>,
    (LogicalPlanDomain, MS): MetricSpace,
    (LogicalPlanDomain, MI): MetricSpace,
    (ExprDomain, MI): MetricSpace,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
{
    let LogicalPlan::Aggregate {
        input,
        keys,
        aggs,
        apply,
        options,
        ..
    } = plan.clone()
    else {
        return fallible!(MakeMeasurement, "Expected Aggregate logical plan");
    };

    let t_prior = input.make_stable(input_domain.clone(), input_metric.clone())?;
    let (input_domain, input_metric) = t_prior.output_space();

    if options.as_ref() != &GroupbyOptions::default() {
        return fallible!(MakeMeasurement, "Unsupported options in logical plan. Do not optimize the lazyframe passed into the constructor. Options should be default, but are {:?}", options);
    }

    if apply.is_some() {
        return fallible!(MakeMeasurement, "Apply is not supported in logical plan");
    }

    let keys = keys
        .iter()
        .map(|e| {
            Ok(match e {
                Expr::Column(name) => vec![(*name).to_string()],
                Expr::Columns(names) => names.clone(),
                e => {
                    return fallible!(
                        MakeMeasurement,
                        "Expected column expression in keys, found {:?}",
                        e
                    )
                }
            })
        })
        .collect::<Fallible<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<BTreeSet<_>>();

    let margin = input_domain
        .margins
        .get(&keys)
        .ok_or_else(|| err!(MakeMeasurement, "Failed to find margin for {:?}", keys))?
        .clone();

    let expr_domain = ExprDomain::new(
        input_domain.clone(),
        ExprContext::Aggregate {
            grouping_columns: keys,
        },
    );

    let t_group_by = Transformation::new(
        expr_domain.clone(),
        expr_domain.clone(),
        Function::new(Clone::clone),
        input_metric.clone(),
        PartitionDistance(input_metric.clone()),
        StabilityMap::new_fallible(move |&d_in| {
            let l0 = margin.max_influenced_partitions.unwrap_or(d_in).min(d_in);
            let li = margin.max_partition_contributions.unwrap_or(d_in).min(d_in);
            let l1 = l0.inf_mul(&li)?.min(d_in);
            Ok((l0, l1, li))
        }),
    )?;

    let m_exprs = (t_group_by
        >> make_basic_composition(
            aggs.into_iter()
                .map(|expr| {
                    make_private_expr(
                        expr_domain.clone(),
                        PartitionDistance(input_metric.clone()),
                        output_measure.clone(),
                        expr.clone(),
                        param,
                    )
                })
                .collect::<Fallible<_>>()?,
        )?)?;

    let f_exprs = m_exprs.function.clone();
    let privacy_map = m_exprs.privacy_map.clone();

    t_prior
        >> Measurement::new(
            input_domain,
            Function::new_fallible(move |arg: &LogicalPlan| {
                let mut output = plan.clone();
                if let LogicalPlan::Aggregate {
                    ref mut input,
                    ref mut aggs,
                    ..
                } = output
                {
                    *input = Box::new(arg.clone());
                    *aggs = f_exprs.eval(&(arg.clone(), all()))?;
                };
                Ok(output)
            }),
            input_metric,
            output_measure,
            privacy_map,
        )?
}

#[cfg(test)]
mod test_make_group_by {
    use crate::domains::{AtomDomain, Margin, SeriesDomain};
    use crate::error::ErrorVariant::MakeMeasurement;
    use crate::error::*;
    use crate::measures::MaxDivergence;
    use polars::prelude::*;

    use crate::metrics::SymmetricDistance;

    use super::*;

    #[test]
    fn test_aggregate() -> Fallible<()> {
        let lf_domain = LogicalPlanDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
            SeriesDomain::new("C", AtomDomain::<i32>::default()),
        ])?
        .with_margin(&["A", "C"], Margin::new().with_public_keys())?;

        let lf = df!(
            "A" => &[1i32, 2, 2],
            "B" => &[1.0f64, 2.0, 2.0],
            "C" => &[8i32, 9, 10],)?
        .lazy();

        let error_variant_res = make_private_aggregate::<_, SymmetricDistance, _>(
            lf_domain,
            SymmetricDistance,
            MaxDivergence::<f64>::default(),
            1.,
            lf.group_by(&[col("A"), col("C")])
                .agg(&[col("B").sum()])
                .logical_plan,
        )
        .map(|_| ())
        .unwrap_err()
        .variant;

        assert_eq!(MakeMeasurement, error_variant_res);

        Ok(())
    }
}
