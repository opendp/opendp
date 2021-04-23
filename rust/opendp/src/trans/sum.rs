use std::cmp::Ordering;
use std::collections::Bound;
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{Div, Mul, Sub};

use crate::core::{DatasetMetric, Function, Metric, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, SymmetricDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{Abs, DistanceCast};
use crate::trans::{MakeTransformation2, MakeTransformation3};

pub struct BoundedSum<MI, MO> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>
}

fn max<T: PartialOrd>(a: T, b: T) -> Option<T> {
    a.partial_cmp(&b).map(|o| if let Ordering::Less = o {b} else {a})
}

pub trait BoundedSumConstant<MI: Metric, MO: Metric> {
    fn get_stability_constant(lower: MO::Distance, upper: MO::Distance) -> Fallible<MO::Distance>;
}

impl<MO: Metric<Distance=T>, T> BoundedSumConstant<HammingDistance, MO> for BoundedSum<HammingDistance, MO>
    where T: 'static + Sub<Output=T> {
    fn get_stability_constant(lower: T, upper: T) -> Fallible<T> {
        Ok(upper - lower)
    }
}

impl<MO: Metric<Distance=T>, T> BoundedSumConstant<SymmetricDistance, MO> for BoundedSum<SymmetricDistance, MO>
    where T: 'static + PartialOrd + Abs {
    fn get_stability_constant(lower: T, upper: T) -> Fallible<T> {
        max(lower.abs(), upper.abs())
            .ok_or_else(|| err!(InvalidDistance, "lower and upper must be comparable"))
    }
}

impl<MI, MO, T> MakeTransformation2<VectorDomain<IntervalDomain<T>>, AllDomain<T>, MI, MO, T, T> for BoundedSum<MI, MO>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric<Distance=T>,
          T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + DistanceCast,
          for <'a> T: Sum<&'a T>,
          Self: BoundedSumConstant<MI, MO> {
    fn make2(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<IntervalDomain<T>>, AllDomain<T>, MI, MO>> {
        if lower > upper { return fallible!(MakeTransformation, "lower bound may not be greater than upper bound") }

        Ok(Transformation::new(
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
            AllDomain::new(),
            Function::new(|arg: &Vec<T>| arg.iter().sum()),
            MI::new(),
            MO::new(),
            StabilityRelation::new_from_constant(Self::get_stability_constant(lower, upper)?)))
    }
}

impl<MO, T> MakeTransformation3<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, MO, T, T, usize> for BoundedSum<SymmetricDistance, MO>
    where MO: SensitivityMetric<Distance=T>,
          T: 'static + Copy + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + DistanceCast,
          for <'a> T: Sum<&'a T> {
    fn make3(lower: T, upper: T, length: usize) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, MO>> {
        if lower > upper { return fallible!(MakeTransformation, "lower bound may not be greater than upper bound") }

        Ok(Transformation::new(
            SizedDomain::new(VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))), length),
            AllDomain::new(),
            Function::new(|arg: &Vec<T>| arg.iter().sum()),
            SymmetricDistance::new(),
            MO::new(),
            // d_out >= d_in * (M - m) / 2
            StabilityRelation::new_from_constant((upper - lower) / T::distance_cast(2)?)))
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::dist::{L1Sensitivity, L2Sensitivity};

    #[test]
    fn test_make_bounded_sum_l1() {
        let transformation = BoundedSum::<HammingDistance, L1Sensitivity<i32>>::make(0, 10).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_l2() {
        let transformation = BoundedSum::<HammingDistance, L2Sensitivity<i32>>::make(0, 10).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }
}