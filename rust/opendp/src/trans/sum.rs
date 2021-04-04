use std::cmp::Ordering;
use std::collections::Bound;
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{Div, Mul, Sub};

use crate::core::{DatasetMetric, Function, Metric, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, SymmetricDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{Abs, DistanceCast};
use crate::trans::{MakeTransformation2, MakeTransformation3};

pub struct BoundedSum<MI, MO, T> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    data: PhantomData<T>,
}

fn max<T: PartialOrd>(a: T, b: T) -> Option<T> {
    a.partial_cmp(&b).map(|o| if let Ordering::Less = o {b} else {a})
}

pub trait BoundedSumConstant<MI: Metric, MO: Metric, T> {
    fn get_stability(lower: MO::Distance, upper: MO::Distance) -> Fallible<StabilityRelation<MI, MO>>;
}

impl<MO, T> BoundedSumConstant<HammingDistance, MO, T> for BoundedSum<HammingDistance, MO, T>
    where MO: Metric<Distance=T>,
          T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast {
    fn get_stability(lower: T, upper: T) -> Fallible<StabilityRelation<HammingDistance, MO>> {
        Ok(StabilityRelation::new_from_constant(upper - lower))
    }
}

impl<MO, T> BoundedSumConstant<SymmetricDistance, MO, T> for BoundedSum<SymmetricDistance, MO, T>
    where MO: Metric<Distance=T>,
          T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast + Abs {
    fn get_stability(lower: T, upper: T) -> Fallible<StabilityRelation<SymmetricDistance, MO>> {
        max(lower.abs(), upper.abs())
            .ok_or_else(|| err!(InvalidDistance, "lower and upper must be comparable"))
            .map(StabilityRelation::new_from_constant)
    }
}

impl<MI, MO, T> MakeTransformation2<VectorDomain<IntervalDomain<T>>, AllDomain<T>, MI, MO, T, T> for BoundedSum<MI, MO, T>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric<Distance=T>,
          T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast,
          Self: BoundedSumConstant<MI, MO, T> {
    fn make2(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<IntervalDomain<T>>, AllDomain<T>, MI, MO>> {
        if lower > upper { return fallible!(MakeTransformation, "lower bound may not be greater than upper bound") }

        Ok(Transformation::new(
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
            AllDomain::new(),
            Function::new(|arg: &Vec<T>| arg.iter().cloned().sum()),
            MI::new(),
            MO::new(),
            Self::get_stability(lower, upper)?))
    }
}

impl<MO, T> MakeTransformation3<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, MO, T, T, usize> for BoundedSum<SymmetricDistance, MO, T>
    where MO: SensitivityMetric<Distance=T>,
          T: 'static + Copy + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast,
          SymmetricDistance: Metric<Distance=u32> {
    fn make3(lower: T, upper: T, length: usize) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, MO>> {
        if lower > upper { return fallible!(MakeTransformation, "lower bound may not be greater than upper bound") }

        Ok(Transformation::new(
            SizedDomain::new(VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))), length),
            AllDomain::new(),
            Function::new(|arg: &Vec<T>| arg.iter().cloned().sum()),
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
        let transformation = BoundedSum::<HammingDistance, L1Sensitivity<_>, i32>::make(0, 10).unwrap();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_l2() {
        let transformation = BoundedSum::<HammingDistance, L2Sensitivity<_>, i32>::make(0, 10).unwrap();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = 15;
        assert_eq!(ret, expected);
    }
}