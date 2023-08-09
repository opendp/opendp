use opendp_derive::bootstrap;
use std::f64::consts::LOG2_E;
use polars::{
    lazy::dsl::Expr,
    prelude::{ChunkedArray, NamedFromOwned, LazyFrame},
    series::Series,
};

use crate::{
    core::{Function, Measurement, MetricSpace, ExprFunction},
    domains::{AtomDomain, ExprDomain, NumericDataType, VectorDomain, LazyFrameDomain},
    error::Fallible,
    measures::MaxDivergence,
    traits::{DistanceConstant, Float, InfCast, Number, InfSub, InfDiv},
    transformations::{ARDatasetMetric, DQuantileOuterMetric, ToVec, IntoFrac},
};

// #[cfg(feature = "ffi")]
// mod ffi;

use super::make_private_quantile;

// #[bootstrap(
//     // features("contrib"), 
//     // arguments(temperature(c_type = "void *")),
//     // generics(MI(suppress), TIA(suppress), QO(default = "float"), A(default="float")),
//     // derived_types(TIA = "$get_active_column_type(input_domain)")
// )]
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
/// * `A` - Alpha type. Can be a (numer, denom) tuple, or float.
pub fn make_multi_private_quantiles<MI, TIA, QO, A>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    candidates: Vec<TIA>,
    temperature: QO,
    alphas: Vec<A>,
    column_name: String,
) -> Fallible<Measurement<ExprDomain<MI::LazyDomain>, Expr, MI, MaxDivergence<QO>>>
where
    MI: DQuantileOuterMetric,
    MI::InnerMetric: 'static + ARDatasetMetric,
    TIA: Number + NumericDataType,
    QO: InfCast<u64> + DistanceConstant<MI::Distance> + Float,
    A: Clone + IntoFrac,

    (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
    (ExprDomain<MI::LazyDomain>, MI::ScoreMetric): MetricSpace,
    (VectorDomain<AtomDomain<TIA>>, MI::InnerMetric): MetricSpace,

    Series: NamedFromOwned<Vec<TIA>>,
    ChunkedArray<TIA::Polars>: ToVec<TIA>,
{
    let nb_alphas = alphas.len();
    if nb_alphas == 0 {
        panic!("No alphas provided")
    }

    // For output measure and stability map
    let quantile_measurement = make_private_quantile(
        input_domain, input_metric, candidates, temperature, alphas.0,
    )?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |lf: &LazyFrame| -> Fallible<Vec<TIA>> {
            let shared_temperature = temperature.clone()/QO::inf_cast(LOG2_E * (nb_alphas as f64) + 1.0)?;

            impl_function(input_domain, input_metric, lf.clone(), candidates, alphas, shared_temperature)
        }),
        input_metric,
        quantile_measurement.output_measure,
        quantile_measurement.privacy_map,
    )
}

// alphas: [0.25, 0.5, 0.75]

// alphas: [0.25, 0.5, 0.75]
// candidates: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

// round 1:
//     alphas: [0.5]
//     candidates: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
//     quantiles: []
//   -> [3]

// round 2:
//     alphas: [0.25, 0.75]
//     candidates: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
//     quantiles: [3]
//   -> [2, 7]

fn release_quantile_layer<MI, TIA, QO>(
    data: &LazyFrame,
    input_domain: LazyFrameDomain,
    input_metric: MI,
    candidates: Vec<TIA>,
    temperature: QO,
    alphas: Vec<f64>,
    quantiles: Vec<TIA>,
    column_name: &str
) -> Fallible<Vec<TIA>> {
    // https://github.com/pola-rs/polars/blob/e4dd665d11659d6e114aad630cd2e49265e5a8ba/polars/polars-lazy/polars-plan/src/dsl/mod.rs#L1466-L1480

    let breaks = quantiles.iter().map(|q| f64::round_cast(q)).collect::<Fallible<Vec<_>>>()?;
    let cut_frame = data.clone().with_column(col(column_name).cut(breaks, None, false, false));

    alphas.iter().enumerate().map(|(partition_id, alpha)| {
        let partition_candidates = candidates.iter()
            .filter(|c| c < breaks[partition_id] && c >= breaks[partition_id - 1])
            .cloned()
            .collect::<Vec<_>>();
        let quantile_meas = ((input_domain, input_metric) >> then_private_select([
            then_col(column_name) >> then_private_quantile(partition_candidates, temperature, alphas[partition_id])
        ]))?;

        let lf = quantile_meas.invoke(&cut_frame.clone().filter(col(column_name).eq(partition_id)));

        item::<TIA>(lf.collect()?)
    }).collect()
}

fn impl_function<MI, TIA, QO, A>(
    input_domain: ExprDomain<MI::LazyDomain>,
    input_metric: MI,
    lf: LazyFrame,
    candidates: Vec<TIA>,
    mut alphas: Vec<A>,
    temperature: QO
) -> Fallible<Vec<TIA>>
where
    MI: DQuantileOuterMetric,
    MI::InnerMetric: 'static + ARDatasetMetric,
    TIA: Number + NumericDataType,
    QO: InfCast<u64> + DistanceConstant<MI::Distance> + Float,
    A: Clone + IntoFrac + InfCast<f64> + InfSub + InfDiv,

    (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
    (ExprDomain<MI::LazyDomain>, MI::ScoreMetric): MetricSpace,
    (VectorDomain<AtomDomain<TIA>>, MI::InnerMetric): MetricSpace,

    Series: NamedFromOwned<Vec<TIA>>,
    ChunkedArray<TIA::Polars>: ToVec<TIA>,
{
    

    // base cases
    let nb_alphas = alphas.len();
    if nb_alphas == 0 {
        return Ok(vec![]);
    }
    if nb_alphas == 1 {
        let meas = 
            make_private_quantile(input_domain, input_metric, candidates, temperature, alphas.0)?;
        return meas; // TODO TIA res from meas
    }

    // estimate mid quantile
    let mid = (nb_alphas + 1)/ 2;
    let p = alphas[mid];
    let mid_quantile: TIA = 
        make_private_quantile(input_domain, input_metric, candidates, temperature, p)?; // TODO TIA res from meas

    // Split alphas and candidates apart
    let mut alphas_l: Vec<A> = Vec::new();
    let mut alphas_u: Vec<A> = Vec::new();
    for &alpha in &alphas[..mid] {
        alphas_l.push(alpha.inf_div(&p)?);
    }

    for &alpha in &alphas[mid + 1..] {
        alphas_u.push((alpha.inf_sub(&p))?.inf_div(&A::inf_cast(1.0)?.inf_sub(&p)?)?);
    }

    let mut candidates_l: Vec<TIA> = Vec::new();
    let mut candidates_u: Vec<TIA> = Vec::new();

    for &candidate in &candidates {
        if candidate > mid_quantile {
            candidates_u.push(candidate);
        } else {
            candidates_l.push(candidate);
        }
    }
    
    // recurse down left and right partitions
    return [
        *impl_function(input_domain, input_metric, lf, candidates_u, alphas_l, temperature)?,
        mid_quantile,
        *impl_function(input_domain, input_metric, lf, candidates_l, alphas_u, temperature)?
    ]
}


#[cfg(test)]
mod test_make_multi_discrete_quantile {
    use polars::prelude::*;

    use crate::{
        metrics::{InsertDeleteDistance, Lp},
        transformations::polars_test::{get_grouped_test_data, get_select_test_data},
    };

    use super::*;

    #[test]
    fn test_discrete_dp_multi_quantiles_select() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_select_test_data()?;

        // Get resulting scores (expression result)
        let candidates = vec![2.0, 4.0, 5.0];

        // Get resulting index (expression result)
        let meas = make_multi_private_quantiles(expr_domain, InsertDeleteDistance, candidates, 1., vec![0.25, 0.5, 0.75])?;
        let expr_meas = meas.invoke(&(lazy_frame.clone(), col("B")))?;
        let release = (*lazy_frame).clone().select([expr_meas]).collect()?;

        println!("{:?}", release);
        Ok(())
    }

    #[test]
    fn test_discrete_dp_mutli_quantiles_groupby() -> Fallible<()> {
        let (expr_domain, group_by) = get_grouped_test_data()?;

        // Get resulting scores (expression result)
        let candidates = vec![2.0, 4.0, 7.0];

        // Get resulting index (expression result)
        let meas =
        make_multi_private_quantiles(expr_domain, Lp(InsertDeleteDistance), candidates, 1., vec![0.25, 0.5, 0.75])?;
        let expr_meas = meas.invoke(&(group_by.clone(), col("B")))?;
        let release = (*group_by).clone().agg([expr_meas]).collect()?;

        println!("{:?}", release);
        Ok(())
    }
}
