use std::collections::Bound;
use std::iter::Sum;
use std::ops::{Div, Mul, Sub, Add};

use num::{Float, One, Zero, NumCast};

use crate::core::{DatasetMetric, Function, Metric, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, SymmetricDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::DistanceCast;


pub trait BoundedVarianceConstant<MI: Metric, MO: Metric> {
    fn get_stability(lower: MO::Distance, upper: MO::Distance, length: usize, ddof: usize) -> Fallible<MO::Distance>;
}

impl<MO: Metric> BoundedVarianceConstant<HammingDistance, MO> for (HammingDistance, MO)
    where MO::Distance: Float + Sub<Output=MO::Distance> + Div<Output=MO::Distance> + NumCast + One {
    fn get_stability(lower: MO::Distance, upper: MO::Distance, length: usize, ddof: usize) -> Fallible<MO::Distance> {
        let _length = c!(length; MO::Distance)?;
        let _1 = MO::Distance::one();
        let _ddof = c!(ddof; MO::Distance)?;
        Ok((upper - lower).powi(2) * (_length - _1) / _length / (_length - _ddof))
    }
}

impl<MO: Metric> BoundedVarianceConstant<SymmetricDistance, MO> for (SymmetricDistance, MO)
    where MO::Distance: Float + Sub<Output=MO::Distance> + Div<Output=MO::Distance> + NumCast + One {
    fn get_stability(lower: MO::Distance, upper: MO::Distance, length: usize, ddof: usize) -> Fallible<MO::Distance> {
        let _length = c!(length; MO::Distance)?;
        let _1 = MO::Distance::one();
        let _ddof = c!(ddof; MO::Distance)?;
        Ok((upper - lower).powi(2) * _length / (_length + _1) / (_length - _ddof))
    }
}

pub fn make_bounded_variance<MI, MO>(
    lower: MO::Distance, upper: MO::Distance, length: usize, ddof: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<MO::Distance>>>, AllDomain<MO::Distance>, MI, MO>>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric,
          MO::Distance: 'static + Clone + PartialOrd + Sub<Output=MO::Distance> + Mul<Output=MO::Distance> + Div<Output=MO::Distance> + DistanceCast + Float + Sum<MO::Distance> + for<'a> Sum<&'a MO::Distance>,
          for<'a> &'a MO::Distance: Sub<Output=MO::Distance>,
          (MI, MO): BoundedVarianceConstant<MI, MO> {
    if lower > upper { return fallible!(MakeTransformation, "lower bound may not be greater than upper bound"); }
    let _length = c!(length; MO::Distance)?;
    let _ddof = c!(ddof; MO::Distance)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))), length),
        AllDomain::new(),
        Function::new(move |arg: &Vec<MO::Distance>| {
            let mean = arg.iter().sum::<MO::Distance>() / _length;
            arg.iter().map(|v| (v - &mean).powi(2)).sum::<MO::Distance>() / (_length - _ddof)
        }),
        MI::default(),
        MO::default(),
        StabilityRelation::new_from_constant(<(MI, MO)>::get_stability(lower, upper, length, ddof)?)))
}


pub trait BoundedCovarianceConstant<MI: Metric, MO: Metric> {
    fn get_stability_constant(lower: (MO::Distance, MO::Distance), upper: (MO::Distance, MO::Distance), length: usize, ddof: usize) -> Fallible<MO::Distance>;
}

impl<MO: Metric> BoundedCovarianceConstant<HammingDistance, MO> for (HammingDistance, MO)
    where MO::Distance: Clone + Sub<Output=MO::Distance> + Div<Output=MO::Distance> + NumCast + One {
    fn get_stability_constant(lower: (MO::Distance, MO::Distance), upper: (MO::Distance, MO::Distance), length: usize, ddof: usize) -> Fallible<MO::Distance> {
        let _length = c!(length; MO::Distance)?;
        let _1 = MO::Distance::one();
        let _ddof = c!(ddof; MO::Distance)?;
        Ok((upper.0 - lower.0) * (upper.1 - lower.1) * (_length.clone() - _1) / _length.clone() / (_length - _ddof))
    }
}

impl<MO: Metric> BoundedCovarianceConstant<SymmetricDistance, MO> for (SymmetricDistance, MO)
    where MO::Distance: Clone + Sub<Output=MO::Distance> + Div<Output=MO::Distance> + Add<Output=MO::Distance> + NumCast + One {
    fn get_stability_constant(lower: (MO::Distance, MO::Distance), upper: (MO::Distance, MO::Distance), length: usize, ddof: usize) -> Fallible<MO::Distance> {
        let _length = c!(length; MO::Distance)?;
        let _1 = MO::Distance::one();
        let _ddof = c!(ddof; MO::Distance)?;
        Ok((upper.0 - lower.0) * (upper.1 - lower.1) * _length.clone() / (_length.clone() + _1) / (_length - _ddof))
    }
}

type CovarianceDomain<T> = SizedDomain<VectorDomain<IntervalDomain<(T, T)>>>;

pub fn make_bounded_covariance<MI, MO>(
    lower: (MO::Distance, MO::Distance),
    upper: (MO::Distance, MO::Distance),
    length: usize, ddof: usize
) -> Fallible<Transformation<CovarianceDomain<MO::Distance>, AllDomain<MO::Distance>, MI, MO>>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric,
          MO::Distance: 'static + Clone + PartialOrd + Sub<Output=MO::Distance> + Mul<Output=MO::Distance> + Div<Output=MO::Distance> + Sum<MO::Distance> + DistanceCast + Zero,
          for <'a> MO::Distance: Div<&'a MO::Distance, Output=MO::Distance> + Add<&'a MO::Distance, Output=MO::Distance>,
          for<'a> &'a MO::Distance: Sub<Output=MO::Distance>,
          (MI, MO): BoundedCovarianceConstant<MI, MO> {

    if lower > upper { return fallible!(MakeTransformation, "lower bound may not be greater than upper bound"); }
    let _length = c!(length; MO::Distance)?;
    let _ddof = c!(ddof; MO::Distance)?;


    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))), length),
        AllDomain::new(),
        Function::new(move |arg: &Vec<(MO::Distance, MO::Distance)>| {
            let (sum_l, sum_r) = arg.iter().fold(
                (MO::Distance::zero(), MO::Distance::zero()),
                |(s_l, s_r), (v_l, v_r)| (s_l + v_l, s_r + v_r));
            let (mean_l, mean_r) = (sum_l / &_length, sum_r / &_length);

            arg.iter()
                .map(|(v_l, v_r)| (v_l - &mean_l) * (v_r - &mean_r))
                .sum::<MO::Distance>() / (&_length - &_ddof)
        }),
        MI::default(),
        MO::default(),
        StabilityRelation::new_from_constant(<(MI, MO)>::get_stability_constant(lower, upper, length, ddof)?)))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dist::{L1Sensitivity};
    use crate::error::ExplainUnwrap;

    #[test]
    fn test_make_bounded_variance_hamming() {
        let arg = vec![1., 2., 3., 4., 5.];

        let transformation_sample = make_bounded_variance::<HammingDistance, L1Sensitivity<f64>>(0., 10., 5, 1).unwrap_test();
        let ret = transformation_sample.function.eval(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.stability_relation.eval(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_bounded_variance::<HammingDistance, L1Sensitivity<f64>>(0., 10., 5, 0).unwrap_test();
        let ret = transformation_pop.function.eval(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.stability_relation.eval(&1, &(100. * 4. / 25.)).unwrap_test());
    }

    #[test]
    fn test_make_bounded_covariance_hamming() {
        let arg = vec![(1., 3.), (2., 4.), (3., 5.), (4., 6.), (5., 7.)];

        let transformation_sample =  make_bounded_covariance::<HammingDistance, L1Sensitivity<f64>>((0., 2.), (10., 12.), 5, 1).unwrap_test();
        let ret = transformation_sample.function.eval(&arg).unwrap_test();
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.stability_relation.eval(&1, &(100. / 5.)).unwrap_test());

        let transformation_pop = make_bounded_covariance::<HammingDistance, L1Sensitivity<f64>>((0., 2.), (10., 12.), 5, 0).unwrap_test();
        let ret = transformation_pop.function.eval(&arg).unwrap_test();
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.stability_relation.eval(&1, &(100. * 4. / 25.)).unwrap_test());
    }
}