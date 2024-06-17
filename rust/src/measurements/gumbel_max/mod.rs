#[cfg(feature = "ffi")]
mod ffi;

use std::fmt::Display;

use dashu::rational::RBig;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::LInfDistance,
    traits::{Float, InfAdd, InfCast, Number},
};

use crate::traits::{
    samplers::{CastInternalRational, GumbelPSRN},
    DistanceConstant,
};

#[derive(PartialEq, Clone)]
pub enum Optimize {
    Min,
    Max,
}

impl Display for Optimize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Optimize::Min => "min",
                Optimize::Max => "max",
            }
        )
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
/// * `optimize` - Indicate whether to privately return the "Max" or "Min"
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
{
    if input_domain.element_domain.nullable() {
        return fallible!(MakeMeasurement, "input domain must be non-nullable");
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let scale_frac = scale.clone().into_rational()?;

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

pub fn select_score<TIA>(
    iter: impl Iterator<Item = TIA>,
    optimize: Optimize,
    scale: RBig,
) -> Fallible<usize>
where
    TIA: Number + CastInternalRational,
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
            let mut shift = v.into_rational()?;
            if optimize == Optimize::Min {
                shift = -shift;
            }
            Ok((i, GumbelPSRN::new(shift, scale.clone())))
        })
        .reduce(|l, r| {
            let (mut l, mut r) = (l?, r?);
            Ok(if l.1.greater_than(&mut r.1)? { l } else { r })
        })
        .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
        .map(|v| v.0)
}

#[cfg(feature = "floating-point")]
#[cfg(test)]
mod test;
