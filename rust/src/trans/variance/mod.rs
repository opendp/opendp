#[cfg(feature="ffi")]
mod ffi;

use std::iter::Sum;
use std::ops::{Add, Div, Sub, Mul};

use num::{Float, One, Zero};

use crate::core::{Function, StabilityRelation, Transformation};
use crate::dist::{AbsoluteDistance, IntDistance, SymmetricDistance};
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{CheckNull, DistanceConstant, ExactIntCast, InfCast, InfSub, InfAdd, InfMul};

pub fn make_sized_bounded_variance<T>(
    size: usize, bounds: (T, T), ddof: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<BoundedDomain<T>>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>> where
    T: DistanceConstant<IntDistance> + Float + One + Sub<Output=T> + Div<Output=T>
    + Sum<T> + for<'a> Sum<&'a T> + ExactIntCast<usize>
    + InfMul + InfSub + InfAdd + CheckNull,
    for<'a> &'a T: Sub<Output=T> + Add<&'a T, Output=T>,
    IntDistance: InfCast<T> {

    let _size = T::exact_int_cast(size)?;
    let _ddof = T::exact_int_cast(ddof)?;
    let (lower, upper) = bounds.clone();
    let _1 = T::one();
    let _2 = T::exact_int_cast(2)?;
    let range = upper.inf_sub(&lower)?;

    // check for potential overflow
    // Bound the magnitude of the sum when computing the mean
    lower.inf_mul(&_size)?;
    upper.inf_mul(&_size)?;
    // The squared difference from the mean is bounded above by range^2
    range.inf_mul(&range)?.inf_mul(&_size)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| {
            let mean = arg.iter().sum::<T>() / _size;
            arg.iter().map(|v| (v - &mean).powi(2)).sum::<T>() / (_size - _ddof)
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(
            range.inf_mul(&range)?
                .inf_mul(&_size)?
                .inf_div(&_size.neg_inf_add(&_1)?)?
                .inf_div(&_size.neg_inf_sub(&_ddof)?)?
                .inf_div(&_2)?)))
}

type CovarianceDomain<T> = SizedDomain<VectorDomain<BoundedDomain<(T, T)>>>;

pub fn make_sized_bounded_covariance<T>(
    size: usize,
    bounds_0: (T, T), bounds_1: (T, T),
    ddof: usize,
) -> Fallible<Transformation<CovarianceDomain<T>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>> where
    T: ExactIntCast<usize> + DistanceConstant<IntDistance> + Zero
    + Add<Output=T> + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T>
    + InfAdd + InfSub + CheckNull,
    for<'a> T: Div<&'a T, Output=T> + Add<&'a T, Output=T>,
    for<'a> &'a T: Sub<Output=T>,
    IntDistance: InfCast<T> {

    let _size = T::exact_int_cast(size)?;
    let _ddof = T::exact_int_cast(ddof)?;
    let _1 = T::exact_int_cast(1)?;
    let _2 = T::exact_int_cast(2)?;
    let range_0 = bounds_0.1.inf_sub(&bounds_0.0)?;
    let range_1 = bounds_1.1.inf_sub(&bounds_1.0)?;

    // check for potential overflow
    // Bound the magnitudes of the sums when computing the means
    bounds_0.0.inf_mul(&_size)?;
    bounds_0.1.inf_mul(&_size)?;
    bounds_1.0.inf_mul(&_size)?;
    bounds_1.1.inf_mul(&_size)?;
    // The squared difference from the mean is bounded above by range^2
    range_0.inf_mul(&range_1)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(
            ((bounds_0.0.clone(), bounds_1.0.clone()), (bounds_0.1.clone(), bounds_1.1.clone())))?), size),
        AllDomain::new(),
        Function::new(enclose!((_size, _ddof), move |arg: &Vec<(T, T)>| {
            let (sum_l, sum_r) = arg.iter().fold(
                (T::zero(), T::zero()),
                |(s_l, s_r), (v_l, v_r)| (s_l + v_l, s_r + v_r));
            let (mean_l, mean_r) = (sum_l / &_size, sum_r / &_size);

            arg.iter()
                .map(|(v_l, v_r)| (v_l - &mean_l) * (v_r - &mean_r))
                .sum::<T>() / (&_size - &_ddof)
        })),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(
            range_0.inf_mul(&range_1)?
                .inf_mul(&_size)?
                .inf_div(&_size.neg_inf_add(&_1)?)?
                .inf_div(&_size.neg_inf_sub(&_ddof)?)?
                .inf_div(&_2)?),
    ))
}


#[cfg(test)]
mod tests {
    use crate::error::ExplainUnwrap;

    use super::*;

    #[test]
    fn test_make_bounded_variance_hamming() {
        let arg = vec![1., 2., 3., 4., 5.];

        let transformation_sample = make_sized_bounded_variance(5, (0., 10.), 1).unwrap_test();
        let ret = transformation_sample.invoke(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.check(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_sized_bounded_variance(5, (0., 10.), 0).unwrap_test();
        let ret = transformation_pop.invoke(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.check(&1, &(100. * 4. / 25.)).unwrap_test());
    }

    #[test]
    fn test_make_bounded_covariance_hamming() {
        let arg = vec![(1., 3.), (2., 4.), (3., 5.), (4., 6.), (5., 7.)];

        let transformation_sample =  make_sized_bounded_covariance(5, (0., 2.), (10., 12.), 1).unwrap_test();
        let ret = transformation_sample.invoke(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.check(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_sized_bounded_covariance(5, (0., 2.), (10., 12.), 0).unwrap_test();
        let ret = transformation_pop.invoke(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.check(&1, &(100. * 4. / 25.)).unwrap_test());
    }
}