use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap, Function},
    error::Fallible,
    measures::{FixedSmoothedMaxDivergence, SmoothedMaxDivergence},
    traits::samplers::SampleDiscreteLaplaceZ2k,
};

/// Make a non-time-safe measurement time-safe by adding a delay.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy curve to be fixed
/// * `delta` - parameter to fix the privacy curve with
///
/// # Generics
/// * `DI` - Input Domain
/// * `TO` - Output Type
/// * `MI` - Input Metric.
/// * `MO` - Output Measure of the input argument. Must be `SmoothedMaxDivergence<Q>`
pub fn make_laplace_delay<DI, TO, MI, MO>(
    m: &Measurement<DI, TO, MI, MO>,
    shift: f64,
    scale: f64,
) -> Fallible<Measurement<DI, TO, MI, MO>>
where
    DI: Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    if m.time_safe {
        return fallible!(MakeTransformation, "Measurement is already time-safe");
    }

    Measurement::new(
        m.input_domain,
        Function::new_fallible(move |arg: &DI::Carrier| {
            let output = m.function.eval(arg)?;
            let delay = f64::sample_discrete_laplace_Z2k(shift, scale, 1074)?;
            sleep(delay);

            Ok(output)
        }),
        m.input_metric.clone(),
        m.output_measure.clone(),
        // TODO: this should add a delta into the map (based on shift/scale?). 
        // Will need a new trait on MO for this.
        // Maybe also refactor the measures:
        //     SmoothedMaxDivergence<Q> -> Smoothed<MaxDivergence<Q>>
        // so as to also support, for example:
        //     Smoothed<ZeroConcentratedDivergence<Q>>, with distance (ρ, δ)?
        // Then the output measure is of type Smoothed<MO::InnerMeasure>
        m.privacy_map.clone(),
    ).map(|m| m.with_time_safe(true))
}
