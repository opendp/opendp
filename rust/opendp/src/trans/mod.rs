//! Various implementations of Transformations.
//!
//! The different [`Transformation`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Transformation` does.

use std::cmp::Ordering;
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{Bound, Div, Mul, Sub};

use num::{One, Signed};

use crate::core::{DatasetMetric, Domain, Metric, SensitivityMetric, StabilityRelation, Transformation, Function};
use crate::dist::{HammingDistance, SymmetricDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::{Error, Fallible};
use crate::traits::DistanceCast;
pub use crate::trans::dataframe::*;

pub mod dataframe;

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

/// Constructs a [`Transformation`] representing the identity function.
pub struct Identity;

impl<D, T, M, Q> MakeTransformation2<D, D, M, M, D, M> for Identity
    where D: Domain<Carrier=T>, T: Clone,
          M: Metric<Distance=Q>, Q: 'static + Clone + Div<Output=Q> + Mul<Output=Q> + PartialOrd + DistanceCast + One {
    fn make2(domain: D, metric: M) -> Fallible<Transformation<D, D, M, M>> {
        Ok(Transformation::new(
            domain.clone(),
            domain,
            Function::new(|arg: &T| arg.clone()),
            metric.clone(),
            metric,
            StabilityRelation::new_from_constant(Q::one())))
    }
}

pub struct Clamp<M, T> {
    metric: PhantomData<M>,
    data: PhantomData<T>,
}

impl<M, T> MakeTransformation2<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>, M, M, T, T> for Clamp<M, T>
    where M: DatasetMetric<Distance=u32>,
          T: 'static + Copy + PartialOrd {
    fn make2(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>, M, M>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))),
            Function::new(move |arg: &Vec<T>| clamp(lower, upper, arg)),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}

fn clamp<T: Copy + PartialOrd>(lower: T, upper: T, x: &Vec<T>) -> Vec<T> {
    fn clamp1<T: Copy + PartialOrd>(lower: T, upper: T, x: T) -> T {
        if x < lower { lower } else if x > upper { upper } else { x }
    }
    x.into_iter().map(|e| clamp1(lower, upper, *e)).collect()
}

pub struct BoundedSum<MI, MO, T> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    data: PhantomData<T>,
}

// TODO: this is kind of ugly and should bubble results
fn max<T: PartialOrd>(a: T, b: T) -> T {
    match a.partial_cmp(&b) {
        Some(Ordering::Less) => b,
        _ => a
    }
}

impl<MO, T> MakeTransformation2<VectorDomain<IntervalDomain<T>>, AllDomain<T>, HammingDistance, MO, T, T> for BoundedSum<HammingDistance, MO, T>
    where T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast,
          HammingDistance: Metric<Distance=u32>,
          MO: SensitivityMetric<Distance=T> {
    fn make2(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<IntervalDomain<T>>, AllDomain<T>, HammingDistance, MO>> {
        if lower > upper { return Err(Error::MakeTransformation("lower bound may not be greater than upper bound".to_string()).into()) }

        Ok(Transformation::new(
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
            AllDomain::new(),
            Function::new(|arg: &Vec<T>| arg.iter().cloned().sum()),
            HammingDistance::new(),
            MO::new(),
            StabilityRelation::new_from_constant(upper - lower)))
    }
}

impl<MO, T> MakeTransformation2<VectorDomain<IntervalDomain<T>>, AllDomain<T>, SymmetricDistance, MO, T, T> for BoundedSum<SymmetricDistance, MO, T>
    where T: 'static + Copy + PartialOrd + Sub<Output=T> + Mul<Output=T> + Sum<T> + Signed + DistanceCast,
          MO: SensitivityMetric<Distance=T>,
          MO::Distance: Clone + Mul<Output=MO::Distance> + Div<Output=MO::Distance> + PartialOrd, {
    // Question- how to set the associated type for a trait that a concrete type is using
    fn make2(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<IntervalDomain<T>>, AllDomain<T>, SymmetricDistance, MO>> {
        if lower > upper { return Err(Error::MakeTransformation("lower bound may not be greater than upper bound".to_string()).into()) }

        Ok(Transformation::new(
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
            AllDomain::new(),
            Function::new(|arg: &Vec<T>| arg.iter().cloned().sum()),
            SymmetricDistance::new(),
            MO::new(),
            // d_out >= d_in * max(|m|, |M|)
            StabilityRelation::new_from_constant(max(num::abs(lower), num::abs(upper)))))
    }
}

impl<MO, T> MakeTransformation3<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, MO, usize, T, T> for BoundedSum<SymmetricDistance, MO, T>
    where T: 'static + Copy + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast,
          MO: SensitivityMetric<Distance=T>,
          SymmetricDistance: Metric<Distance=u32> {
    fn make3(length: usize, lower: T, upper: T) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, MO>> {
        if lower > upper { return Err(Error::MakeTransformation("lower bound may not be greater than upper bound".to_string()).into()) }

        Ok(Transformation::new(
            SizedDomain::new(VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))), length),
            AllDomain::new(),
            Function::new(|arg: &Vec<T>| arg.iter().cloned().sum()),
            SymmetricDistance::new(),
            MO::new(),
            // d_out >= d_in * (M - m) / 2
            StabilityRelation::new_from_constant((upper - lower) / T::cast(2)?)))
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
            Function::new(move |arg: &Vec<T>| arg.len() as u32),
            MI::new(),
            MO::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}


#[cfg(test)]
mod tests {
    use crate::dist::{L1Sensitivity, L2Sensitivity};

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
        let transformation = Clamp::<HammingDistance, i32>::make(0, 10).unwrap();
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
