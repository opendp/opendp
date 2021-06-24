use std::collections::Bound;
use std::iter::Sum;
use std::ops::{Div, Sub, Add};

use num::{Float, One, Zero, NumCast};

use crate::core::{DatasetMetric, Function, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, SymmetricDistance, AbsoluteDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::DistanceConstant;


pub trait BoundedVarianceConstant<T> {
    fn get_stability(lower: T, upper: T, length: usize, ddof: usize) -> Fallible<T>;
}

impl<T> BoundedVarianceConstant<T> for HammingDistance
    where T: Float + Sub<Output=T> + Div<Output=T> + NumCast + One {
    fn get_stability(lower: T, upper: T, length: usize, ddof: usize) -> Fallible<T> {
        let _length = num_cast!(length; T)?;
        let _1 = T::one();
        let _ddof = num_cast!(ddof; T)?;
        Ok((upper - lower).powi(2) * (_length - _1) / _length / (_length - _ddof))
    }
}

impl<T> BoundedVarianceConstant<T> for SymmetricDistance
    where T: Float + Sub<Output=T> + Div<Output=T> + NumCast + One {
    fn get_stability(lower: T, upper: T, length: usize, ddof: usize) -> Fallible<T> {
        let _length = num_cast!(length; T)?;
        let _1 = T::one();
        let _ddof = num_cast!(ddof; T)?;
        Ok((upper - lower).powi(2) * _length / (_length + _1) / (_length - _ddof))
    }
}

pub fn make_bounded_variance<MI, T>(
    lower: T, upper: T, length: usize, ddof: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, MI, AbsoluteDistance<T>>>
    where MI: DatasetMetric,
          T: DistanceConstant + Sub<Output=T> + Float + Sum<T> + for<'a> Sum<&'a T>,
          for<'a> &'a T: Sub<Output=T>,
          MI: BoundedVarianceConstant<T> {
    let _length = num_cast!(length; T)?;
    let _ddof = num_cast!(ddof; T)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))?), length),
        AllDomain::new(),
        Function::new(move |arg: &Vec<T>| {
            let mean = arg.iter().sum::<T>() / _length;
            arg.iter().map(|v| (v - &mean).powi(2)).sum::<T>() / (_length - _ddof)
        }),
        MI::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(MI::get_stability(lower, upper, length, ddof)?)))
}


pub trait BoundedCovarianceConstant<T> {
    fn get_stability_constant(lower: (T, T), upper: (T, T), length: usize, ddof: usize) -> Fallible<T>;
}

impl<T> BoundedCovarianceConstant<T> for HammingDistance
    where T: Clone + Sub<Output=T> + Div<Output=T> + NumCast + One {
    fn get_stability_constant(lower: (T, T), upper: (T, T), length: usize, ddof: usize) -> Fallible<T> {
        let _length = num_cast!(length; T)?;
        let _1 = T::one();
        let _ddof = num_cast!(ddof; T)?;
        Ok((upper.0 - lower.0) * (upper.1 - lower.1) * (_length.clone() - _1) / _length.clone() / (_length - _ddof))
    }
}

impl<T> BoundedCovarianceConstant<T> for SymmetricDistance
    where T: Clone + Sub<Output=T> + Div<Output=T> + Add<Output=T> + NumCast + One {
    fn get_stability_constant(lower: (T, T), upper: (T, T), length: usize, ddof: usize) -> Fallible<T> {
        let _length = num_cast!(length; T)?;
        let _1 = T::one();
        let _ddof = num_cast!(ddof; T)?;
        Ok((upper.0 - lower.0) * (upper.1 - lower.1) * _length.clone() / (_length.clone() + _1) / (_length - _ddof))
    }
}

type CovarianceDomain<T> = SizedDomain<VectorDomain<IntervalDomain<(T, T)>>>;

pub fn make_bounded_covariance<MI, T>(
    lower: (T, T),
    upper: (T, T),
    length: usize, ddof: usize
) -> Fallible<Transformation<CovarianceDomain<T>, AllDomain<T>, MI, AbsoluteDistance<T>>>
    where MI: DatasetMetric,
          T: DistanceConstant + Sub<Output=T> + Sum<T> + Zero,
          for <'a> T: Div<&'a T, Output=T> + Add<&'a T, Output=T>,
          for<'a> &'a T: Sub<Output=T>,
          MI: BoundedCovarianceConstant<T> {

    let _length = num_cast!(length; T)?;
    let _ddof = num_cast!(ddof; T)?;


    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))?), length),
        AllDomain::new(),
        Function::new(move |arg: &Vec<(T, T)>| {
            let (sum_l, sum_r) = arg.iter().fold(
                (T::zero(), T::zero()),
                |(s_l, s_r), (v_l, v_r)| (s_l + v_l, s_r + v_r));
            let (mean_l, mean_r) = (sum_l / &_length, sum_r / &_length);

            arg.iter()
                .map(|(v_l, v_r)| (v_l - &mean_l) * (v_r - &mean_r))
                .sum::<T>() / (&_length - &_ddof)
        }),
        MI::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(MI::get_stability_constant(lower, upper, length, ddof)?)))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ExplainUnwrap;

    #[test]
    fn test_make_bounded_variance_hamming() {
        let arg = vec![1., 2., 3., 4., 5.];

        let transformation_sample = make_bounded_variance::<HammingDistance, f64>(0., 10., 5, 1).unwrap_test();
        let ret = transformation_sample.function.eval(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.stability_relation.eval(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_bounded_variance::<HammingDistance, f64>(0., 10., 5, 0).unwrap_test();
        let ret = transformation_pop.function.eval(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.stability_relation.eval(&1, &(100. * 4. / 25.)).unwrap_test());
    }

    #[test]
    fn test_make_bounded_covariance_hamming() {
        let arg = vec![(1., 3.), (2., 4.), (3., 5.), (4., 6.), (5., 7.)];

        let transformation_sample =  make_bounded_covariance::<HammingDistance, f64>((0., 2.), (10., 12.), 5, 1).unwrap_test();
        let ret = transformation_sample.function.eval(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.stability_relation.eval(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_bounded_covariance::<HammingDistance, f64>((0., 2.), (10., 12.), 5, 0).unwrap_test();
        let ret = transformation_pop.function.eval(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.stability_relation.eval(&1, &(100. * 4. / 25.)).unwrap_test());
    }
}