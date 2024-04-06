use crate::core::{
    apply_plugin, match_plugin, ExprFunction, MetricSpace, Scalar, StabilityMap, Transformation,
};
use crate::domains::MarginPub;
use crate::metrics::{LInfDistance, Parallel, PartitionDistance};
use crate::traits::{Number, RoundCast};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{
    score_candidates_constants, score_candidates_map, validate_candidates, StableExpr,
};
use crate::{core::Function, domains::ExprDomain, error::Fallible};

use polars::lazy::dsl::Expr;
use polars::prelude::DataType::*;

mod plugin_dq_score;
pub(crate) use plugin_dq_score::DQScoreArgs;

/// Make a measurement that adds Laplace noise to a Polars expression.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the Laplace noise will be added
pub fn make_expr_discrete_quantile_score<MI: 'static + UnboundedMetric>(
    input_domain: ExprDomain,
    input_metric: PartitionDistance<MI>,
    expr: Expr,
) -> Fallible<
    Transformation<ExprDomain, ExprDomain, PartitionDistance<MI>, Parallel<LInfDistance<Scalar>>>,
>
where
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
    let (alpha_num, alpha_den, _) = constants.clone();

    t_prior
        >> Transformation::<_, _, PartitionDistance<MI>, Parallel<LInfDistance<_>>>::new(
            middle_domain.clone(),
            middle_domain,
            Function::new_expr(move |input_expr| {
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
                let out_li = Scalar::from(score_candidates_map(
                    alpha_num,
                    alpha_den,
                    margin.public_info == Some(MarginPub::Lengths),
                )(li)?);
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
    let Some((score_input, args)) = match_plugin(expr, "dq_score")? else {
        return Ok(None);
    };

    let [score_input] = <&[_; 1]>::try_from(score_input.as_slice()).map_err(|_| {
        err!(
            MakeTransformation,
            "dq_score expects a single input expression"
        )
    })?;

    Ok(Some((score_input, args)))
}

#[cfg(test)]
pub mod test_expr_quantile {
    use super::*;
    use polars::prelude::*;

    use crate::{
        core::PrivacyNamespaceHelper,
        domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
        metrics::SymmetricDistance,
    };

    pub fn get_quantile_test_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
        let pub_key_margin = Margin::new()
            .with_max_partition_length(1000)
            .with_public_keys();

        let lf_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
        ])?
        .with_margin::<&str>(&[], pub_key_margin.clone())?
        .with_margin(&["A"], pub_key_margin)?;

        let a = (0..1010).map(|i| (i % 101) as f64).collect::<Vec<_>>();
        let b = (0..1010).map(|i| (i % 10)).collect::<Vec<_>>();
        let lf = df!("A" => a, "B" => b)?.lazy();

        Ok((lf_domain, lf))
    }

    #[test]
    fn test_expr_discrete_quantile_score() -> Fallible<()> {
        let (lf_domain, lf) = get_quantile_test_data()?;
        let expr_domain = lf_domain.select();
        let candidates = vec![0., 10., 20., 30., 40., 50., 60., 70., 80., 90., 100.];

        let m_quant: Transformation<_, _, _, Parallel<LInfDistance<Scalar>>> = col("A")
            .dp()
            .quantile_score(candidates, 0.5)
            .make_stable(expr_domain, PartitionDistance(SymmetricDistance))?;

        let dp_expr = m_quant.invoke(&(lf.logical_plan.clone(), all()))?.1;

        let df_actual = lf.clone().select([dp_expr]).collect()?;
        let AnyValue::Array(series, _) = df_actual.column("A")?.get(0)? else {
            panic!("Expected an array");
        };

        let actual: Vec<u64> = series
            .u64()?
            .downcast_iter()
            .flat_map(StaticArray::values_iter)
            .collect();

        let expected = vec![1000, 800, 600, 400, 200, 0, 200, 400, 600, 800, 1000];
        assert_eq!(actual, expected);

        Ok(())
    }
}
