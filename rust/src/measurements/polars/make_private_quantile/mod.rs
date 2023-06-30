use opendp_derive::bootstrap;
use polars::{
    lazy::dsl::{Expr, GetOutput},
    prelude::{ChunkedArray, NamedFromOwned, UInt32Type},
    series::Series,
};

use crate::{
    core::{Function, Measurement, MetricSpace},
    domains::{AtomDomain, ExprDomain, NumericDataType, VectorDomain},
    error::Fallible,
    measurements::Optimize,
    measures::MaxDivergence,
    traits::{
        samplers::SampleUniform, DistanceConstant, ExactIntCast, Float, InfCast, Number, RoundCast,
    },
    transformations::{
        make_quantile_score_candidates_expr, traits::UnboundedMetric, DQuantileOuterMetric, ToVec,
    },
};

#[cfg(feature = "ffi")]
mod ffi;

use super::then_report_noisy_max_gumbel_expr;

#[bootstrap(
    features("contrib"),
    arguments(temperature(c_type = "void *")),
    generics(
        MI(suppress),
        TIA(suppress),
        QO(default = "float"),
        A(default = "float")
    ),
    derived_types(TIA = "$get_active_column_type(input_domain)")
)]
/// Makes a Measurement to compute DP quantiles with Polars.
/// Based on a list of candidates, first compute scores, then use exponential mechanism
/// to get the maximum index of the quantile.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `candidates` - Potential quantile to score
/// * `temperature` - Higher temperatures are more private.
/// * `alpha` - Quantile value in [0, 1]. Choose 0.5 for median
///
/// # Generics
/// * `MI` - Input Metric.
/// * `TIA` - Atomic Input Type. Type of elements in the candidates input vector.
/// * `QO` - Output Distance Type.
pub fn make_private_quantile_expr<MI, TIA, QO>(
    input_domain: ExprDomain<MI::LazyDomain>,
    input_metric: MI,
    candidates: Vec<TIA>,
    temperature: QO,
    alpha: f64,
) -> Fallible<Measurement<ExprDomain<MI::LazyDomain>, Expr, MI, MaxDivergence<QO>>>
where
    MI: DQuantileOuterMetric,
    MI::InnerMetric: 'static + UnboundedMetric,
    TIA: Number + NumericDataType,
    QO: InfCast<u64> + DistanceConstant<MI::Distance> + Float + SampleUniform + RoundCast<u64>,

    (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
    (ExprDomain<MI::LazyDomain>, MI::ScoreMetric): MetricSpace,
    (VectorDomain<AtomDomain<TIA>>, MI::InnerMetric): MetricSpace,

    Series: NamedFromOwned<Vec<TIA>>,
    ChunkedArray<TIA::Polars>: ToVec<TIA>,
{
    make_quantile_score_candidates_expr(input_domain, input_metric, candidates.clone(), alpha)?
        >> then_report_noisy_max_gumbel_expr::<MI::ScoreMetric, _, _>(temperature, Optimize::Min)
        >> Function::new(move |expr: &Expr| {
            expr.clone().map(
                enclose!(candidates, move |s: Series| {
                    let vec = s
                        .unpack::<UInt32Type>()?
                        .into_no_null_iter()
                        .map(|index| {
                            candidates
                                .get(usize::exact_int_cast(index)?)
                                .cloned()
                                .ok_or_else(|| err!(FailedFunction, "Index out of bounds"))
                        })
                        .collect::<Fallible<Vec<TIA>>>()?;

                    Ok(Some(Series::from_vec(&s.name(), vec)))
                }),
                GetOutput::from_type(TIA::dtype()),
            )
        })
}

#[cfg(test)]
mod test_make_discrete_quantile {
    use polars::prelude::*;

    use crate::{
        metrics::{InsertDeleteDistance, Lp},
        transformations::polars_test::{get_grouped_test_data, get_select_test_data},
    };

    use super::*;

    #[test]
    fn test_discrete_dp_quantile_select() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_select_test_data()?;

        // Get resulting scores (expression result)
        let candidates = vec![2.0, 4.0, 5.0];

        // Get resulting index (expression result)
        let meas =
            make_private_quantile_expr(expr_domain, InsertDeleteDistance, candidates, 1., 0.8)?;
        let expr_meas = meas.invoke(&(lazy_frame.clone(), col("B")))?;
        let release = (*lazy_frame).clone().select([expr_meas]).collect()?;

        println!("{:?}", release);
        Ok(())
    }

    #[test]
    fn test_discrete_dp_quantile_groupby() -> Fallible<()> {
        let (expr_domain, group_by) = get_grouped_test_data()?;

        // Get resulting scores (expression result)
        let candidates = vec![2.0, 4.0, 7.0];

        // Get resulting index (expression result)
        let meas =
            make_private_quantile_expr(expr_domain, Lp(InsertDeleteDistance), candidates, 1., 0.5)?;
        let expr_meas = meas.invoke(&(group_by.clone(), col("B")))?;
        let release = (*group_by).clone().agg([expr_meas]).collect()?;

        println!("{:?}", release);
        Ok(())
    }
}
