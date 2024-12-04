#[cfg(feature = "ffi")]
mod ffi;

use dashu::float::FBig;
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

use crate::traits::samplers::GumbelRV;

use super::{select_score, ArgmaxRV, Optimize};

#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(optimize(c_type = "char *", rust_type = "String")),
    generics(TIA(suppress))
)]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable VectorDomain.
/// * `input_metric` - Metric on the input domain. Must be LInfDistance
/// * `scale` - Noise scale for the Gumbel distribution.
/// * `optimize` - Indicate whether to privately return the "max" or "min"
///
/// # Generics
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
pub fn make_report_noisy_max_gumbel<TIA>(
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
    if input_domain.element_domain.nullable() {
        return fallible!(
            MakeMeasurement,
            "elements in the input vector domain must be non-null"
        );
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
    }

    let f_scale = FBig::try_from(scale.clone())
        .map_err(|_| err!(MakeMeasurement, "scale ({}) must be finite", scale))?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<TIA>| {
            select_score::<_, GumbelRV>(arg.iter().cloned(), optimize.clone(), f_scale.clone())
        }),
        input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(report_noisy_max_gumbel_map(scale, input_metric)),
    )
}

impl ArgmaxRV for GumbelRV {
    fn new(shift: FBig, scale: FBig) -> Fallible<Self> {
        GumbelRV::new(shift, scale)
    }
}

pub(crate) fn report_noisy_max_gumbel_map<QI>(
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
