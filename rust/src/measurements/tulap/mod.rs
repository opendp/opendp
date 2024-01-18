use dashu::float::FBig;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::FixedSmoothedMaxDivergence,
    metrics::{AbsoluteDistance, IntDistance, PartitionDistance},
    traits::{
        samplers::{pinpoint, TulapPSRN},
        InfCast, InfMul,
    },
};

#[cfg(feature = "ffi")]
mod ffi;

/// Make a Measurement that adds noise from the Tulap distribution to the input.
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric of the input.
/// * `epsilon` - Privacy parameter ε.
/// * `delta` - Privacy parameter δ.
#[bootstrap(features("contrib"))]
pub fn make_tulap(
    input_domain: VectorDomain<AtomDomain<f64>>,
    input_metric: PartitionDistance<AbsoluteDistance<f64>>,
    epsilon: f64,
    delta: f64,
) -> Fallible<
    Measurement<
        VectorDomain<AtomDomain<f64>>,
        Vec<f64>,
        PartitionDistance<AbsoluteDistance<f64>>,
        FixedSmoothedMaxDivergence<f64>,
    >,
> {
    let f_epsilon = FBig::try_from(epsilon)?;
    let f_delta = FBig::try_from(delta)?;
    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<f64>| {
            arg.iter()
                .map(|v| {
                    let shift = FBig::try_from(*v).unwrap_or(FBig::ZERO);
                    let mut tulap = TulapPSRN::new(shift, f_epsilon.clone(), f_delta.clone());
                    pinpoint::<TulapPSRN, f64>(&mut tulap)
                })
                .collect()
        }),
        input_metric,
        FixedSmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |&(l0, _l1, linf): &(IntDistance, f64, f64)| {
            let d_in = f64::inf_cast(l0)?.inf_mul(&linf)?;
            Ok((d_in.inf_mul(&epsilon)?, d_in.inf_mul(&delta)?))
        }),
    )
}
