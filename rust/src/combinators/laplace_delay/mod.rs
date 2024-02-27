use std::{thread::sleep, time::Duration};

use dashu::{
    base::Sign,
    float::{round::mode::Up, FBig},
    integer::IBig,
    rational::RBig,
};

use crate::{
    core::{Domain, Function, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{FixedSmoothedMaxDivergence, MaxDivergence},
    traits::samplers::sample_discrete_laplace,
};

use super::BasicCompositionMeasure;


#[cfg(test)]
mod test;

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
pub fn make_laplace_delay<DI, TO, MI>(
    m: &Measurement<DI, TO, MI, MaxDivergence<f64>>,
    shift: u64,
    scale: f64,
) -> Fallible<Measurement<DI, TO, MI, FixedSmoothedMaxDivergence<f64>>>
where
    DI: Domain,
    DI::Carrier: 'static,
    TO: 'static,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    if m.time_safe {
        return fallible!(MakeTransformation, "Measurement is already time-safe");
    }

    let i_shift = IBig::from(shift);
    let f_shift = FBig::from(shift);
    let r_scale = RBig::try_from(scale)?;
    let f_scale = FBig::try_from(scale)?;
    let function = m.function.clone();
    let privacy_map = m.privacy_map.clone();
    let output_metric = FixedSmoothedMaxDivergence::default();

    Measurement::new(
        m.input_domain.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            let output = function.eval(arg)?;
            let delay = i_shift.clone() + sample_discrete_laplace(r_scale.clone())?;
            let (sign, mag) = delay.into_parts();
            match sign {
                Sign::Positive => sleep(Duration::from_millis(
                    u64::try_from(mag).unwrap_or(u64::MAX),
                )),
                Sign::Negative => (),
            };

            Ok(output)
        }),
        m.input_metric.clone(),
        output_metric.clone(),
        PrivacyMap::new_fallible(move |d_in| {
            let (eps_0, del_0) = (privacy_map.eval(d_in)?, 0.);

            let Some(ref time_map) = privacy_map.time else {
                return fallible!(FailedMap, "PrivacyMap does not have a time component");
            };

            let t_in = time_map(d_in)?;
            if shift < t_in {
                return fallible!(FailedFunction, "Shift is less than the input time");
            }

            let t_in = FBig::<Up>::from(t_in);
            let f_eps_t = &t_in / &f_scale;
            let f_del_t = (-&f_eps_t * (&f_shift - &t_in) / t_in).exp();
            let (eps_t, del_t) = (f_eps_t.to_f64().value(), f_del_t.to_f64().value());

            output_metric.compose(vec![(eps_0, del_0), (eps_t, del_t)])
        }),
    )
    .map(|m| m.with_time_safe(true))
}
