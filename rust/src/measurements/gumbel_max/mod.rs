#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::LInfDistance,
    traits::{Float, Number},
};

#[cfg(feature = "use-mpfr")]
use crate::traits::{
    samplers::{CastInternalRational, GumbelPSRN},
    DistanceConstant,
};
#[cfg(feature = "use-mpfr")]
use rug::ops::NegAssign;

#[cfg(not(feature = "use-mpfr"))]
use crate::traits::{samplers::SampleUniform, CheckNull, InfCast, RoundCast};

#[derive(PartialEq)]
pub enum Optimize {
    Max,
    Min,
}

#[cfg(feature = "use-mpfr")]
#[bootstrap(
    features("contrib", "floating-point"),
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
) -> Fallible<
    Measurement<VectorDomain<AtomDomain<TIA>>, usize, LInfDistance<TIA>, MaxDivergence<QO>>,
>
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
            (arg.iter().cloned().enumerate())
                .map(|(i, v)| {
                    let mut shift = v.into_rational()?;
                    if optimize == Optimize::Min {
                        shift.neg_assign();
                    }
                    Ok((i, GumbelPSRN::new(shift, scale_frac.clone())))
                })
                .reduce(|l, r| {
                    let (mut l, mut r) = (l?, r?);
                    Ok(if l.1.greater_than(&mut r.1)? { l } else { r })
                })
                .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
                .map(|v| v.0)
        }),
        input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            // convert L_\infty distance to range distance
            let d_in = input_metric.range_distance(*d_in)?;

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
        }),
    )
}

#[cfg(not(feature = "use-mpfr"))]
pub fn make_report_noisy_max_gumbel<TIA, QO>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    scale: QO,
    optimize: Optimize,
) -> Fallible<
    Measurement<VectorDomain<AtomDomain<TIA>>, usize, LInfDistance<TIA>, MaxDivergence<QO>>,
>
where
    TIA: Clone + Number,
    QO: 'static
        + crate::traits::DistanceConstant<TIA>
        + crate::traits::RoundCast<TIA>
        + Float
        + crate::traits::samplers::SampleUniform,
{
    if input_domain.element_domain.nullable() {
        return fallible!(MakeMeasurement, "input domain must be non-nullable");
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let sign = match optimize {
        Optimize::Max => QO::one(),
        Optimize::Min => QO::one().neg(),
    };

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<TIA>| {
            arg.iter()
                .cloned()
                .map(|v| QO::round_cast(v).map(|v| sign * v / scale))
                // enumerate before sampling so that indexes are inside the result
                .enumerate()
                // gumbel samples are porous
                .map(|(i, llik)| {
                    let llik = llik?;
                    QO::sample_standard_uniform(false).map(|u| (i, llik - u.ln().neg().ln()))
                })
                // retrieve the highest noisy likelihood pair
                .try_fold((arg.len(), QO::neg_infinity()), |acc: (usize, QO), res| {
                    res.map(|v| if acc.1 > v.1 { acc } else { v })
                })
                // only return the index
                .map(|v| v.0)
        }),
        input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            // convert L_\infty distance to range distance
            let d_in = input_metric.range_distance(*d_in)?;

            // convert data type to QO
            let d_in = QO::inf_cast(d_in)?;

            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if d_in.is_zero() {
                return Ok(QO::zero());
            }
            // d_out >= d_in / scale
            d_in.inf_div(&scale)
        }),
    )
}

#[cfg(feature = "floating-point")]
#[cfg(test)]
pub mod test_exponential {
    use crate::error::Fallible;

    use super::*;

    #[test]
    fn test_exponential() -> Fallible<()> {
        let input_domain = VectorDomain::new(AtomDomain::default());
        let input_metric = LInfDistance::default();
        let de = make_report_noisy_max_gumbel(input_domain, input_metric, 1., Optimize::Max)?;
        let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
        println!("{:?}", release);

        Ok(())
    }
}
