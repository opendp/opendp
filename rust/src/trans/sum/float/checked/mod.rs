use crate::{
    core::{Function, StabilityRelation, Transformation},
    dist::{AbsoluteDistance, IntDistance, SymmetricDistance},
    dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    samplers::Shuffle,
    traits::{ExactIntCast, InfAdd, InfCast, InfMul, InfSub},
};

use super::{Float, Pairwise, Sequential, SumError};

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_bounded_float_checked_sum<S>(
    size_limit: usize,
    bounds: (S::Item, S::Item),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<S::Item>>>,
        AllDomain<S::Item>,
        SymmetricDistance,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: CheckedSum,
    S::Item: 'static + Float,
{
    let size_limit_ = S::Item::exact_int_cast(size_limit)?;
    let (lower, upper) = bounds.clone();

    lower
        .inf_mul(&size_limit_)
        .or(upper.inf_mul(&size_limit_))
        .map_err(|_| {
            err!(
                MakeTransformation,
                "potential for overflow when computing function"
            )
        })?;

    let error = S::error(size_limit, lower.clone(), upper.clone())?;
    let ideal_sensitivity = upper.inf_sub(&lower)?;

    Ok(Transformation::new(
        SizedDomain::new(
            VectorDomain::new(BoundedDomain::new_closed(bounds)?),
            size_limit,
        ),
        AllDomain::new(),
        Function::new_fallible(move |arg: &Vec<S::Item>| {
            let mut data = arg.clone();
            if arg.len() > size_limit {
                data.shuffle()?
            }
            Ok(S::infallible_sum(&data[..size_limit.min(data.len())]))
        }),
        SymmetricDistance::default(),
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

pub fn make_sized_bounded_float_checked_sum<S>(
    size: usize,
    bounds: (S::Item, S::Item),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<S::Item>>>,
        AllDomain<S::Item>,
        SymmetricDistance,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: CheckedSum,
    S::Item: 'static + Float,
{
    let size_ = S::Item::exact_int_cast(size)?;
    let (lower, upper) = bounds.clone();
    let error = S::error(size, lower.clone(), upper.clone())?;

    lower
        .inf_mul(&size_)
        .or(upper.inf_mul(&size_))
        .map_err(|_| {
            err!(
                MakeTransformation,
                "potential for overflow when computing function"
            )
        })?;

    let ideal_sensitivity = upper.inf_sub(&lower)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(move |arg: &Vec<S::Item>| S::infallible_sum(arg)),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(move |d_in: &IntDistance| {
            // d_out =  |BS*(v) - BS*(v')| where BS* is the finite sum and BS the ideal sum
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in * (U - L) + 2 * error
            //       =  d_in * (U - L) + 2 * n^2/2^k * max(|L|, U)
            //       =  d_in * (U - L) + n^2/2^(k - 1) * max(|L|, U)
            S::Item::inf_cast(*d_in)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&error)
        }),
    ))
}

pub trait CheckedSum: SumError {
    fn infallible_sum(arg: &[Self::Item]) -> Self::Item;
}
impl<T: Float> CheckedSum for Sequential<T> {
    fn infallible_sum(arg: &[T]) -> T {
        arg.iter().cloned().sum()
    }
}

impl<T: Float> CheckedSum for Pairwise<T> {
    fn infallible_sum(arg: &[T]) -> T {
        match arg.len() {
            0 => T::zero(),
            1 => arg[0].clone(),
            n => {
                let m = n / 2;
                Self::infallible_sum(&arg[..m]) + Self::infallible_sum(&arg[m..])
            }
        }
    }
}
