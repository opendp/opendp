use crate::{
    core::{Function, Transformation, StabilityMap},
    dist::{AbsoluteDistance, InsertDeleteDistance, IntDistance},
    dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{AlertingAbs, InfAdd, InfCast, InfMul, InfSub, TotalOrd},
};

use super::{Float, Pairwise, Sequential, SumRelaxation};

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
    let ideal_sensitivity = upper.inf_sub(&lower)?.total_max(lower.alerting_abs()?.total_max(upper)?)?;
    let relaxation = S::relaxation(size_limit, lower, upper)?;

    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new(move |arg: &Vec<S::Item>| {
            S::saturating_sum(&arg[..size_limit.min(arg.len())])
        }),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            // d_out =  |BS*(v) - BS*(v')| where BS* is the finite sum and BS the ideal sum
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in * ideal_sens + 2 * error
            //       =  d_in * ideal_sens + relaxation
            S::Item::inf_cast(*d_in)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)
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
    let ideal_sensitivity = upper.inf_sub(&lower)?;
    let relaxation = S::relaxation(size, lower, upper)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(move |arg: &Vec<S::Item>| S::saturating_sum(arg)),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            // d_out =  |BS*(v) - BS*(v')| where BS* is the finite sum and BS the ideal sum
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in * (U - L) + 2 * error
            //       =  d_in * (U - L) + relaxation
            S::Item::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)
        }),
    ))
}

pub trait SaturatingSum: SumRelaxation {
    fn saturating_sum(arg: &[Self::Item]) -> Self::Item;
}

impl<T: Float> SaturatingSum for Sequential<T> {
    fn saturating_sum(arg: &[T]) -> T {
        arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))
    }
}

impl<T: Float> SaturatingSum for Pairwise<T> {
    fn saturating_sum(arg: &[T]) -> T {
        match arg.len() {
            0 => T::zero(),
            1 => arg[0].clone(),
            n => {
                let m = n / 2;
                Self::saturating_sum(&arg[..m]).saturating_add(&Self::saturating_sum(&arg[m..]))
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_bounded_float_ordered_sum() -> Fallible<()> {
        let trans = make_bounded_float_ordered_sum::<Sequential<f64>>(4, (1., 10.))?;
        let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
        assert_eq!(sum, 10.);

        let trans = make_bounded_float_ordered_sum::<Pairwise<f32>>(4, (1., 10.))?;
        let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
        assert_eq!(sum, 10.);

        Ok(())
    }

    #[test]
    fn test_make_sized_bounded_float_ordered_sum() -> Fallible<()> {
        let trans = make_sized_bounded_float_ordered_sum::<Sequential<f64>>(4, (1., 10.))?;
        let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
        assert_eq!(sum, 10.);

        let trans = make_sized_bounded_float_ordered_sum::<Pairwise<f32>>(4, (1., 10.))?;
        let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
        assert_eq!(sum, 10.);
        
        Ok(())
    }
}