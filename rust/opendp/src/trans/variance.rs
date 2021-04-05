use std::collections::Bound;
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{Div, Mul, Sub, Add};

use num::{Float, One, Zero, NumCast};

use crate::core::{DatasetMetric, Function, Metric, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, SymmetricDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::DistanceCast;
use crate::trans::MakeTransformation4;

pub struct BoundedVariance<MI, MO> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
}


pub trait BoundedVarianceConstant<MI: Metric, MO: Metric> {
    fn get_stability(lower: MO::Distance, upper: MO::Distance, length: usize, ddof: usize) -> Fallible<MO::Distance>;
}

impl<MO: Metric<Distance=T>, T> BoundedVarianceConstant<HammingDistance, MO> for BoundedVariance<HammingDistance, MO>
    where T: Float + Sub<Output=T> + Div<Output=T> + NumCast + One {
    fn get_stability(lower: T, upper: T, length: usize, ddof: usize) -> Fallible<T> {
        let n = T::from(length).ok_or_else(|| err!(FailedCast))?;
        let _1 = T::one();
        let ddof = T::from(ddof).ok_or_else(|| err!(FailedCast))?;
        Ok((upper - lower).powi(2) * (n - _1) / n / (n - ddof))
    }
}

impl<MO: Metric<Distance=T>, T> BoundedVarianceConstant<SymmetricDistance, MO> for BoundedVariance<SymmetricDistance, MO>
    where T: Float + Sub<Output=T> + Div<Output=T> + NumCast + One {
    fn get_stability(lower: T, upper: T, length: usize, ddof: usize) -> Fallible<T> {
        let n = T::from(length).ok_or_else(|| err!(FailedCast))?;
        let _1 = T::one();
        let ddof = T::from(ddof).ok_or_else(|| err!(FailedCast))?;
        Ok((upper - lower).powi(2) * n / (n + _1) / (n - ddof))
    }
}


impl<MI, MO, T> MakeTransformation4<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, MI, MO, T, T, usize, usize> for BoundedVariance<MI, MO>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric<Distance=T>,
          T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast + Float,
          Self: BoundedVarianceConstant<MI, MO> {
    fn make4(lower: T, upper: T, length: usize, ddof: usize) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, MI, MO>> {
        if lower > upper { return fallible!(MakeTransformation, "lower bound may not be greater than upper bound"); }
        let _length = T::from(length).ok_or_else(|| err!(FailedCast))?;
        let _ddof = T::from(length).ok_or_else(|| err!(FailedCast))?;

        Ok(Transformation::new(
            SizedDomain::new(VectorDomain::new(
                IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))), length),
            AllDomain::new(),
            Function::new(move |arg: &Vec<T>| {
                let mean = arg.iter().cloned().sum::<T>() / _length;
                arg.iter().cloned().map(|v| (v - mean).powi(2)).sum::<T>() / (_length - _ddof)
            }),
            MI::new(),
            MO::new(),
            StabilityRelation::new_from_constant(Self::get_stability(lower, upper, length, ddof)?)))
    }
}


pub struct BoundedCovariance<MI, MO> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
}


pub trait BoundedCovarianceConstant<MI: Metric, MO: Metric> {
    fn get_stability_constant(lower: (MO::Distance, MO::Distance), upper: (MO::Distance, MO::Distance), length: usize, ddof: usize) -> Fallible<MO::Distance>;
}

impl<MO: Metric<Distance=T>, T> BoundedCovarianceConstant<HammingDistance, MO> for BoundedCovariance<HammingDistance, MO>
    where T: Clone + Sub<Output=T> + Div<Output=T> + NumCast + One {
    fn get_stability_constant(lower: (T, T), upper: (T, T), length: usize, ddof: usize) -> Fallible<T> {
        let n = T::from(length).ok_or_else(|| err!(FailedCast))?;
        let _1 = T::one();
        let ddof = T::from(ddof).ok_or_else(|| err!(FailedCast))?;
        Ok((upper.0 - lower.0) * (upper.1 - lower.1) * (n.clone() - _1) / n.clone() / (n - ddof))
    }
}

impl<MO: Metric<Distance=T>, T> BoundedCovarianceConstant<SymmetricDistance, MO> for BoundedCovariance<SymmetricDistance, MO>
    where T: Clone + Sub<Output=T> + Div<Output=T> + Add<Output=T> + NumCast + One {
    fn get_stability_constant(lower: (T, T), upper: (T, T), length: usize, ddof: usize) -> Fallible<T> {
        let n = T::from(length).ok_or_else(|| err!(FailedCast))?;
        let _1 = T::one();
        let ddof = T::from(ddof).ok_or_else(|| err!(FailedCast))?;
        Ok((upper.0 - lower.0) * (upper.1 - lower.1) * n.clone() / (n.clone() + _1) / (n - ddof))
    }
}

type CovarianceDomain<T> = SizedDomain<VectorDomain<IntervalDomain<(T, T)>>>;

impl<MI, MO, T> MakeTransformation4<CovarianceDomain<T>, AllDomain<T>, MI, MO, (T, T), (T, T), usize, usize> for BoundedCovariance<MI, MO>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric<Distance=T>,
          T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast + Zero,
          for<'a> &'a T: Sub<Output=T>,
          Self: BoundedCovarianceConstant<MI, MO> {
    fn make4(lower: (T, T), upper: (T, T), length: usize, ddof: usize) -> Fallible<Transformation<CovarianceDomain<T>, AllDomain<T>, MI, MO>> {
        if lower > upper { return fallible!(MakeTransformation, "lower bound may not be greater than upper bound"); }
        let n = T::from(length).ok_or_else(|| err!(FailedCast))?;
        let _ddof = T::from(ddof).ok_or_else(|| err!(FailedCast))?;

        Ok(Transformation::new(
            SizedDomain::new(VectorDomain::new(
                IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))), length),
            AllDomain::new(),
            Function::new(move |arg: &Vec<(T, T)>| {
                let (sum_l, sum_r) = arg.clone().into_iter().fold(
                    (T::zero(), T::zero()),
                    |(s_l, s_r), (v_l, v_r)| (s_l + v_l, s_r + v_r));
                let (mean_l, mean_r) = (sum_l / n, sum_r / n);

                arg.iter()
                    .map(|(v_l, v_r)| (v_l - &mean_l) * (v_r - &mean_r))
                    .sum::<T>() / (n - _ddof)
            }),
            MI::new(),
            MO::new(),
            StabilityRelation::new_from_constant(Self::get_stability_constant(lower, upper, length, ddof)?)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dist::{L1Sensitivity};
    use crate::error::ExplainUnwrap;

    #[test]
    fn test_make_bounded_variance_hamming() {
        let arg = vec![1., 2., 3., 4., 5.];

        let transformation_sample = BoundedVariance::<HammingDistance, L1Sensitivity<f64>>::make(0., 10., 5, 1).unwrap_test();
        let ret = transformation_sample.function.eval(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);

        let transformation_pop = BoundedVariance::<HammingDistance, L1Sensitivity<f64>>::make(0., 10., 5, 0).unwrap_test();
        let ret = transformation_pop.function.eval(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);

        assert!(transformation_sample.stability_relation.eval(&1, &2.).unwrap_test())
    }
}