use crate::{
    core::{Domain, Measurement, Metric, PrivacyMap},
    error::Fallible,
    measures::{MaxDivergence, ZeroConcentratedDivergence},
    traits::Float, interactive::{IntoDyn, Queryable, FromDyn},
};

#[cfg(feature = "ffi")]
mod ffi;

/// Constructs a new output measurement where the output measure
/// is casted from `MaxDivergence<QO>` to `ZeroConcentratedDivergence<QO>`.
///
/// # Citations
/// - [BS16 Concentrated Differential Privacy: Simplifications, Extensions, and Lower Bounds](https://arxiv.org/pdf/1605.02065.pdf#subsection.3.1)
/// 
/// # Arguments
/// * `meas` - a measurement with a privacy curve to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `DO` - Output Domain
/// * `MI` - Input Metric
/// * `QO` - Output distance type. One of `f32` or `f64`.
pub fn make_pureDP_to_zCDP<DI, DOQ, DOA, MI, QO>(
    meas: Measurement<DI, DOQ, DOA, MI, MaxDivergence<QO>>,
) -> Fallible<Measurement<DI, DOQ, DOA, MI, ZeroConcentratedDivergence<QO>>>
where
    DI: Domain,
    DOQ: Domain,
    DOA: Domain,
    DOA::Carrier: Sized,
    MI: 'static + Metric,
    QO: Float,
    Queryable<DOQ::Carrier, DOA>: IntoDyn + FromDyn
{
    let Measurement {
        input_domain,
        query_domain,
        answer_domain,
        function,
        input_metric,
        privacy_map,
        ..
    } = meas;

    let _2 = QO::one() + QO::one();

    Ok(Measurement::new(
        input_domain,
        query_domain,
        answer_domain,
        function,
        input_metric,
        ZeroConcentratedDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map.eval(d_in).and_then(|eps| eps.inf_pow(&_2)?.inf_div(&_2))
        }),
    ))
}