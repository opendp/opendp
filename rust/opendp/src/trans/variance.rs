use std::collections::Bound;
use std::iter::Sum;
use std::ops::{Div, Sub, Add};

use num::{Float, One, Zero};

use crate::core::{Function, StabilityRelation, Transformation};
use crate::dist::{SymmetricDistance, AbsoluteDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::DistanceConstant;


pub fn make_bounded_variance<T>(
    lower: T, upper: T, length: usize, ddof: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant + Float + One + Sub<Output=T> + Div<Output=T> + Sum<T> + for<'a> Sum<&'a T>,
          for<'a> &'a T: Sub<Output=T> + Add<&'a T, Output=T> {
    let _length = num_cast!(length; T)?;
    let _ddof = num_cast!(ddof; T)?;
    let _1 = T::one();
    let _2 = &_1 + &_1;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))?), length),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| {
            let mean = arg.iter().sum::<T>() / _length;
            arg.iter().map(|v| (v - &mean).powi(2)).sum::<T>() / (_length - _ddof)
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(
            (upper - lower).powi(2)
                * _length
                / (_length + _1)
                / (_length - _ddof)
                / _2)))
}

type CovarianceDomain<T> = SizedDomain<VectorDomain<IntervalDomain<(T, T)>>>;

pub fn make_bounded_covariance<T>(
    lower: (T, T),
    upper: (T, T),
    length: usize, ddof: usize
) -> Fallible<Transformation<CovarianceDomain<T>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant + Zero + One + Sub<Output=T> + Div<Output=T> + Add<Output=T> + Sum<T>,
          for <'a> T: Div<&'a T, Output=T> + Add<&'a T, Output=T>,
          for<'a> &'a T: Sub<Output=T> {

    let _length = num_cast!(length; T)?;
    let _ddof = num_cast!(ddof; T)?;
    let _1 = T::one();
    let _2 = _1.clone() + &_1;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))?), length),
        AllDomain::new(),
        Function::new(enclose!((_length, _ddof), move |arg: &Vec<(T, T)>| {
            let (sum_l, sum_r) = arg.iter().fold(
                (T::zero(), T::zero()),
                |(s_l, s_r), (v_l, v_r)| (s_l + v_l, s_r + v_r));
            let (mean_l, mean_r) = (sum_l / &_length, sum_r / &_length);

            arg.iter()
                .map(|(v_l, v_r)| (v_l - &mean_l) * (v_r - &mean_r))
                .sum::<T>() / (&_length - &_ddof)
        })),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(
            (upper.0 - lower.0) * (upper.1 - lower.1)
                * _length.clone()
                / (_length.clone() + _1)
                / (_length - _ddof)
                / _2)))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ExplainUnwrap;

    #[test]
    fn test_make_bounded_variance_hamming() {
        let arg = vec![1., 2., 3., 4., 5.];

        let transformation_sample = make_bounded_variance(0., 10., 5, 1).unwrap_test();
        let ret = transformation_sample.function.eval(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.stability_relation.eval(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_bounded_variance(0., 10., 5, 0).unwrap_test();
        let ret = transformation_pop.function.eval(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.stability_relation.eval(&1, &(100. * 4. / 25.)).unwrap_test());
    }

    #[test]
    fn test_make_bounded_covariance_hamming() {
        let arg = vec![(1., 3.), (2., 4.), (3., 5.), (4., 6.), (5., 7.)];

        let transformation_sample =  make_bounded_covariance((0., 2.), (10., 12.), 5, 1).unwrap_test();
        let ret = transformation_sample.function.eval(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.stability_relation.eval(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_bounded_covariance((0., 2.), (10., 12.), 5, 0).unwrap_test();
        let ret = transformation_pop.function.eval(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.stability_relation.eval(&1, &(100. * 4. / 25.)).unwrap_test());
    }
}