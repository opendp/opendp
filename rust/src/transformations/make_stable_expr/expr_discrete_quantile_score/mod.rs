use crate::core::{
    apply_plugin, match_plugin, ExprFunction, MetricSpace, StabilityMap, Transformation,
};
use crate::domains::MarginPub;
use crate::measurements::expr_index_candidates::Candidates;
use crate::metrics::{LInfDistance, Parallel, PartitionDistance};
use crate::traits::{InfCast, Number};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{
    score_candidates_constants, score_candidates_map, validate_candidates, StableExpr,
};
use crate::{core::Function, domains::ExprDomain, error::Fallible};

use polars::datatypes::{
    Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type, PolarsDataType,
    StaticArray, UInt32Type, UInt64Type,
};
use polars::lazy::dsl::Expr;
use polars::prelude::DataType::*;

mod plugin_dq_score;
pub(crate) use plugin_dq_score::DiscreteQuantileScoreArgs;
use polars::series::Series;

#[cfg(test)]
pub mod test;

static DQ_SCORE_PLUGIN_NAME: &str = "discrete_quantile_score";

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
    let Some((input, args)) = match_discrete_quantile_score(&expr)? else {
        return fallible!(
            MakeTransformation,
            "Expected {} function",
            DQ_SCORE_PLUGIN_NAME
        );
    };

    let DiscreteQuantileScoreArgs {
        alpha,
        candidates: Candidates(candidates),
        ..
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

    match active_series.field.dtype {
        UInt32 => validate::<UInt32Type>(&candidates),
        UInt64 => validate::<UInt64Type>(&candidates),
        Int8 => validate::<Int8Type>(&candidates),
        Int16 => validate::<Int16Type>(&candidates),
        Int32 => validate::<Int32Type>(&candidates),
        Int64 => validate::<Int64Type>(&candidates),
        Float32 => validate::<Float32Type>(&candidates),
        Float64 => validate::<Float64Type>(&candidates),
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
                    DiscreteQuantileScoreArgs {
                        alpha,
                        candidates: Candidates(candidates.clone()),
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

fn validate<T: 'static + PolarsDataType>(candidates: &Series) -> Fallible<()>
where
    for<'a> T::Physical<'a>: Number,
{
    if candidates.null_count() > 0 {
        return fallible!(
            MakeTransformation,
            "Candidates must not contain null values"
        );
    }
    validate_candidates(&series_to_vec::<T>(&candidates.cast(&T::get_dtype())?)?)
}

fn series_to_vec<'a, T: 'static + PolarsDataType>(
    series: &'a Series,
) -> Fallible<Vec<T::Physical<'a>>>
where
    T::Array: StaticArray,
{
    Ok(series
        .unpack::<T>()?
        .downcast_iter()
        .flat_map(StaticArray::values_iter)
        .collect::<Vec<_>>())
}

/// Determine if the given expression is a discrete quantile score expression.
///
/// # Arguments
/// * `expr` - The expression to check
///
/// # Returns
/// If matched, ipnut expression and discrete quantile score arguments
pub(crate) fn match_discrete_quantile_score(
    expr: &Expr,
) -> Fallible<Option<(&Expr, DiscreteQuantileScoreArgs)>> {
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
