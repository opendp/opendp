use num::{CheckedSub, NumCast};

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AllDomain, BoundedDomain, VectorDomain},
    error::Fallible,
    metrics::InfDifferenceDistance,
    traits::{samplers::SampleBernoulli, CheckedAbs, Float, InfCast, Integer, RoundCast},
};

// float_scorer >> lipschitz_mul >> lipschitz_cast_integer >> lipschitz_utility_scc >> exponential
// int_scorer >> lipschitz_utility_scc >> exponential

// the bound given in I19
pub fn make_lipschitz_randomized_round<TI, TO>() -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TI>>,
        VectorDomain<AllDomain<TO>>,
        InfDifferenceDistance<TI>,
        InfDifferenceDistance<TI>,
    >,
>
where
    bool: SampleBernoulli<TI>,
    TI: Float,
    TO: Integer + RoundCast<TI> + InfCast<TI>,
{
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(|arg: &Vec<TI>| {
            arg.iter()
                .map(|v| {
                    Ok(TO::round_cast(
                        v.trunc()
                            + if bool::sample_bernoulli(v.fract().abs(), false)? {
                                TI::zero()
                            } else {
                                TI::one()
                            },
                    )
                    .unwrap_or_else(|_| {
                        if v.is_sign_negative() {
                            TO::MIN_FINITE
                        } else {
                            TO::MAX_FINITE
                        }
                    }))
                })
                .collect()
        }),
        InfDifferenceDistance::default(),
        InfDifferenceDistance::default(),
        StabilityMap::new(|d_in| *d_in),
    ))
}

// shift + cast + clamp
pub fn make_lipschitz_utility_scc<TI, TO>(
    bounds: (TI, TI),
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TI>>,
        VectorDomain<BoundedDomain<TO>>,
        InfDifferenceDistance<TI>,
        InfDifferenceDistance<TO>,
    >,
>
where
    TI: Integer + CheckedAbs + CheckedSub + NumCast,
    TO: Integer + RoundCast<TI> + InfCast<TI> + NumCast + Ord,
{
    let max: TO = NumCast::from(bounds.1.alerting_sub(&bounds.0)?)
        .ok_or_else(|| err!(MakeTransformation, "bounds are too wide for target type"))?;

    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new(BoundedDomain::new_closed((TO::zero(), max))?),
        Function::new(move |arg: &Vec<TI>| {
            arg.iter()
                .cloned()
                .map(move |v: TI| -> TO {
                    if v > bounds.0 {
                        // since v > lower, then v - lower > 0, so NumCast to unsigned is valid
                        v.checked_sub(&bounds.0)
                            .and_then(|v| NumCast::from(v))
                            // since v - lower > 0 then checked_sub can only overflow to inf
                            .unwrap_or(max)
                            .min(max)
                    } else {
                        // v is lte lower, so v - lower is negative. Clamp to zero
                        TO::zero()
                    }
                })
                .collect()
        }),
        InfDifferenceDistance::default(),
        InfDifferenceDistance::default(),
        StabilityMap::new_from_constant(TO::one()),
    ))
}
