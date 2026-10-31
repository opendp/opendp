use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{MaxDivergence, ZeroConcentratedDivergence},
    traits::{InfDiv, InfPowI},
};

#[cfg(feature = "ffi")]
mod ffi;

/// Constructs a new output measurement where the output measure
/// is casted from `MaxDivergence` to `ZeroConcentratedDivergence`.
///
/// # Citations
/// - [BS16 Concentrated Differential Privacy: Simplifications, Extensions, and Lower Bounds](https://arxiv.org/pdf/1605.02065.pdf#subsection.3.1)
///
/// # Arguments
/// * `meas` - a measurement with a privacy curve to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `MI` - Input Metric
/// * `TO` - Output Type
pub fn make_pureDP_to_zCDP<DI, MI, TO>(
    m: Measurement<DI, MI, MaxDivergence, TO>,
) -> Fallible<Measurement<DI, MI, ZeroConcentratedDivergence, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map = m.privacy_map.clone();
    m.with_map(
        m.input_metric.clone(),
        ZeroConcentratedDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let eps = privacy_map.eval(d_in)?;
            // Genuine unbounded privacy loss stays infinite; this is not a
            // finite-result overflow in the legacy finite-only arithmetic API.
            if eps == f64::INFINITY {
                return Ok(f64::INFINITY);
            }
            eps.inf_powi(2.into())?.inf_div(&2.0)
        }),
    )
}

#[cfg(test)]
#[test]
fn test_unbounded_privacy_loss() -> Fallible<()> {
    use crate::{domains::AtomDomain, measurements::make_laplace, metrics::AbsoluteDistance};
    let m = make_laplace(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        0.0,
        None,
    )?;
    let m = make_pureDP_to_zCDP(m)?;
    assert_eq!(m.map(&0.0)?, 0.0);
    assert_eq!(m.map(&1.0)?, f64::INFINITY);
    Ok(())
}
