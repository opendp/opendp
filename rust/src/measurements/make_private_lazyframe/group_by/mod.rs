use std::collections::HashSet;
use std::sync::Arc;

use crate::accuracy::{
    conservative_discrete_gaussian_tail_to_alpha, conservative_discrete_laplacian_tail_to_alpha,
};
use crate::combinators::{CompositionMeasure, make_composition};
use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::{CategoricalDomain, Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::measurements::expr_noise::NoiseDistribution;
use crate::measurements::make_private_expr;
use crate::measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence};
use crate::metrics::{Bounds, FrameDistance, L0PInfDistance, L01InfDistance};
use crate::traits::{InfAdd, InfMul, InfPowI, InfSub, option_min};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{StableDslPlan, StableExpr};
use dashu::integer::{IBig, UBig};
use dashu::rational::RBig;
use make_private_expr::PrivateExpr;
use matching::find_len_expr;
use polars::prelude::{DslPlan, JoinType, LazyFrame, len};
use polars_plan::dsl::{Expr, col, lit};

#[cfg(test)]
mod test;

mod matching;
pub(crate) use matching::{KeySanitizer, MatchGroupBy, is_threshold_predicate, match_group_by};
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
pub fn make_private_group_by<MI, MO>(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<MI>,
    output_measure: MO,
    plan: DslPlan,
    global_scale: Option<f64>,
    threshold: Option<u32>,
) -> Fallible<Measurement<DslPlanDomain, FrameDistance<MI>, MO, DslPlan>>
where
    MI: 'static + UnboundedMetric,
    MI::EventMetric: UnboundedMetric,
    MO: 'static + ApproximateMeasure,
    Expr: PrivateExpr<L01InfDistance<MI::EventMetric>, MO>
        + StableExpr<L01InfDistance<MI::EventMetric>, L01InfDistance<MI::EventMetric>>,
    DslPlan: StableDslPlan<FrameDistance<MI>, FrameDistance<MI::EventMetric>>,
{
    let is_truncated = input_metric.0.identifier().is_some();

    let Some(MatchGroupBy {
        input,
        group_by,
        aggs,
        mut key_sanitizer,
    }) = match_group_by(plan)?
    else {
        return fallible!(MakeMeasurement, "expected group by");
    };

    // 1: establish stability of `group_by`
    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    for expr in &group_by {
        // grouping keys must be stable
        let t_group_by = expr.clone().make_stable(
            WildExprDomain {
                columns: middle_domain.series_domains.clone(),
                context: Context::RowByRow,
            },
            L0PInfDistance(middle_metric.0.clone()),
        )?;

        let series_domain = &t_group_by.output_domain.column;
        if let Ok(domain) = series_domain.element_domain::<CategoricalDomain>() {
            if domain.categories().is_none() {
                return fallible!(
                    MakeMeasurement,
                    "Categories are data-dependent, which may reveal sensitive record ordering. Cast {} to string before grouping.",
                    series_domain.name
                );
            }
        };
    }

    // 2: prepare for release of `aggs`
    let group_by_id = HashSet::from_iter(group_by.iter().cloned());
    let mut margin = middle_domain.get_margin(&group_by_id);

    let is_join = if let Some(KeySanitizer::Join { keys, .. }) = key_sanitizer.clone() {
        let num_keys = LazyFrame::from((*keys).clone()).select([len()]).collect()?;
        margin.max_groups = Some(num_keys.column("len")?.u32()?.last().unwrap());
        true
    } else {
        false
    };

    let m_expr_aggs = aggs.into_iter().map(|expr| {
        make_private_expr(
            WildExprDomain {
                columns: middle_domain.series_domains.clone(),
                context: Context::Aggregation {
                    margin: margin.clone(),
                },
            },
            L0PInfDistance(middle_metric.0.clone()),
            output_measure.clone(),
            expr,
            global_scale,
        )
    });
    let m_aggs = make_composition(m_expr_aggs.collect::<Fallible<_>>()?)?;

    let f_comp = m_aggs.function.clone();
    let privacy_map = m_aggs.privacy_map.clone();

    // 3: prepare for release of `keys`
    let (dp_exprs, null_exprs): (_, Vec<Option<Expr>>) = m_aggs
        .invoke(&input)?
        .into_iter()
        .map(|plan| (plan.expr, plan.fill))
        .unzip();

    // 3.1: reconcile information about the threshold
    let threshold_info = if margin.invariant.is_some() || is_join {
        None
    } else if let Some((name, threshold_value)) = match_filter(&key_sanitizer) {
        let noise = find_len_expr(&dp_exprs, Some(name.as_str()))?.1;
        Some((name, noise, threshold_value, false))
    } else if let Some(threshold_value) = threshold {
        let (name, noise) = find_len_expr(&dp_exprs, None)?;
        Some((name, noise, threshold_value, true))
    } else {
        return fallible!(
            MakeMeasurement,
            "The key-set of {:?} is private and cannot be released without a filter or join. Please pass a filtering threshold into make_private_lazyframe or conduct a join against a public key-set.",
            group_by_id
        );
    };

    // 3.2: update key sanitizer
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
                        err!(MakeMeasurement, "{} can't be joined with an explicit key set because missing groups cannot be filled", name)
                    })?;

                    Ok(col(name).fill_null(null_expr))
                })
                .collect::<Fallible<_>>()?,
        );
    }

    let function = Function::new_fallible(move |arg: &DslPlan| {
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
    });

    let privacy_map = PrivacyMap::new_fallible(move |d_in: &Bounds| {
        let bound = d_in.get_bound(&group_by_id);

        // incorporate all information into optional bounds
        let l0 = option_min(bound.num_groups, margin.max_groups);
        let li = option_min(bound.per_group, margin.max_length);
        let l1 = d_in.get_bound(&HashSet::new()).per_group;

        // reduce optional bounds to concrete bounds
        let (l0, l1, li) = match (l0, l1, li) {
            (Some(l0), Some(l1), Some(li)) => (l0, l1, li),
            (l0, Some(l1), li) => (l0.unwrap_or(l1), l1, li.unwrap_or(l1)),
            (Some(l0), None, Some(li)) => (l0, l0.inf_mul(&li)?, li),
            _ => {
                let msg = if is_truncated {
                    let mut msg =
                        " This is likely due to a missing truncation earlier in the data pipeline."
                            .to_string();
                    let by_str = group_by_id
                        .iter()
                        .map(|e| format!("{e:?}"))
                        .collect::<Vec<_>>()
                        .join(", ");

                    if l0.is_none() {
                        msg = format!(
                            "{msg} To bound `num_groups` in the Context API, try using `.truncate_num_groups(num_groups, by=[{by_str}])`."
                        );
                    }
                    if li.is_none() {
                        msg = format!(
                            "{msg} To bound `per_group` in the Context API, try using `.truncate_per_group(per_group, by=[{by_str}])`."
                        );
                    }
                    msg
                } else {
                    "".to_string()
                };
                return fallible!(
                    FailedMap,
                    "num_groups ({l0:?}), total contributions ({l1:?}), and per_group ({li:?}) are not sufficiently well-defined.{msg}"
                );
            }
        };

        let mut d_out = privacy_map.eval(&(l0, l1, li))?;

        if margin.invariant.is_some() || is_join {
            ()
        } else if let Some((_, noise, threshold_value, _)) = &threshold_info {
            if li >= *threshold_value {
                return fallible!(FailedMap, "threshold must be greater than {:?}", li);
            }

            let d_instability = threshold_value.neg_inf_sub(&li)?;
            let delta_single =
                integrate_discrete_noise_tail(noise.distribution, noise.scale, d_instability)?;
            let delta_joint = (1.0).inf_sub(
                &(1.0)
                    .neg_inf_sub(&delta_single)?
                    .neg_inf_powi(IBig::from(l0))?,
            )?;
            d_out = MO::add_delta(d_out, delta_joint)?;
        } else {
            return fallible!(
                FailedMap,
                "The key-set is data-dependent and must be protected. Either use a join with a public key-set (`.with_keys(keys)` in the Context API) or use Approximate-DP or Approximate-zCDP."
            );
        }

        Ok(d_out)
    });

    // 4: build final measurement
    t_prior
        >> Measurement::new(
            middle_domain,
            middle_metric,
            output_measure,
            function,
            privacy_map,
        )?
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

pub trait ApproximateMeasure: CompositionMeasure {
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

impl<MO: CompositionMeasure> ApproximateMeasure for Approximate<MO>
where
    Self: CompositionMeasure<Distance = (MO::Distance, f64)>,
{
    fn add_delta((d_out, delta): Self::Distance, delta_p: f64) -> Fallible<Self::Distance> {
        Ok((d_out, delta.inf_add(&delta_p)?))
    }
}

fn integrate_discrete_noise_tail(
    distribution: NoiseDistribution,
    scale: f64,
    tail_bound: u32,
) -> Fallible<f64> {
    let scale = RBig::try_from(scale)?;
    let tail_bound = UBig::from(tail_bound);
    match distribution {
        NoiseDistribution::Laplace => {
            conservative_discrete_laplacian_tail_to_alpha(scale, tail_bound)
        }
        NoiseDistribution::Gaussian => {
            conservative_discrete_gaussian_tail_to_alpha(scale, tail_bound)
        }
    }
}
