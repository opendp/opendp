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
    traits::{
        samplers::{ExponentialDist, InverseCDF, PSRN},
        Float, InfAdd, InfCast, Number,
    },
};

use crate::traits::{
    samplers::{CastInternalRational, GumbelDist},
    DistanceConstant,
};

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
/// * `scale` - Noise scale for the Gumbel distribution.
/// * `optimize` - Indicate whether to privately return the "max" or "min"
///
/// # Generics
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
/// * `QO` - Output Distance Type.
pub fn make_report_noisy_max_gumbel<TIA, QO>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    scale: QO,
    optimize: Optimize,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, usize, LInfDistance<TIA>, MaxDivergence<QO>>>
where
    TIA: Number + CastInternalRational,
    QO: CastInternalRational + DistanceConstant<TIA> + Float,
    FBig: TryFrom<TIA> + TryFrom<QO>,
{
    if input_domain.element_domain.nullable() {
        return fallible!(MakeMeasurement, "values must be non-null");
    }

    if input_domain.element_domain.nullable() {
        return fallible!(MakeMeasurement, "input domain must be non-nullable");
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let f_scale =
        FBig::try_from(scale.clone()).map_err(|_| err!(MakeMeasurement, "scale must be finite"))?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<TIA>| {
            select_score::<_, GumbelDist>(arg.iter().cloned(), optimize.clone(), f_scale.clone())
        }),
        input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(report_noisy_max_gumbel_map(scale, input_metric)),
    )
}

pub(crate) fn report_noisy_max_gumbel_map<QI, QO>(
    scale: QO,
    input_metric: LInfDistance<QI>,
) -> impl Fn(&QI) -> Fallible<QO>
where
    QI: Clone + InfAdd,
    QO: Float + InfCast<QI>,
{
    move |d_in: &QI| {
        // convert L_\infty distance to range distance
        let d_in = input_metric.range_distance(d_in.clone())?;

        // convert data type to QO
        let d_in = QO::inf_cast(d_in)?;

        if d_in.is_sign_negative() {
            return fallible!(InvalidDistance, "sensitivity must be non-negative");
        }

        if scale.is_zero() {
            return Ok(QO::infinity());
        }

        // d_out >= d_in / scale
        d_in.inf_div(&scale)
    }
}

pub(crate) trait ArgmaxDist: InverseCDF + Sized {
    fn new(shift: FBig, scale: FBig) -> PSRN<Self>;
}

impl ArgmaxDist for GumbelDist {
    fn new(shift: FBig, scale: FBig) -> PSRN<Self> {
        GumbelDist::new_psrn(shift, scale)
    }
}
impl ArgmaxDist for ExponentialDist {
    fn new(shift: FBig, scale: FBig) -> PSRN<Self> {
        ExponentialDist::new_psrn(shift, scale)
    }
}

pub(crate) fn select_score<TIA, D: ArgmaxDist>(
    iter: impl Iterator<Item = TIA>,
    optimize: Optimize,
    scale: FBig,
) -> Fallible<usize>
where
    TIA: Number,
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
        .map(|(i, v)| {
            let mut shift = FBig::try_from(v).unwrap_or(FBig::ZERO);
            if optimize == Optimize::Min {
                shift = -shift;
            }
            Ok((i, D::new(shift, scale.clone())))
        })
        .reduce(|l, r| {
            let (mut l, mut r) = (l?, r?);
            Ok(if l.1.is_gt(&mut r.1)? { l } else { r })
        })
        .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
        .map(|v| v.0)
}
