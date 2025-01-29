use std::fmt::Debug;
use std::sync::Arc;

use crate::accuracy::{
    conservative_discrete_gaussian_tail_to_alpha, conservative_discrete_laplacian_tail_to_alpha,
};
use crate::combinators::{make_basic_composition, BasicCompositionMeasure};
use crate::core::{Function, Measurement, MetricSpace, PrivacyMap};
use crate::domains::{CategoricalDomain, Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::measurements::expr_noise::Distribution;
use crate::measurements::make_private_expr;
use crate::measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence};
use crate::metrics::PartitionDistance;
use crate::traits::{InfAdd, InfMul, InfPowI, InfSub};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{DatasetMetric, StableDslPlan, StableExpr};
use dashu::integer::IBig;
use make_private_expr::PrivateExpr;
use matching::find_len_expr;
use polars::prelude::{len, JoinType, LazyFrame};
use polars_plan::dsl::{col, lit, Expr};
use polars_plan::plans::DslPlan;

#[cfg(test)]
mod test;

mod matching;
pub(crate) use matching::{is_threshold_predicate, match_group_by, KeySanitizer, MatchGroupBy};
use polars_plan::prelude::ProjectionOptions;

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
    Expr: PrivateExpr<PartitionDistance<MI>, MO>
        + StableExpr<PartitionDistance<MI>, PartitionDistance<MI>>,
    DslPlan: StableDslPlan<MS, MI>,
    (DslPlanDomain, MS): MetricSpace,
    (DslPlanDomain, MI): MetricSpace,
{
    let Some(MatchGroupBy {
        input,
        group_by,
        aggs,
        mut key_sanitizer,
    }) = match_group_by(plan)?
    else {
        return fallible!(MakeMeasurement, "expected group by");
    };

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    // create a transformation for each expression
    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::RowByRow,
    };
    let t_group_by = group_by
        .iter()
        .map(|expr| {
            expr.clone().make_stable(
                expr_domain.clone(),
                PartitionDistance(middle_metric.clone()),
            )
        })
        .collect::<Fallible<Vec<_>>>()?;

    t_group_by.iter().try_for_each(|t_group_by| {
        let series_domain = &t_group_by.output_domain.column;
        let Ok(domain) = series_domain.element_domain::<CategoricalDomain>() else {
            return Ok(())
        };
        if domain.categories().is_none() {
            return fallible!(
                MakeMeasurement,
                "Categories are data-dependent, which may reveal sensitive record ordering. Cast {} to string before grouping.", 
                series_domain.name
            );
        }
        Ok(())
    })?;
    let group_by_id = group_by.iter().cloned().collect();

    let mut margin = middle_domain.get_margin(&group_by_id);

    let is_join = if let Some(KeySanitizer::Join { keys, .. }) = key_sanitizer.clone() {
        let num_keys = LazyFrame::from((*keys).clone()).select([len()]).collect()?;
        margin.max_num_partitions = Some(num_keys.column("len")?.u32()?.last().unwrap());

        true
    } else {
        false
    };

    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::Aggregation {
            margin: margin.clone(),
        },
    };

    let m_exprs = make_basic_composition(
        aggs.into_iter()
            .map(|expr| {
                make_private_expr(
                    expr_domain.clone(),
                    PartitionDistance(middle_metric.clone()),
                    output_measure.clone(),
                    expr,
                    global_scale,
                )
            })
            .collect::<Fallible<_>>()?,
    )?;

    let f_comp = m_exprs.function.clone();
    let privacy_map = m_exprs.privacy_map.clone();
    let (dp_exprs, null_exprs): (_, Vec<Option<Expr>>) = m_exprs
        .invoke(&input)?
        .into_iter()
        .map(|ep| (ep.expr, ep.fill))
        .unzip();

    let threshold_info = if margin.public_info.is_some() || is_join {
        None
    } else if let Some((name, threshold_value)) = match_filter(&key_sanitizer) {
        let noise = find_len_expr(&dp_exprs, Some(name.as_str()))?.1;
        Some((name, noise, threshold_value, false))
    } else if let Some(threshold_value) = threshold {
        let (name, noise) = find_len_expr(&dp_exprs, None)?;
        Some((name, noise, threshold_value, true))
    } else {
        return fallible!(MakeMeasurement, "The key-set of {:?} is private and cannot be released without a filter or join. Please pass a filtering threshold into make_private_lazyframe or conduct a join against a public key-set.", group_by_id);
    };

    if let Some((name, _, threshold_value, is_present)) = &threshold_info {
        let threshold_expr = col(name).gt(lit(*threshold_value));
        key_sanitizer = Some(KeySanitizer::Filter(match (is_present, key_sanitizer) {
            (false, Some(KeySanitizer::Filter(predicate))) => threshold_expr.and(predicate),
            _ => threshold_expr,
        }))
    } else if let Some(KeySanitizer::Join { fill_null, .. }) = key_sanitizer.as_mut() {
        *fill_null = Some(
            dp_exprs.iter().zip(null_exprs
                .into_iter())
                .map(|(dp_expr, null_expr)| {
                    let name = dp_expr.clone().meta().output_name()?;
                    let null_expr = null_expr.ok_or_else(|| {
                        let name = dp_expr.clone().meta().output_name().map_or_else(|_| "an expression".to_string(), |n| format!("column \"{n}\""));
                        err!(MakeMeasurement, "{} can't be joined with an explicit key set because missing partitions cannot be filled", name)
                    })?;

                    Ok(col(name).fill_null(null_expr))
                })
                .collect::<Fallible<_>>()?,
        );
    }

    let m_group_by_agg = Measurement::new(
        middle_domain,
        Function::new_fallible(move |arg: &DslPlan| {
            let output = DslPlan::GroupBy {
                input: Arc::new(arg.clone()),
                keys: group_by.clone(),
                aggs: f_comp.eval(&arg)?.into_iter().map(|p| p.expr).collect(),
                apply: None,
                maintain_order: false,
                options: Default::default(),
            };
            Ok(match key_sanitizer.clone() {
                Some(KeySanitizer::Filter(predicate)) => DslPlan::Filter {
                    input: Arc::new(output),
                    predicate: predicate.clone(),
                },
                Some(KeySanitizer::Join {
                    keys: labels,
                    how,
                    left_on,
                    right_on,
                    options,
                    fill_null,
                }) => {
                    let (input_left, input_right) = match how {
                        JoinType::Left => (labels, Arc::new(output)),
                        JoinType::Right => (Arc::new(output), labels),
                        _ => unreachable!(
                            "Invariant: by constructor checks, JoinType is only Left or Right"
                        ),
                    };
                    DslPlan::HStack {
                        input: Arc::new(DslPlan::Join {
                            input_left,
                            input_right,
                            left_on,
                            right_on,
                            predicates: vec![],
                            options,
                        }),
                        exprs: fill_null.unwrap(),
                        options: ProjectionOptions::default(),
                    }
                }
                None => output,
            })
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

            if is_join {
                ()
            } else if let Some((_, noise, threshold_value, _)) = &threshold_info {
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

fn match_filter(key_sanitizer: &Option<KeySanitizer>) -> Option<(String, u32)> {
    key_sanitizer
        .as_ref()
        .and_then(|s| {
            if let KeySanitizer::Filter(p) = s {
                Some(p.clone())
            } else {
                None
            }
        })
        .and_then(is_threshold_predicate)
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
