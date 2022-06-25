use crate::{
    core::{Function, StabilityRelation, Transformation},
    dist::{AbsoluteDistance, InsertDeleteDistance, IntDistance},
    dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    samplers::Shuffle,
    traits::{AlertingAbs, InfAdd, InfCast, InfMul, InfSub, TotalOrd},
};

use super::{Float, SaturatingSum};

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_bounded_float_ordered_sum<S>(
    size_limit: usize,
    bounds: (S::Item, S::Item),
) -> Fallible<
    Transformation<
        VectorDomain<BoundedDomain<S::Item>>,
        AllDomain<S::Item>,
        InsertDeleteDistance,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: SaturatingSum,
    S::Item: 'static + Float,
{
    let (lower, upper) = bounds.clone();

    let error = S::error(size_limit, lower.clone(), upper.clone())?;
    let ideal_sensitivity = lower.alerting_abs()?.total_max(upper)?;

    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new_fallible(move |arg: &Vec<S::Item>| {
            let mut data = arg.clone();
            if arg.len() > size_limit {
                data.shuffle()?
            }
            Ok(S::saturating_sum(&data[..size_limit.min(data.len())]))
        }),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(move |d_in: &IntDistance| {
            // d_out =  |BS*(v) - BS*(v')| where BS* is the finite sum and BS the ideal sum
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in * max(|L|, U) + 2 * accuracy
            //       =  d_in * max(|L|, U) + 2 * n^2/2^k * max(|L|, U)
            //       =  d_in * max(|L|, U) + n^2/2^(k - 1) * max(|L|, U)
            S::Item::inf_cast(*d_in)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&error)
        }),
    ))
}

pub fn make_sized_bounded_float_ordered_sum<S>(
    size: usize,
    bounds: (S::Item, S::Item),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<S::Item>>>,
        AllDomain<S::Item>,
        InsertDeleteDistance,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: SaturatingSum,
    S::Item: 'static + Float,
{
    let (lower, upper) = bounds.clone();
    let error = S::error(size, lower.clone(), upper.clone())?;
    let ideal_sensitivity = upper.inf_sub(&lower)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(move |arg: &Vec<S::Item>| S::saturating_sum(arg)),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(move |d_in: &IntDistance| {
            // d_out =  |BS*(v) - BS*(v')| where BS* is the finite sum and BS the ideal sum
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in * (U - L) + 2 * accuracy
            //       =  d_in * (U - L) + 2 * n^2/2^k * (U - L)
            //       =  (d_in + n^2/2^(k - 1)) * (U - L)
            S::Item::inf_cast(*d_in)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&error)
        }),
    ))
}
