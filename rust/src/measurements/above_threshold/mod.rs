use dashu::{base::ConversionError, float::FBig};
use num::Zero;

use crate::{
    core::{Function, Measurement, MetricSpace, PrivacyMap},
    domains::AllDomain,
    error::Fallible,
    interactive::Queryable,
    measures::MaxDivergence,
    metrics::LInfDistance,
    traits::{
        samplers::{LaplaceRV, PartialSample},
        InfCast, InfDiv, InfMul, Number,
    },
};

#[cfg(test)]
mod test;

pub fn make_above_threshold<TI, QI: Number>(
    input_domain: AllDomain<Queryable<TI, QI>>,
    input_metric: LInfDistance<QI>,
    scale: f64,
    threshold: QI,
) -> Fallible<
    Measurement<AllDomain<Queryable<TI, QI>>, Queryable<TI, bool>, LInfDistance<QI>, MaxDivergence>,
>
where
    TI: 'static,
    f64: InfCast<QI>,
    FBig: TryFrom<QI, Error = ConversionError> + TryFrom<f64>,
    (AllDomain<Queryable<TI, QI>>, LInfDistance<QI>): MetricSpace,
{
    let threshold_scale = scale.inf_mul(&if input_metric.monotonic { 1.0 } else { 2.0 })?;

    let f_scale_stream = FBig::try_from(scale)?;
    let f_scale_thresh = FBig::try_from(threshold_scale)?;
    let f_thresh = FBig::try_from(threshold)?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Queryable<TI, QI>| {
            let mut stream = arg.clone();
            let mut found = false;

            let mut noisy_threshold =
                PartialSample::new(LaplaceRV::new(f_thresh.clone(), f_scale_thresh.clone())?);
            let f_scale_stream = f_scale_stream.clone();

            Queryable::new_external(move |query: &TI| {
                if found {
                    return fallible!(FailedFunction, "queries exhausted");
                }

                let aggregate = FBig::try_from(stream.eval(query)?)?;
                let mut aggregate_psrn =
                    PartialSample::new(LaplaceRV::new(aggregate, f_scale_stream.clone())?);

                if aggregate_psrn.greater_than(&mut noisy_threshold)? {
                    found = true;
                }

                Ok(found)
            })
        }),
        input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &QI| {
            let d_in = input_metric.range_distance(*d_in)?;

            let d_in = f64::inf_cast(d_in.clone())?;
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }

            if d_in.is_zero() {
                return Ok(0.0);
            }

            if scale.is_zero() {
                return Ok(f64::INFINITY);
            }

            // d_in / scale
            d_in.inf_div(&scale)
        }),
    )
}
