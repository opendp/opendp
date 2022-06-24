use num::{One, Zero};

use crate::{
    core::{Function, StabilityRelation, Transformation},
    dist::{AbsoluteDistance, InsertDeleteDistance, IntDistance},
    dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{
        AlertingAbs, CheckNull, DistanceConstant, ExactIntCast, FloatBits, InfAdd, InfCast, InfDiv,
        InfMul, InfPow, InfSub, SaturatingAdd,
    },
};

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_bounded_float_ordered_sum<T>(
    size_limit: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        VectorDomain<BoundedDomain<T>>,
        AllDomain<T>,
        InsertDeleteDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: DistanceConstant<IntDistance>
        + Zero
        + One
        + ExactIntCast<usize>
        + ExactIntCast<IntDistance>
        + ExactIntCast<T::Bits>
        + InfAdd
        + InfSub
        + InfMul
        + InfDiv
        + InfPow
        + FloatBits
        + AlertingAbs
        + SaturatingAdd
        + CheckNull,
    IntDistance: InfCast<T>,
{
    let (lower, upper) = bounds.clone();

    let size_limit_ = T::exact_int_cast(size_limit)?;
    let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
    let _2 = T::one().inf_add(&T::one())?;

    let accuracy = size_limit_
        .inf_mul(&size_limit_)?
        .inf_div(&_2.inf_pow(&mantissa_bits.neg_inf_sub(&T::one())?)?)?;
    let ideal_sensitivity = lower.alerting_abs()?.total_max(upper)?;

    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| {
            arg.iter()
                .take(size_limit)
                .fold(T::zero(), |acc, v| acc.saturating_add(v))
        }),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(move |d_in: &IntDistance| {
            // d_out =  |BS*(v) - BS*(v')| where BS* is the finite sum and BS the ideal sum
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in * max(|L|, U) + 2 * accuracy
            //       =  d_in * max(|L|, U) + 2 * n^2/2^k * max(|L|, U)
            //       =  (d_in + n^2/2^(k - 1)) * max(|L|, U)
            T::inf_cast(*d_in)?.inf_add(&accuracy)?.inf_mul(&ideal_sensitivity)
        }),
    ))
}

pub fn make_sized_bounded_float_ordered_sum<T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<T>>>,
        AllDomain<T>,
        InsertDeleteDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: DistanceConstant<IntDistance>
        + ExactIntCast<T::Bits>
        + InfAdd
        + InfPow
        + Zero
        + One
        + FloatBits
        + ExactIntCast<usize>
        + InfSub
        + SaturatingAdd
        + CheckNull,
    IntDistance: InfCast<T>,
{
    let (lower, upper) = bounds.clone();

    let size_ = T::exact_int_cast(size)?;
    let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
    let _2 = T::one().inf_add(&T::one())?;

    let accuracy = size_
        .inf_mul(&size_)?
        .inf_div(&_2.inf_pow(&mantissa_bits.neg_inf_sub(&T::one())?)?)?;
    let ideal_sensitivity = upper.inf_sub(&lower)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| {
            arg.iter()
                .take(size)
                .fold(T::zero(), |acc, v| acc.saturating_add(v))
        }),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(move |d_in: &IntDistance| {
            // d_out =  |BS*(v) - BS*(v')| where BS* is the finite sum and BS the ideal sum
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in * (U - L) + 2 * accuracy
            //       =  d_in * (U - L) + 2 * n^2/2^k * (U - L)
            //       =  (d_in + n^2/2^(k - 1)) * (U - L)
            T::inf_cast(*d_in)?.inf_add(&accuracy)?.inf_mul(&ideal_sensitivity)
        }),
    ))
}
