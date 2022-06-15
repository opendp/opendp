use num::{One, Zero};

use crate::{
    core::{Function, StabilityMap, Transformation},
    core::{AbsoluteDistance, InsertDeleteDistance, IntDistance},
    core::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{
        AlertingAbs, CheckNull, DistanceConstant, ExactIntCast, FloatBits, InfAdd, InfCast, InfDiv,
        InfMul, InfPow, InfSub, SaturatingAdd,
    },
};

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_bounded_float_sequential_sum<T>(
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

    let relaxation = size_limit_
        .inf_mul(&size_limit_)?
        .inf_div(&_2.inf_pow(&mantissa_bits)?)?;
    let ideal_sensitivity = lower.alerting_abs()?.total_max(upper)?;

    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| arg.iter().take(size_limit)
            .fold(T::zero(), |acc, v| acc.saturating_add(v))),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            T::inf_cast(*d_in)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)
        }),
    ))
}

pub fn make_sized_bounded_float_sequential_sum<T>(
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
        + InfDiv
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

    let relaxation = size_
        .inf_mul(&size_)?
        .inf_div(&_2.inf_pow(&mantissa_bits)?)?;
    let ideal_sensitivity = upper.inf_sub(&lower)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| arg.iter().take(size)
            .fold(T::zero(), |acc, v| acc.saturating_add(v))),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            T::inf_cast(*d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)
        }),
    ))
}
