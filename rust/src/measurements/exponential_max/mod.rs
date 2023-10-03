#[cfg(feature = "ffi")]
mod ffi;

use core::f64;

use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::LInfDistance,
    traits::{InfAdd, InfCast, InfDiv, Number},
};

use crate::traits::samplers::ExponentialRV;
use dashu::float::FBig;

use super::{select_score, Optimize};

#[cfg(test)]
pub mod test;

#[bootstrap(
    features("contrib"),
    arguments(optimize(c_type = "char *", rust_type = "String")),
    generics(TIA(suppress))
)]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score
/// with noise added from the exponential distribution.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable VectorDomain.
/// * `input_metric` - Metric on the input domain. Must be LInfDistance
/// * `scale` - Higher scales are more private.
/// * `optimize` - Indicate whether to privately return the "max" or "min"
///
/// # Generics
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
/// * `QO` - Output Distance Type.
pub fn make_report_noisy_max_exponential<TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    scale: f64,
    optimize: Optimize,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, usize, LInfDistance<TIA>, MaxDivergence>>
where
    TIA: Number,
    FBig: TryFrom<TIA> + TryFrom<f64>,
    f64: InfCast<TIA>,
{
    if input_domain.element_domain.nan() {
        return fallible!(MakeMeasurement, "values must be non-nan");
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let f_scale =
        FBig::try_from(scale.clone()).map_err(|_| err!(MakeMeasurement, "scale must be finite"))?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<TIA>| {
            select_score::<_, ExponentialRV>(arg.iter().cloned(), optimize.clone(), f_scale.clone())
        }),
        input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(report_noisy_max_exponential_map(scale, input_metric)),
    )
}

pub(crate) fn report_noisy_max_exponential_map<QI>(
    scale: f64,
    input_metric: LInfDistance<QI>,
) -> impl Fn(&QI) -> Fallible<f64>
where
    QI: Clone + InfAdd,
    f64: InfCast<QI>,
{
    move |d_in: &QI| {
        // convert L_\infty distance to range distance
        let d_in = input_metric.range_distance(d_in.clone())?;

        // convert data type to QO
        let d_in = f64::inf_cast(d_in)?;

        if d_in.is_sign_negative() {
            return fallible!(InvalidDistance, "sensitivity must be non-negative");
        }

        if scale.is_zero() {
            return Ok(f64::INFINITY);
        }

        // d_out >= d_in / scale
        d_in.inf_div(&scale)
    }
}
