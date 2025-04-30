use dashu::float::FBig;
use num::Zero;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, StreamDomain},
    error::Fallible,
    interactive::Queryable,
    metrics::LInfDistance,
    traits::{InfCast, InfDiv, InfMul, Number, samplers::PartialSample},
};

use super::{Optimize, SelectionMeasure};

#[cfg(test)]
mod test;

pub fn make_sparse_vector<MO: SelectionMeasure, SI, SO: Number>(
    input_domain: StreamDomain<SI, AtomDomain<SO>>,
    input_metric: LInfDistance<SO>,
    output_measure: MO,
    k: usize,
    scale: f64,
    threshold: SO,
    optimize: Optimize,
) -> Fallible<
    Measurement<StreamDomain<SI, AtomDomain<SO>>, Queryable<SI, bool>, LInfDistance<SO>, MO>,
>
where
    SI: 'static,
    SO: Number,
    FBig: TryFrom<SO> + TryFrom<f64>,
    f64: InfCast<SO>,
{
    if input_domain.output_domain.nullable() {
        return fallible!(
            MakeMeasurement,
            "elements in stream's output domain must be non-null"
        );
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
    }

    let f_scale = FBig::try_from(scale.clone())
        .map_err(|_| err!(MakeMeasurement, "scale ({}) must be finite", scale))?;

    let scale_threshold = scale.inf_mul(&if input_metric.monotonic { 1.0 } else { 2.0 })?;
    let f_scale_threshold = FBig::try_from(scale_threshold.clone())
        .map_err(|_| err!(MakeMeasurement, "scale ({}) must be finite", scale))?;

    let mut f_threshold = FBig::try_from(threshold)
        .map_err(|_| err!(MakeMeasurement, "threshold ({}) must be finite", scale))?;

    if let Optimize::Min = optimize {
        f_threshold = -f_threshold;
    }

    Measurement::new(
        input_domain,
        Function::new_fallible(move |stream: &Queryable<SI, SO>| {
            let mut stream = stream.clone();
            let mut k = k.clone();

            let threshold_rv = MO::RV::new(f_threshold.clone(), f_scale_threshold.clone())?;
            let mut noisy_threshold = PartialSample::new(threshold_rv);

            let f_scale = f_scale.clone();

            Queryable::new_external(move |input: &SI| {
                if k == 0 {
                    return fallible!(FailedFunction, "queries exhausted");
                }

                let mut candidate = FBig::try_from(stream.eval(input)?).unwrap_or(FBig::ZERO);

                if let Optimize::Min = optimize {
                    candidate = -candidate;
                }

                let candidate_rv = MO::RV::new(candidate, f_scale.clone())?;
                let mut noisy_output = PartialSample::new(candidate_rv);

                let found = noisy_output.greater_than(&mut noisy_threshold)?;
                if found {
                    k -= 1;
                }

                Ok(found)
            })
        }),
        input_metric.clone(),
        output_measure,
        PrivacyMap::new_fallible(move |d_in: &SO| {
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
