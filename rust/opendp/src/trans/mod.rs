//! Various implementations of Transformations.
//!
//! The different [`Transformation`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Transformation` does.

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{Bound, Div, Mul, Sub};

use crate::core::{DatasetMetric, Domain, Function, Metric, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, SymmetricDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{Abs, DistanceCast};
pub use crate::trans::dataframe::*;

pub mod dataframe;
pub mod manipulation;

// Trait for all constructors, can have different implementations depending on concrete types of Domains and/or Metrics
pub trait MakeTransformation0<DI: Domain, DO: Domain, MI: Metric, MO: Metric> {
    fn make() -> Fallible<crate::core::Transformation<DI, DO, MI, MO>> {
        Self::make0()
    }
    fn make0() -> Fallible<crate::core::Transformation<DI, DO, MI, MO>>;
}

pub trait MakeTransformation1<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1> {
    fn make(param1: P1) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>> {
        Self::make1(param1)
    }
    fn make1(param1: P1) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>>;
}

pub trait MakeTransformation2<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1, P2> {
    fn make(param1: P1, param2: P2) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>> {
        Self::make2(param1, param2)
    }
    fn make2(param1: P1, param2: P2) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>>;
}

pub trait MakeTransformation3<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1, P2, P3> {
    fn make(param1: P1, param2: P2, param3: P3) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>> {
        Self::make3(param1, param2, param3)
    }
    fn make3(param1: P1, param2: P2, param3: P3) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>>;
}

pub trait MakeTransformation4<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1, P2, P3, P4> {
    fn make(param1: P1, param2: P2, param3: P3, param4: P4) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>> {
        Self::make4(param1, param2, param3, param4)
    }
    fn make4(param1: P1, param2: P2, param3: P3, param4: P4) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>>;
}

pub struct BoundedSum<MI, MO, T> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    data: PhantomData<T>,
}

fn max<T: PartialOrd>(a: T, b: T) -> Option<T> {
    a.partial_cmp(&b).map(|o| if let Ordering::Less = o {b} else {a})
}

pub trait BoundedSumStability<MI: Metric, MO: Metric, T> {
    fn get_stability(lower: MO::Distance, upper: MO::Distance) -> Fallible<StabilityRelation<MI, MO>>;
}

impl<MO, T> BoundedSumStability<HammingDistance, MO, T> for BoundedSum<HammingDistance, MO, T>
    where MO: Metric<Distance=T>,
          T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast {
    fn get_stability(lower: T, upper: T) -> Fallible<StabilityRelation<HammingDistance, MO>> {
        Ok(StabilityRelation::new_from_constant(upper - lower))
    }
}
impl<MO, T> BoundedSumStability<SymmetricDistance, MO, T> for BoundedSum<SymmetricDistance, MO, T>
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
          BoundedSum<MI, MO, T>: BoundedSumStability<MI, MO, T> {
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

pub struct Count<MI, MO, T> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    data: PhantomData<T>,
}

impl<MI, MO, T> MakeTransformation0<VectorDomain<AllDomain<T>>, AllDomain<u32>, MI, MO> for Count<MI, MO, T>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric<Distance=u32> {
    fn make0() -> Fallible<Transformation<VectorDomain<AllDomain<T>>, AllDomain<u32>, MI, MO>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            AllDomain::new(),
            // min(arg.len(), u32::MAX)
            Function::new(move |arg: &Vec<T>| u32::try_from(arg.len()).unwrap_or(u32::MAX)),
            MI::new(),
            MO::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}


#[cfg(test)]
mod tests {
    use crate::dist::{L1Sensitivity, L2Sensitivity};
    use crate::trans::manipulation::{Clamp, Identity};

    use super::*;

    #[test]
    fn test_identity() {
        let identity = Identity::make(AllDomain::new(), HammingDistance).unwrap();
        let arg = 99;
        let ret = identity.function.eval(&arg).unwrap();
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_make_split_lines() {
        let transformation = SplitLines::<HammingDistance>::make().unwrap();
        let arg = "ant\nbat\ncat\n".to_owned();
        let ret = transformation.function.eval(&arg).unwrap();
        assert_eq!(ret, vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]);
    }

    #[test]
    fn test_make_parse_series() {
        let transformation = ParseSeries::<i32, HammingDistance>::make(true).unwrap();
        let arg = vec!["1".to_owned(), "2".to_owned(), "3".to_owned(), "foo".to_owned()];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = vec![1, 2, 3, 0];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_split_records() {
        let transformation = SplitRecords::<HammingDistance>::make(None).unwrap();
        let arg = vec!["ant, foo".to_owned(), "bat, bar".to_owned(), "cat, baz".to_owned()];
        let ret = transformation.function.eval(&arg).unwrap();
        assert_eq!(ret, vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ]);
    }

    #[test]
    fn test_make_clamp() {
        let transformation = Clamp::<HammingDistance, Vec<i32>, u32>::make(0, 10).unwrap();
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }

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

    #[test]
    fn test_make_count_l1() {
        let transformation = Count::<SymmetricDistance, L1Sensitivity<_>, i32>::make().unwrap();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_l2() {
        let transformation = Count::<SymmetricDistance, L2Sensitivity<_>, i32>::make().unwrap();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = 5;
        assert_eq!(ret, expected);
    }
}
