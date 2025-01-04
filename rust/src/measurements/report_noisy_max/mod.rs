use crate::{
    core::{Function, Measure, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::{BoundedRange, MaxDivergence},
    metrics::LInfDistance,
    traits::{
        samplers::{ExponentialRV, GumbelRV, InverseCDF, PartialSample},
        InfCast, InfDiv, Number,
    },
};
use dashu::float::FBig;
use num::Zero;
use opendp_derive::bootstrap;
use std::fmt::Display;

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(
        optimize(c_type = "char *", rust_type = "String"),
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
    ),
    generics(MO(suppress), TIA(suppress))
)]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable VectorDomain.
/// * `input_metric` - Metric on the input domain. Must be LInfDistance
/// * `output_measure` - One of `MaxDivergence` or `BoundedRange`.
/// * `scale` - Scale for the noise distribution.
/// * `optimize` - Indicate whether to privately return the "max" or "min"
///
/// # Generics
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
pub fn make_report_noisy_max<MO: SelectionMeasure, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    output_measure: MO,
    scale: f64,
    optimize: Optimize,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, usize, LInfDistance<TIA>, MO>>
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
            if scale.is_zero() {
                let cmp = |l: &TIA, r: &TIA| match optimize {
                    Optimize::Max => l > r,
                    Optimize::Min => l < r,
                };
                return Ok(arg
                    .iter()
                    .enumerate()
                    .reduce(|l, r| if cmp(&l.1, &r.1) { l } else { r })
                    .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
                    .0);
            }

            (arg.iter().enumerate())
                // skip NaN scores. These should not be in the input domain, but this results in graceful failure
                .filter_map(|(i, v)| Some((i, FBig::try_from(*v).ok()?)))
                .map(|(i, mut shift)| {
                    // normalize sign
                    if optimize == Optimize::Min {
                        shift = -shift;
                    }

                    // create a partial sample
                    Ok((i, PartialSample::new(MO::RV::new(shift, f_scale.clone())?)))
                })
                .reduce(|l, r| {
                    let (mut l, mut r) = (l?, r?);
                    Ok(if l.1.greater_than(&mut r.1)? { l } else { r })
                })
                .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
                .map(|v| v.0)
        }),
        input_metric.clone(),
        output_measure,
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            // convert L_\infty distance to range distance
            let d_in = input_metric.range_distance(d_in.clone())?;

            // convert data type to QO
            let d_in = f64::inf_cast(d_in)?;

            if d_in.is_sign_negative() {
                return fallible!(
                    InvalidDistance,
                    "sensitivity ({}) must be non-negative",
                    d_in
                );
            }

            if scale.is_zero() {
                return Ok(f64::INFINITY);
            }

            // d_out >= d_in / scale
            d_in.inf_div(&scale)
        }),
    )
}

pub trait SelectionMeasure: 'static + Measure<Distance = f64> {
    type RV: ShiftScaleRV;
}

impl SelectionMeasure for BoundedRange {
    type RV = GumbelRV;
}

impl SelectionMeasure for MaxDivergence {
    type RV = ExponentialRV;
}

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

pub trait ShiftScaleRV: InverseCDF + Sized {
    fn new(shift: FBig, scale: FBig) -> Fallible<Self>;
}

impl ShiftScaleRV for GumbelRV {
    fn new(shift: FBig, scale: FBig) -> Fallible<Self> {
        GumbelRV::new(shift, scale)
    }
}

impl ShiftScaleRV for ExponentialRV {
    fn new(shift: FBig, scale: FBig) -> Fallible<Self> {
        ExponentialRV::new(shift, scale)
    }
}
