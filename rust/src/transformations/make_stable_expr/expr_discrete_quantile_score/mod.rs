use crate::core::{
    apply_plugin, match_plugin, ExprFunction, MetricSpace, StabilityMap, Transformation,
};
use crate::domains::MarginPub;
use crate::metrics::{LInfDistance, Parallel, PartitionDistance};
use crate::traits::{InfCast, Number, RoundCast};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{
    score_candidates_constants, score_candidates_map, validate_candidates, StableExpr,
};
use crate::{core::Function, domains::ExprDomain, error::Fallible};

use polars::lazy::dsl::Expr;
use polars::prelude::DataType::*;

mod plugin_dq_score;
pub(crate) use plugin_dq_score::DQScoreArgs;

#[cfg(test)]
pub mod test;

static DQ_SCORE_PLUGIN_NAME: &str = "dq_score";

/// Make a measurement that adds Laplace noise to a Polars expression.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the Laplace noise will be added
pub fn make_expr_discrete_quantile_score<MI>(
    input_domain: ExprDomain,
    input_metric: PartitionDistance<MI>,
    expr: Expr,
) -> Fallible<
    Transformation<ExprDomain, ExprDomain, PartitionDistance<MI>, Parallel<LInfDistance<f64>>>,
>
where
    MI: 'static + UnboundedMetric,
    Expr: StableExpr<PartitionDistance<MI>, PartitionDistance<MI>>,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
{
    let Some((input, args)) = match_dq_score(&expr)? else {
        return fallible!(MakeTransformation, "Expected quantile_score function");
    };

    let DQScoreArgs {
        alpha, candidates, ..
    } = args;

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let active_series = middle_domain.active_series()?.clone();
    if active_series.nullable {
        return fallible!(
            MakeTransformation,
            "Quantile estimation requires non-null inputs"
        );
    }

    fn validate<T: Number + RoundCast<f64>>(candidates: &Vec<f64>) -> Fallible<()> {
        validate_candidates(
            &candidates
                .iter()
                .cloned()
                .map(T::round_cast)
                .collect::<Fallible<_>>()?,
        )
    }

    match active_series.field.dtype {
        UInt32 => validate::<u32>(&candidates),
        UInt64 => validate::<u64>(&candidates),
        Int8 => validate::<i8>(&candidates),
        Int16 => validate::<i16>(&candidates),
        Int32 => validate::<i32>(&candidates),
        Int64 => validate::<i64>(&candidates),
        Float32 => validate::<f32>(&candidates),
        Float64 => validate::<f64>(&candidates),
        UInt8 | UInt16 => {
            return fallible!(
                FailedFunction,
                "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64."
            )
        }
        dtype => {
            return fallible!(
                MakeTransformation,
                "Expected numeric data type, found {:?}",
                dtype
            );
        }
    }?;

    let margin = middle_domain.active_margin()?.clone();

    let mpl = margin
        .max_partition_length
        .ok_or_else(|| err!(MakeTransformation, "Must know max_partition_length"))?;

    let constants = score_candidates_constants::<u64>(Some(mpl as u64), alpha)?;
    // alpha = alpha_num / alpha_den (numerator and denominator of alpha)
    let (alpha_num, alpha_den, _) = constants.clone();

    t_prior
        >> Transformation::<_, _, PartitionDistance<MI>, Parallel<LInfDistance<_>>>::new(
            middle_domain.clone(),
            middle_domain,
            Function::then_expr(move |input_expr| {
                apply_plugin(
                    input_expr,
                    expr.clone(),
                    DQScoreArgs {
                        alpha,
                        candidates: candidates.clone(),
                        constants: Some(constants),
                    },
                )
            }),
            middle_metric,
            Parallel(LInfDistance::new(false)),
            StabilityMap::new_fallible(move |(l0, _, li)| {
                let out_li = f64::inf_cast(score_candidates_map(
                    alpha_num,
                    alpha_den,
                    margin.public_info == Some(MarginPub::Lengths),
                )(li)?)?;
                Ok((*l0, out_li))
            }),
        )?
}

/// Determine if the given expression is a discrete quantile score expression.
///
/// # Arguments
/// * `expr` - The expression to check
///
/// # Returns
/// If matched, ipnut expression and discrete quantile score arguments
pub(crate) fn match_dq_score(expr: &Expr) -> Fallible<Option<(&Expr, DQScoreArgs)>> {
    let Some((score_input, args)) = match_plugin(expr, DQ_SCORE_PLUGIN_NAME)? else {
        return Ok(None);
    };

    let [score_input] = <&[_; 1]>::try_from(score_input.as_slice()).map_err(|_| {
        err!(
            MakeTransformation,
            "{} expects a single input expression",
            DQ_SCORE_PLUGIN_NAME
        )
    })?;

    Ok(Some((score_input, args)))
}
