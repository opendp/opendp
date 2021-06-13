use std::cmp::Ordering;
use std::collections::Bound;
use std::iter::Sum;
use std::ops::Sub;

use crate::core::{DatasetMetric, Function, Metric, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, SymmetricDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{Abs, DistanceCast, DistanceConstant};

fn max<T: PartialOrd>(a: T, b: T) -> Option<T> {
    a.partial_cmp(&b).map(|o| if let Ordering::Less = o {b} else {a})
}

pub trait BoundedSumConstant<MI: Metric, MO: Metric> {
    fn get_stability_constant(lower: MO::Distance, upper: MO::Distance) -> Fallible<MO::Distance>;
}

impl<MO: Metric> BoundedSumConstant<HammingDistance, MO> for (HammingDistance, MO)
    where MO::Distance: 'static + Sub<Output=MO::Distance> {
    fn get_stability_constant(lower: MO::Distance, upper: MO::Distance) -> Fallible<MO::Distance> {
        Ok(upper - lower)
    }
}

impl<MO: Metric> BoundedSumConstant<SymmetricDistance, MO> for (SymmetricDistance, MO)
    where MO::Distance: 'static + PartialOrd + Abs {
    fn get_stability_constant(lower: MO::Distance, upper: MO::Distance) -> Fallible<MO::Distance> {
        max(lower.abs(), upper.abs())
            .ok_or_else(|| err!(InvalidDistance, "lower and upper must be comparable"))
    }
}

pub fn make_bounded_sum<MI, MO>(
    lower: MO::Distance, upper: MO::Distance
) -> Fallible<Transformation<VectorDomain<IntervalDomain<MO::Distance>>, AllDomain<MO::Distance>, MI, MO>>
    where MI: DatasetMetric,
          MO: SensitivityMetric,
          MO::Distance: DistanceConstant + Sub<Output=MO::Distance>,
          for <'a> MO::Distance: Sum<&'a MO::Distance>,
          (MI, MO): BoundedSumConstant<MI, MO> {

    Ok(Transformation::new(
        VectorDomain::new(IntervalDomain::new(
            Bound::Included(lower.clone()), Bound::Included(upper.clone()))?),
        AllDomain::new(),
        Function::new(|arg: &Vec<MO::Distance>| arg.iter().sum()),
        MI::default(),
        MO::default(),
        StabilityRelation::new_from_constant(<(MI, MO)>::get_stability_constant(lower, upper)?)))
}


pub fn make_bounded_sum_n<MO>(
    lower: MO::Distance, upper: MO::Distance, length: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<MO::Distance>>>, AllDomain<MO::Distance>, SymmetricDistance, MO>>
    where MO: SensitivityMetric,
          MO::Distance: DistanceConstant + Sub<Output=MO::Distance>,
          for <'a> MO::Distance: Sum<&'a MO::Distance> {

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(IntervalDomain::new(
            Bound::Included(lower.clone()), Bound::Included(upper.clone()))?), length),
        AllDomain::new(),
        Function::new(|arg: &Vec<MO::Distance>| arg.iter().sum()),
        SymmetricDistance::default(),
        MO::default(),
        // d_out >= d_in * (M - m) / 2
        StabilityRelation::new_from_constant((upper - lower) / MO::Distance::distance_cast(2)?)))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dist::{L1Sensitivity, L2Sensitivity};

    #[test]
    fn test_make_bounded_sum_l1() {
        let transformation = make_bounded_sum::<HammingDistance, L1Sensitivity<i32>>(0, 10).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_l2() {
        let transformation = make_bounded_sum::<HammingDistance, L2Sensitivity<i32>>(0, 10).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_n() {
        let transformation = make_bounded_sum_n::<L2Sensitivity<i32>>(0, 10, 5).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }
}