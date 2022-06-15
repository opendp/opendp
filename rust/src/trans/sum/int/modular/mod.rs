
use num::{traits::WrappingAdd, Zero};

use crate::{
    error::Fallible, 
    core::{Transformation, Function, StabilityMap}, 
    core::{VectorDomain, BoundedDomain, AllDomain, SizedDomain}, 
    core::{SymmetricDistance, AbsoluteDistance, IntDistance, Modular}, 
    traits::{DistanceConstant, CheckNull, InfCast, InfSub, AlertingAbs}
};

use super::AddIsExact;

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_bounded_int_modular_sum<T>(
    bounds: (T, T),
) -> Fallible<
    Transformation<
        VectorDomain<BoundedDomain<T>>,
        AllDomain<T>,
        SymmetricDistance,
        Modular<AbsoluteDistance<T>>,
    >,
> 
where
    T: DistanceConstant<IntDistance>
        + CheckNull
        + Zero
        + AlertingAbs
        + WrappingAdd
        + AddIsExact,
    IntDistance: InfCast<T> 
{
    let (lower, upper) = bounds.clone();
    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>|
            arg.iter().fold(T::zero(), |sum, v| sum.wrapping_add(v))
        ),
        SymmetricDistance::default(),
        Modular::default(),
        StabilityMap::new_from_constant(lower.alerting_abs()?.total_max(upper)?)
    ))
}


pub fn make_sized_bounded_int_modular_sum<T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<T>>>,
        AllDomain<T>,
        SymmetricDistance,
        Modular<AbsoluteDistance<T>>,
    >,
> 
where
    T: DistanceConstant<IntDistance>
        + InfSub
        + CheckNull
        + Zero
        + WrappingAdd
        + AddIsExact,
    IntDistance: InfCast<T> 
{
    let (lower, upper) = bounds.clone();
    let range = upper.inf_sub(&lower)?;
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>|
            arg.iter().fold(T::zero(), |sum, v| sum.wrapping_add(v))
        ),
        SymmetricDistance::default(),
        Modular::default(),
        StabilityMap::new_fallible(
            // If d_in is odd, we still only consider databases with (d_in - 1) / 2 substitutions,
            //    so floor division is acceptable
            move |d_in: &IntDistance| T::inf_cast(d_in / 2).and_then(|d_in| d_in.inf_mul(&range)),
        ),
    ))
}