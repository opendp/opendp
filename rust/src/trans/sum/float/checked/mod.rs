use crate::{
    core::{Function, StabilityRelation, Transformation},
    dist::{AbsoluteDistance, IntDistance, SymmetricDistance},
    dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    samplers::Shuffle,
    traits::{InfAdd, InfCast, InfMul, InfSub, TotalOrd, AlertingAbs},
    trans::CanSumOverflow,
};

use super::{Float, Pairwise, Sequential, SumRelaxation};

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_bounded_float_checked_sum<S>(
    size_limit: usize,
    bounds: (S::Item, S::Item),
) -> Fallible<
    Transformation<
        VectorDomain<BoundedDomain<S::Item>>,
        AllDomain<S::Item>,
        SymmetricDistance,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: UncheckedSum,
    S::Item: 'static + Float,
{
    if S::Item::sum_can_overflow(size_limit, bounds) {
        return fallible!(
            MakeTransformation,
            "potential for overflow when computing function"
        );
    }

    let (lower, upper) = bounds.clone();
    let ideal_sensitivity = upper.inf_sub(&lower)?.total_max(lower.alerting_abs()?.total_max(upper)?)?;
    let relaxation = S::relaxation(size_limit, lower, upper)?;

    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new_fallible(move |arg: &Vec<S::Item>| {
            let mut data = arg.clone();
            if arg.len() > size_limit {
                data.shuffle()?
            }
            Ok(S::unchecked_sum(&data[..size_limit.min(data.len())]))
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(move |d_in: &IntDistance| {
            // d_out =  |BS*(v) - BS*(v')| where BS* is the finite sum and BS the ideal sum
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in * max(|L|, U) + 2 * error
            //       =  d_in * max(|L|, U) + relaxation
            S::Item::inf_cast(*d_in)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)
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
    S: UncheckedSum,
    S::Item: 'static + Float,
{
    if S::Item::sum_can_overflow(size, bounds) {
        return fallible!(
            MakeTransformation,
            "potential for overflow when computing function"
        );
    }

    let (lower, upper) = bounds.clone();
    let ideal_sensitivity = upper.inf_sub(&lower)?;
    let relaxation = S::relaxation(size, lower, upper)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        // Under the assumption that the input data is in input domain, then an unchecked sum is safe.
        Function::new(move |arg: &Vec<S::Item>| S::unchecked_sum(arg)),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(move |d_in: &IntDistance| {
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

pub trait UncheckedSum: SumRelaxation {
    fn unchecked_sum(arg: &[Self::Item]) -> Self::Item;
}
impl<T: Float> UncheckedSum for Sequential<T> {
    fn unchecked_sum(arg: &[T]) -> T {
        arg.iter().cloned().sum()
    }
}

impl<T: Float> UncheckedSum for Pairwise<T> {
    fn unchecked_sum(arg: &[T]) -> T {
        match arg.len() {
            0 => T::zero(),
            1 => arg[0].clone(),
            n => {
                let m = n / 2;
                Self::unchecked_sum(&arg[..m]) + Self::unchecked_sum(&arg[m..])
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_bounded_float_checked_sum() -> Fallible<()> {
        let trans = make_bounded_float_checked_sum::<Sequential<f64>>(4, (1., 10.))?;
        let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
        assert_eq!(sum, 10.);

        let trans = make_bounded_float_checked_sum::<Pairwise<f32>>(4, (1., 10.))?;
        let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
        assert_eq!(sum, 10.);

        assert!(make_bounded_float_checked_sum::<Pairwise<f32>>(100000000, (1e20, 1e30)).is_err());

        Ok(())
    }

    #[test]
    fn test_make_sized_bounded_float_checked_sum() -> Fallible<()> {
        let trans = make_sized_bounded_float_checked_sum::<Sequential<f64>>(4, (1., 10.))?;
        let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
        assert_eq!(sum, 10.);

        let trans = make_sized_bounded_float_checked_sum::<Pairwise<f32>>(4, (1., 10.))?;
        let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
        assert_eq!(sum, 10.);

        assert!(
            make_sized_bounded_float_checked_sum::<Pairwise<f32>>(100000000, (1e20, 1e30)).is_err()
        );

        Ok(())
    }
}
