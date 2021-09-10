use std::iter::Sum;
use std::ops::{Div, Sub, Add};

use num::{Float, One, Zero};

use crate::core::{Function, StabilityRelation, Transformation};
use crate::dist::{SymmetricDistance, AbsoluteDistance, IntDistance};
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{DistanceConstant, ExactIntCast, InfCast, CheckedMul, CheckNull};


pub fn make_sized_bounded_variance<T>(
    size: usize, bounds: (T, T), ddof: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<BoundedDomain<T>>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant<IntDistance> + Float + One + Sub<Output=T> + Div<Output=T> + Sum<T> + for<'a> Sum<&'a T> + ExactIntCast<usize> + CheckedMul + CheckNull,
          for<'a> &'a T: Sub<Output=T> + Add<&'a T, Output=T>,
          IntDistance: InfCast<T> {
    let _size = T::exact_int_cast(size)?;
    let _ddof = T::exact_int_cast(ddof)?;
    let (lower, upper) = bounds.clone();
    let _1 = T::one();
    let _2 = &_1 + &_1;

    let range = (&upper - &lower) / _2.clone();
    if range.clone().checked_mul(&range).is_none() {
        return fallible!(MakeTransformation, "Detected potential for overflow when computing function.")
    }

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
            (upper - lower).powi(2)
                * _size
                / (_size + _1)
                / (_size - _ddof)
                / _2)))
}

type CovarianceDomain<T> = SizedDomain<VectorDomain<BoundedDomain<(T, T)>>>;

pub fn make_sized_bounded_covariance<T>(
    size: usize,
    bounds_0: (T, T), bounds_1: (T, T),
    ddof: usize
) -> Fallible<Transformation<CovarianceDomain<T>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: ExactIntCast<usize> + DistanceConstant<IntDistance> + Zero + One + Sub<Output=T> + Div<Output=T> + Add<Output=T> + Sum<T> + CheckedMul + CheckNull,
          for <'a> T: Div<&'a T, Output=T> + Add<&'a T, Output=T>,
          for<'a> &'a T: Sub<Output=T>,
          IntDistance: InfCast<T> {

    let _size = T::exact_int_cast(size)?;
    let _ddof = T::exact_int_cast(ddof)?;
    let _1 = T::one();
    let _2 = _1.clone() + &_1;

    if ((&bounds_0.1 - &bounds_0.0) / _2.clone()).checked_mul(
        &((&bounds_1.1 - &bounds_1.0) / _2.clone())).is_none() {
        return fallible!(MakeTransformation, "Detected potential for overflow when computing function.")
    }

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
            (bounds_0.1 - bounds_0.0) * (bounds_1.1 - bounds_1.0)
                * _size.clone()
                / (_size.clone() + _1)
                / (_size - _ddof)
                / _2)))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ExplainUnwrap;

    #[test]
    fn test_make_bounded_variance_hamming() {
        let arg = vec![1., 2., 3., 4., 5.];

        let transformation_sample = make_sized_bounded_variance(5, (0., 10.), 1).unwrap_test();
        let ret = transformation_sample.function.eval(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.stability_relation.eval(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_sized_bounded_variance(5, (0., 10.), 0).unwrap_test();
        let ret = transformation_pop.function.eval(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.stability_relation.eval(&1, &(100. * 4. / 25.)).unwrap_test());
    }

    #[test]
    fn test_make_bounded_covariance_hamming() {
        let arg = vec![(1., 3.), (2., 4.), (3., 5.), (4., 6.), (5., 7.)];

        let transformation_sample =  make_sized_bounded_covariance(5, (0., 2.), (10., 12.), 1).unwrap_test();
        let ret = transformation_sample.function.eval(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.stability_relation.eval(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_sized_bounded_covariance(5, (0., 2.), (10., 12.), 0).unwrap_test();
        let ret = transformation_pop.function.eval(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.stability_relation.eval(&1, &(100. * 4. / 25.)).unwrap_test());
    }
}