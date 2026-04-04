use crate::core::{StabilityMap, Transformation};
use crate::domains::{Context, ExprDomain, Invariant, Margin, WildExprDomain};
use crate::metrics::{L0InfDistance, L01InfDistance, LInfDistance};
use crate::polars::{OpenDPPlugin, apply_plugin, literal_value_of, match_plugin};
use crate::traits::{InfCast, Number};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{
    StableExpr, check_candidates, score_candidates_constants, score_candidates_map,
};
use crate::{core::Function, error::Fallible};

use polars::datatypes::{
    Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type, StaticArray, UInt32Type,
    UInt64Type,
};
use polars::lazy::dsl::Expr;
use polars::prelude::DataType::{self, *};

mod plugin_dq_score;
pub(crate) use plugin_dq_score::{DiscreteQuantileScorePlugin, DiscreteQuantileScoreShim};
use polars::prelude::{AnyValue, LiteralValue, NamedFrom, PolarsPhysicalType, Scalar};
use polars::series::Series;

#[cfg(test)]
pub mod test;

/// Make a measurement that adds Laplace noise to a Polars expression.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the Laplace noise will be added
pub fn make_expr_discrete_quantile_score<MI>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    expr: Expr,
) -> Fallible<
    Transformation<
        WildExprDomain,
        L01InfDistance<MI>,
        ExprDomain,
        L0InfDistance<LInfDistance<f64>>,
    >,
>
where
    MI: 'static + UnboundedMetric,
    Expr: StableExpr<L01InfDistance<MI>, L01InfDistance<MI>>,
{
    let Some((input, alpha, candidates)) = match_discrete_quantile_score(&expr)? else {
        return fallible!(
            MakeTransformation,
            "Expected {} function",
            DiscreteQuantileScoreShim::NAME
        );
    };

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let active_series = &middle_domain.column;
    if active_series.nullable {
        return fallible!(
            MakeTransformation,
            "Quantile estimation requires non-null inputs. Try using `.fill_null(x)` or `.drop_null()` first."
        );
    }
    let nan = match active_series.dtype() {
        DataType::Float32 => active_series.atom_domain::<f32>()?.nan(),
        DataType::Float64 => active_series.atom_domain::<f64>()?.nan(),
        _ => false,
    };
    if nan {
        return fallible!(
            MakeTransformation,
            "Quantile estimation requires non-nan inputs. Try using `.fill_nan(x)` or `.drop_nan()` first."
        );
    }

    let candidates = candidates.strict_cast(&active_series.dtype())?;

    match active_series.dtype() {
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
            );
        }
        dtype => {
            return fallible!(
                MakeTransformation,
                "Expected numeric data type, found {:?}",
                dtype
            );
        }
    }?;

    let margin = middle_domain.context.aggregation("quantile")?;

    let mgl = margin
        .max_length
        .ok_or_else(|| err!(MakeTransformation, "Must know max_length"))?;

    // alpha = alpha_num / alpha_den (numerator and denominator of alpha)
    let (alpha_num, alpha_den, size_limit) = score_candidates_constants(Some(mgl as u64), alpha)?;

    let len = candidates.len();
    let fill_value = Expr::Literal(LiteralValue::Scalar(Scalar::new(
        DataType::Array(Box::new(DataType::UInt64), len),
        AnyValue::Array(Series::new("".into(), &vec![0u64; len]), len),
    )));

    let mut output_domain = active_series.clone();
    output_domain.set_dtype(DataType::Array(
        Box::new(DataType::UInt64),
        candidates.len(),
    ))?;

    let output_domain = ExprDomain {
        column: output_domain,
        context: Context::Aggregation {
            margin: Margin {
                max_length: Some(1),
                ..margin
            },
        },
    };

    t_prior
        >> Transformation::<_, L01InfDistance<MI>, _, L0InfDistance<LInfDistance<_>>>::new(
            middle_domain,
            middle_metric,
            output_domain,
            L0InfDistance(LInfDistance::new(false)),
            Function::then_expr(move |input_expr| {
                apply_plugin(
                    vec![input_expr],
                    expr.clone(),
                    DiscreteQuantileScorePlugin {
                        alpha: (alpha_num, alpha_den),
                        candidates: candidates.clone(),
                        size_limit,
                    },
                )
            })
            .fill_with(fill_value),
            StabilityMap::new_fallible(move |(l0, _, li)| {
                let out_li = f64::inf_cast(score_candidates_map(
                    alpha_num,
                    alpha_den,
                    margin.invariant == Some(Invariant::Lengths),
                )(li)?)?;
                Ok((*l0, out_li))
            }),
        )?
}

fn validate<T: 'static + PolarsPhysicalType>(candidates: &Series) -> Fallible<()>
where
    for<'a> T::Physical<'a>: Number,
{
    if candidates.null_count() > 0 {
        return fallible!(
            MakeTransformation,
            "Candidates must not contain null values, found {} null value(s)",
            candidates.null_count()
        );
    }
    check_candidates(&series_to_vec::<T>(
        &candidates.cast(&T::get_static_dtype())?,
    )?)
}

fn series_to_vec<'a, T: 'static + PolarsPhysicalType>(
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
pub(crate) fn match_discrete_quantile_score(expr: &Expr) -> Fallible<Option<(&Expr, f64, Series)>> {
    let Some(input) = match_plugin::<DiscreteQuantileScoreShim>(expr)? else {
        return Ok(None);
    };

    let Ok([data, alpha, candidates]) = <&[_; 3]>::try_from(input.as_slice()) else {
        return fallible!(
            MakeMeasurement,
            "{:?} expects three inputs: data, alpha and candidates",
            DiscreteQuantileScoreShim::NAME
        );
    };

    let alpha = literal_value_of::<f64>(alpha)?
        .ok_or_else(|| err!(MakeTransformation, "alpha must be known"))?;
    let candidates = literal_value_of::<Series>(candidates)?
        .ok_or_else(|| err!(MakeTransformation, "candidates must be known"))?;

    Ok(Some((data, alpha, candidates)))
}
