//! Various implementations of Transformations.
//!
//! The different [`Transformation`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Transformation` does.

use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{Bound, Mul, Sub, Div};

use crate::core::{DatasetMetric, Domain, Metric, SensitivityMetric, Transformation};
use crate::dist::{HammingDistance, L1Sensitivity, SymmetricDistance};
use crate::dom::{AllDomain, IntervalDomain, VectorDomain, SizedDomain};
pub use crate::trans::dataframe::*;
use num::{Signed, NumCast};
use std::cmp::Ordering;

pub mod dataframe;

// Trait for all constructors, can have different implementations depending on concrete types of Domains and/or Metrics
pub trait MakeTransformation0<DI: Domain, DO: Domain, MI: Metric, MO: Metric> {
    fn construct() -> crate::core::Transformation<DI, DO, MI, MO>;
}

pub trait MakeTransformation1<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1> {
    fn construct(param1: P1) -> crate::core::Transformation<DI, DO, MI, MO>;
}

pub trait MakeTransformation2<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1, P2> {
    fn construct(param1: P1, param2: P2) -> crate::core::Transformation<DI, DO, MI, MO>;
}

pub trait MakeTransformation3<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1, P2, P3> {
    fn construct(param1: P1, param2: P2, param3: P3) -> crate::core::Transformation<DI, DO, MI, MO>;
}

pub trait MakeTransformation4<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1, P2, P3, P4> {
    fn construct(param1: P1, param2: P2, param3: P3, param4: P4) -> crate::core::Transformation<DI, DO, MI, MO>;
}

/// Constructs a [`Transformation`] representing the identity function.
pub struct Identity;

impl<D, T, M, Q> MakeTransformation2<D, D, M, M, D, M> for Identity
    where D: Domain<Carrier=T>, T: Clone,
          M: Metric<Distance=Q>, Q: Clone {
    fn construct(domain: D, metric: M) -> Transformation<D, D, M, M> {
        let function = |arg: &T| arg.clone();
        let stability_relation = |_d_in: &Q, _d_out: &Q| true;
        Transformation::new(domain.clone(), domain, function, metric.clone(), metric, stability_relation)
    }
}

pub struct Clamp<M, T> {
    metric: PhantomData<M>,
    data: PhantomData<T>,
}

impl<M, T> MakeTransformation2<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>, M, M, T, T> for Clamp<M, T>
    where M: Metric<Distance=u32> + DatasetMetric,
          T: 'static + Copy + PartialOrd {
    fn construct(lower: T, upper: T) -> Transformation<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>, M, M> {
        Transformation::new(
            VectorDomain::new_all(),
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))),
            move |arg: &Vec<T>| -> Vec<T> {
                clamp(lower, upper, arg)
            },
            M::new(),
            M::new(),
            |d_in: &u32, d_out: &u32| *d_out >= *d_in)
    }
}

fn clamp<T: Copy + PartialOrd>(lower: T, upper: T, x: &Vec<T>) -> Vec<T> {
    fn clamp1<T: Copy + PartialOrd>(lower: T, upper: T, x: T) -> T {
        if x < lower { lower } else if x > upper { upper } else { x }
    }
    x.into_iter().map(|e| clamp1(lower, upper, *e)).collect()
}

pub struct BoundedSum<MI, T> {
    input_metric: PhantomData<MI>,
    data: PhantomData<T>,
}

impl<MO, T> MakeTransformation3<VectorDomain<IntervalDomain<T>>, AllDomain<T>, HammingDistance, MO, T, T, MO> for BoundedSum<HammingDistance, T>
    where T: 'static + Copy + PartialOrd + Sub<Output=T> + NumCast + Mul<Output=T> + Sum<T>,
          MO: SensitivityMetric<Distance=T> {
    fn construct(lower: T, upper: T, output_metric: MO) -> Transformation<VectorDomain<IntervalDomain<T>>, AllDomain<T>, HammingDistance, MO> {
        Transformation::new(
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
            AllDomain::new(),
            |arg: &Vec<T>| arg.iter().cloned().sum(),
            HammingDistance::new(),
            output_metric,
            move |d_in: &u32, d_out: &T| *d_out >= T::from(*d_in).unwrap() * (upper - lower))
    }
}

// TODO: this is kind of ugly and should bubble results
fn max<T: PartialOrd>(a: T, b: T) -> T {
    match a.partial_cmp(&b) {
        Some(Ordering::Less) => b,
        _ => a
    }
}

impl<MO, T> MakeTransformation3<VectorDomain<IntervalDomain<T>>, AllDomain<T>, SymmetricDistance, MO, T, T, MO> for BoundedSum<SymmetricDistance, T>
    where T: 'static + Copy + PartialOrd + Sub<Output=T> + NumCast + Mul<Output=T> + Sum<T> + Signed,
          MO: SensitivityMetric<Distance=T> {
    // Question- how to set the associated type for a trait that a concrete type is using
    fn construct(lower: T, upper: T, output_metric: MO) -> Transformation<VectorDomain<IntervalDomain<T>>, AllDomain<T>, SymmetricDistance, MO> {
        Transformation::new(
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
            AllDomain::new(),
            |arg: &Vec<T>| arg.iter().cloned().sum(),
            SymmetricDistance::new(),
            output_metric,
            // d_out >= d_in * max(|m|, |M|)
            move |d_in: &u32, d_out: &T| *d_out >= T::from(*d_in).unwrap() * max(num::abs(lower), num::abs(upper)))
    }
}

impl<MO, T> MakeTransformation4<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, MO, usize, T, T, MO> for BoundedSum<SymmetricDistance, T>
    where T: 'static + Copy + PartialOrd + Sub<Output=T> + NumCast + Mul<Output=T> + Div<Output=T> + Sum<T>,
          MO: SensitivityMetric<Distance=T>,
          SymmetricDistance: Metric<Distance=u32>  {
    fn construct(length: usize, lower: T, upper: T, output_metric: MO) -> Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, MO> {
        Transformation::new(
            SizedDomain::new(VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))), length),
            AllDomain::new(),
            |arg: &Vec<T>| arg.iter().cloned().sum(),
            SymmetricDistance::new(),
            output_metric,
            // d_out >= d_in * (M - m) / 2
            move |d_in: &u32, d_out: &T| *d_out >= T::from(*d_in).unwrap() * (upper - lower) / T::from(2).unwrap())
    }
}

pub struct Count<MI: Metric, T> {
    input_metric: PhantomData<MI>,
    data: PhantomData<T>
}

impl<MI, MO, T> MakeTransformation1<VectorDomain<AllDomain<T>>, AllDomain<u32>, MI, MO, MO> for Count<MI, T>
    where MI: Metric<Distance=u32> + DatasetMetric,
          MO: Metric<Distance=u32> + SensitivityMetric {
    fn construct(output_space: MO) -> Transformation<VectorDomain<AllDomain<T>>, AllDomain<u32>, MI, MO> {
        Transformation::new(
            VectorDomain::new_all(),
            AllDomain::new(),
            move |arg: &Vec<T>| arg.len() as u32,
            MI::new(),
            output_space,
            |d_in: &u32, d_out: &u32| *d_out >= *d_in)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dist::L2Sensitivity;

    #[test]
    fn test_identity() {
        let identity = Identity::construct(AllDomain::new(), HammingDistance);
        let arg = 99;
        let ret = identity.function.eval(&arg);
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_make_split_lines() {
        let transformation = SplitLines::<HammingDistance>::construct();
        let arg = "ant\nbat\ncat\n".to_owned();
        let ret = transformation.function.eval(&arg);
        assert_eq!(ret, vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]);
    }

    #[test]
    fn test_make_parse_series() {
        let transformation = ParseSeries::<i32, HammingDistance>::construct(true);
        let arg = vec!["1".to_owned(), "2".to_owned(), "3".to_owned(), "foo".to_owned()];
        let ret = transformation.function.eval(&arg);
        let expected = vec![1, 2, 3, 0];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_split_records() {
        let transformation = SplitRecords::<HammingDistance>::construct(None);
        let arg = vec!["ant, foo".to_owned(), "bat, bar".to_owned(), "cat, baz".to_owned()];
        let ret = transformation.function.eval(&arg);
        assert_eq!(ret, vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ]);
    }

    #[test]
    fn test_make_clamp() {
        let transformation = Clamp::<HammingDistance, i32>::construct(0, 10);
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.function.eval(&arg);
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_l1() {
        let transformation = BoundedSum::<HammingDistance, i32>::construct(0, 10, L1Sensitivity::new());
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg);
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_l2() {
        let transformation = BoundedSum::<HammingDistance, i32>::construct(0, 10, L2Sensitivity::new());
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg);
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_l1() {
        let transformation = Count::<SymmetricDistance, i32>::construct(L1Sensitivity::new());
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg);
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_l2() {
        let transformation = Count::<SymmetricDistance, i32>::construct(L2Sensitivity::new());
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg);
        let expected = 5;
        assert_eq!(ret, expected);
    }
}
