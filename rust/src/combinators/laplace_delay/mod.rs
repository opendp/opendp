use std::{thread::sleep, time::Duration};

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace},
    error::Fallible,
    traits::samplers::sample_discrete_laplace_linear,
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
    shift: u64,
    scale: f64,
) -> Fallible<Measurement<DI, TO, MI, MO>>
where
    DI: Domain,
    DI::Carrier: 'static,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    if m.time_safe {
        return fallible!(MakeTransformation, "Measurement is already time-safe");
    }
    let function = m.function.clone();

    Measurement::new(
        m.input_domain.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            let output = function.eval(arg)?;
            let delay = sample_discrete_laplace_linear(shift, scale, Some((0, shift * 2)))?;
            sleep(Duration::from_secs(delay));

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
    )
    .map(|m| m.with_time_safe(true))
}
