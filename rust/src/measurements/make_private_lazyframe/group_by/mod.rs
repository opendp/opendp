use std::fmt::Debug;
use std::sync::Arc;

use crate::accuracy::{
    conservative_discrete_gaussian_tail_to_alpha, conservative_discrete_laplacian_tail_to_alpha,
};
use crate::combinators::{make_basic_composition, BasicCompositionMeasure};
use crate::core::{Function, Measurement, MetricSpace, PrivacyMap};
use crate::domains::{DslPlanDomain, ExprContext, ExprDomain};
use crate::error::*;
use crate::measurements::expr_noise::Distribution;
use crate::measurements::make_private_expr;
use crate::measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence};
use crate::metrics::PartitionDistance;
use crate::traits::{InfAdd, InfMul, InfPowI, InfSub};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{DatasetMetric, StableDslPlan};
use dashu::integer::IBig;
use make_private_expr::PrivateExpr;
use matching::{find_len_expr, match_grouping_columns, MatchGroupBy};
use polars_plan::dsl::{all, col, lit, Expr};
use polars_plan::plans::DslPlan;

#[cfg(test)]
mod test;

mod matching;
pub(crate) use matching::{is_threshold_predicate, match_group_by};

/// Create a private version of an aggregate operation on a LazyFrame.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `output_measure` - The measure of the output LazyFrame.
/// * `plan` - The LazyFrame to transform.
/// * `global_scale` - The parameter for the measurement.
/// * `threshold` - Only keep groups with length greater than threshold
pub fn make_private_group_by<MS, MI, MO>(
    input_domain: DslPlanDomain,
    input_metric: MS,
    output_measure: MO,
    plan: DslPlan,
    global_scale: Option<f64>,
    threshold: Option<u32>,
) -> Fallible<Measurement<DslPlanDomain, DslPlan, MS, MO>>
where
    MS: 'static + DatasetMetric,
    MI: 'static + UnboundedMetric,
    MO: 'static + ApproximateMeasure,
    MO::Distance: Debug,
    Expr: PrivateExpr<PartitionDistance<MI>, MO>,
    DslPlan: StableDslPlan<MS, MI>,
    (DslPlanDomain, MS): MetricSpace,
    (DslPlanDomain, MI): MetricSpace,
    (ExprDomain, MI): MetricSpace,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
{
    let Some(MatchGroupBy {
        input: input_expr,
        keys,
        aggs,
        predicate,
    }) = match_group_by(plan)?
    else {
        return fallible!(MakeMeasurement, "expected group by");
    };

    let t_prior = input_expr.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let grouping_columns = match_grouping_columns(keys.clone())?;

    let margin = middle_domain
        .margins
        .get(&grouping_columns)
        .cloned()
        .unwrap_or_default();

    let expr_domain = ExprDomain::new(
        middle_domain.clone(),
        ExprContext::Aggregate {
            grouping_columns: grouping_columns.clone(),
        },
    );

    let m_exprs = make_basic_composition(
        aggs.into_iter()
            .map(|expr| {
                make_private_expr(
                    expr_domain.clone(),
                    PartitionDistance(middle_metric.clone()),
                    output_measure.clone(),
                    expr.clone(),
                    global_scale,
                )
            })
            .collect::<Fallible<_>>()?,
    )?;

    let f_comp = m_exprs.function.clone();
    let privacy_map = m_exprs.privacy_map.clone();
    let dp_exprs = m_exprs.invoke(&(input_expr, all()))?;

    let threshold_info = if margin.public_info.is_some() {
        None
    } else if let Some((name, threshold_value)) =
        predicate.clone().and_then(|p| is_threshold_predicate(p))
    {
        let noise = find_len_expr(&dp_exprs, Some(name.as_str()))?.1;
        Some((name, noise, threshold_value, false))
    } else if let Some(threshold_value) = threshold {
        let (name, noise) = find_len_expr(&dp_exprs, None)?;
        Some((name, noise, threshold_value, true))
    } else {
        return fallible!(MakeMeasurement, "The key-set of {:?} is private and cannot be released without filtering. Please pass a filtering threshold into make_private_lazyframe.", grouping_columns);
    };

    let final_predicate = if let Some((name, _, threshold_value, is_present)) = &threshold_info {
        let threshold_expr = col(&name).gt(lit(*threshold_value));
        Some(match (is_present, predicate) {
            (false, Some(predicate)) => threshold_expr.and(predicate),
            _ => threshold_expr,
        })
    } else {
        predicate
    };

    let m_group_by_agg = Measurement::new(
        middle_domain,
        Function::new_fallible(move |arg: &DslPlan| {
            let mut output = DslPlan::GroupBy {
                input: Arc::new(arg.clone()),
                keys: keys.clone(),
                aggs: f_comp.eval(&(arg.clone(), all()))?,
                apply: None,
                maintain_order: false,
                options: Default::default(),
            };

            if let Some(final_predicate) = &final_predicate {
                output = DslPlan::Filter {
                    input: Arc::new(output),
                    predicate: final_predicate.clone(),
                }
            }
            Ok(output)
        }),
        middle_metric,
        output_measure.clone(),
        PrivacyMap::new_fallible(move |&d_in| {
            let mip = margin.max_influenced_partitions.unwrap_or(d_in);
            let mnp = margin.max_num_partitions.unwrap_or(d_in);
            let mpc = margin.max_partition_contributions.unwrap_or(d_in);
            let mpl = margin.max_partition_length.unwrap_or(d_in);

            let l0 = mip.min(mnp).min(d_in);
            let li = mpc.min(mpl).min(d_in);
            let l1 = l0.inf_mul(&li)?.min(d_in);

            let mut d_out = privacy_map.eval(&(l0, l1, li))?;

            if let Some((_, noise, threshold_value, _)) = &threshold_info {
                if li >= *threshold_value {
                    return fallible!(FailedMap, "threshold must be greater than {:?}", li);
                }

                let d_instability = threshold_value.inf_sub(&li)?;
                let delta_single =
                    integrate_discrete_noise_tail(noise.distribution, noise.scale, d_instability)?;
                let delta_joint = (1.0).inf_sub(
                    &(1.0)
                        .neg_inf_sub(&delta_single)?
                        .neg_inf_powi(IBig::from(l0))?,
                )?;
                d_out = MO::add_delta(d_out, delta_joint)?;
            } else if margin.public_info.is_none() {
                return fallible!(FailedMap, "key-sets cannot be privatized under {:?}. FixedSmoothedMaxDivergence is necessary.", output_measure);
            }

            Ok(d_out)
        }),
    )?;

    t_prior >> m_group_by_agg
}

pub trait ApproximateMeasure: BasicCompositionMeasure {
    fn add_delta(d_out: Self::Distance, delta_p: f64) -> Fallible<Self::Distance>;
}

macro_rules! impl_measure_non_catastrophic {
    ($ty:ty) => {
        impl ApproximateMeasure for $ty {
            fn add_delta(_d_out: Self::Distance, _delta_p: f64) -> Fallible<Self::Distance> {
                return fallible!(
                    MakeMeasurement,
                    "key-sets cannot be privatized under {:?}. Approximate-DP is necessary.",
                    stringify!($ty)
                );
            }
        }
    };
}

impl_measure_non_catastrophic!(MaxDivergence);
impl_measure_non_catastrophic!(ZeroConcentratedDivergence);

impl<MO: BasicCompositionMeasure> ApproximateMeasure for Approximate<MO>
where
    Self: BasicCompositionMeasure<Distance = (MO::Distance, f64)>,
{
    fn add_delta((d_out, delta): Self::Distance, delta_p: f64) -> Fallible<Self::Distance> {
        Ok((d_out, delta.inf_add(&delta_p)?))
    }
}

fn integrate_discrete_noise_tail(
    distribution: Distribution,
    scale: f64,
    tail_bound: u32,
) -> Fallible<f64> {
    match distribution {
        Distribution::Laplace => conservative_discrete_laplacian_tail_to_alpha(scale, tail_bound),
        Distribution::Gaussian => conservative_discrete_gaussian_tail_to_alpha(scale, tail_bound),
    }
}
