#[cfg(feature = "ffi")]
mod ffi;

use std::fmt::Display;

use dashu::float::FBig;
use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::LInfDistance,
    traits::{samplers::PartialSample, InfAdd, InfCast, InfDiv, Number},
};

use crate::traits::{samplers::GumbelRV, DistanceConstant};

#[cfg(test)]
mod test;

#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "polars", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "polars", serde(rename_all = "lowercase"))]
pub enum Optimize {
    Min,
    Max,
}

impl Display for Optimize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Optimize::Min => f.write_str("min"),
            Optimize::Max => f.write_str("max"),
        }
    }
}

impl TryFrom<&str> for Optimize {
    type Error = crate::error::Error;
    fn try_from(s: &str) -> Fallible<Self> {
        Ok(match s {
            "min" => Optimize::Min,
            "max" => Optimize::Max,
            _ => return fallible!(FailedCast, "optimize must be \"min\" or \"max\""),
        })
    }
}

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
/// * `scale` - Higher scales are more private.
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
    f64: DistanceConstant<TIA>,
    FBig: TryFrom<TIA> + TryFrom<f64>,
{
    if input_domain.element_domain.nullable() {
        return fallible!(MakeMeasurement, "input domain must be non-nullable");
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let scale_frac = FBig::try_from(scale.clone())
        .map_err(|_| err!(MakeMeasurement, "scale parameter must be finite"))?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<TIA>| {
            select_score(arg.iter().cloned(), optimize.clone(), scale_frac.clone())
        }),
        input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(report_noisy_max_gumbel_map(scale, input_metric)),
    )
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

pub fn select_score<TIA>(
    iter: impl Iterator<Item = TIA>,
    optimize: Optimize,
    scale: FBig,
) -> Fallible<usize>
where
    TIA: PartialOrd,
    FBig: TryFrom<TIA>,
{
    if scale.is_zero() {
        let cmp = |l: &TIA, r: &TIA| match optimize {
            Optimize::Max => l > r,
            Optimize::Min => l < r,
        };
        return Ok(iter
            .enumerate()
            .reduce(|l, r| if cmp(&l.1, &r.1) { l } else { r })
            .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
            .0);
    }

    (iter.enumerate())
        // skip NaN scores. These should not be in the input domain, but this results in graceful failure
        .filter_map(|(i, v)| Some((i, FBig::try_from(v).ok()?)))
        .map(|(i, mut shift)| {
            // normalize sign
            if optimize == Optimize::Min {
                shift = -shift;
            }

            // create a partial sample
            Ok((i, PartialSample::new(GumbelRV::new(shift, scale.clone())?)))
        })
        .reduce(|l, r| {
            let (mut l, mut r) = (l?, r?);
            Ok(if l.1.greater_than(&mut r.1)? { l } else { r })
        })
        .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
        .map(|v| v.0)
}
