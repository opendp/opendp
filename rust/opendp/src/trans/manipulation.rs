use std::collections::Bound;
use std::marker::PhantomData;
use std::ops::{Div, Mul};

use num::One;

use crate::core::{DatasetMetric, Domain, Function, Metric, StabilityRelation, Transformation};
use crate::dom::{AllDomain, IntervalDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{CastFrom, DistanceCast};
use crate::trans::{MakeTransformation0, MakeTransformation2};


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
    data: PhantomData<T>
}

impl<M, T, Q> MakeTransformation2<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>, M, M, T, T> for Clamp<M, Vec<T>>
    where M: Metric<Distance=Q>,
          T: 'static + Clone + PartialOrd,
          Q: 'static + One + Mul<Output=Q> + Div<Output=Q> + PartialOrd + DistanceCast {
    fn make2(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>, M, M>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
            Function::new(move |arg: &Vec<T>| arg.into_iter().map(|e| clamp(&lower, &upper, e)).collect()),
            M::new(),
            M::new(),
            // clamping has a c-stability of one, as well as a lipschitz constant of one
            StabilityRelation::new_from_constant(Q::one())))
    }
}

impl<M, T, Q> MakeTransformation2<AllDomain<T>, IntervalDomain<T>, M, M, T, T> for Clamp<M, T>
    where M: Metric<Distance=Q>,
          T: 'static + Clone + PartialOrd,
          Q: 'static + One + Mul<Output=Q> + Div<Output=Q> + PartialOrd + DistanceCast {
    fn make2(lower: T, upper: T) -> Fallible<Transformation<AllDomain<T>, IntervalDomain<T>, M, M>> {
        Ok(Transformation::new(
            AllDomain::new(),
            IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone())),
            Function::new(move |arg: &T| clamp(&lower, &upper, arg)),
            M::new(),
            M::new(),
            // clamping has a c-stability of one, as well as a lipschitz constant of one
            StabilityRelation::new_from_constant(Q::one())))
    }
}

fn clamp<T: Clone + PartialOrd>(lower: &T, upper: &T, x: &T) -> T {
    (if x < &lower { lower } else if x > &upper { upper } else { x }).clone()
}

pub struct Unclamp<M, T> {
    metric: PhantomData<M>,
    data: PhantomData<T>
}

impl<M, T, Q> MakeTransformation2<VectorDomain<IntervalDomain<T>>, VectorDomain<AllDomain<T>>, M, M, T, T> for Unclamp<M, Vec<T>>
    where M: Metric<Distance=Q>,
          T: 'static + Clone + PartialOrd,
          Q: 'static + Default + DistanceCast + One + Div<Output=Q> + Mul<Output=Q> + PartialOrd {
    fn make2(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<IntervalDomain<T>>, VectorDomain<AllDomain<T>>, M, M>> {
        Ok(Transformation::new(
            VectorDomain::new(IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))),
            VectorDomain::new_all(),
            Function::new(move |arg: &Vec<T>| arg.clone()),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(Q::one())
        ))
    }
}

impl<M, T, Q> MakeTransformation2<IntervalDomain<T>, AllDomain<T>, M, M, Bound<T>, Bound<T>> for Unclamp<M, T>
    where M: Metric<Distance=Q>,
          T: 'static + Clone + PartialOrd,
          Q: 'static + Default + DistanceCast + One + Div<Output=Q> + Mul<Output=Q> + PartialOrd {
    fn make2(lower: Bound<T>, upper: Bound<T>) -> Fallible<Transformation<IntervalDomain<T>, AllDomain<T>, M, M>> {
        Ok(Transformation::new(
            IntervalDomain::new(lower, upper),
            AllDomain::new(),
            Function::new(move |arg: &T| arg.clone()),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(Q::one())
        ))
    }
}


pub struct Cast<M, TI, TO> {
    metric: PhantomData<M>,
    data_input: PhantomData<TI>,
    data_output: PhantomData<TO>,
}

impl<M, TI, TO> MakeTransformation0<VectorDomain<AllDomain<TI>>, VectorDomain<AllDomain<TO>>, M, M> for Cast<M, Vec<TI>, Vec<TO>>
    where M: DatasetMetric<Distance=u32>,
          TI: Clone, TO: CastFrom<TI> + Default {
    fn make0() -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, VectorDomain<AllDomain<TO>>, M, M>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            VectorDomain::new_all(),
            Function::new(move |arg: &Vec<TI>| arg.into_iter()
                .map(|v| TO::cast(v.clone()).unwrap_or_else(|_| TO::default()))
                .collect()),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}

// casting primitive types is not exposed over ffi.
// Need a way to also cast M::Distance that doesn't allow changing M
impl<M, TI, TO> MakeTransformation0<AllDomain<TI>, AllDomain<TO>, M, M> for Cast<M, TI, TO>
    where M: Metric,
          M::Distance: 'static + One + DistanceCast + Div<Output=M::Distance> + Mul<Output=M::Distance> + PartialOrd,
          TI: Clone,
          TO: 'static + CastFrom<TI> + Default {
    fn make0() -> Fallible<Transformation<AllDomain<TI>, AllDomain<TO>, M, M>> {
        Ok(Transformation::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new(move |v: &TI| TO::cast(v.clone()).unwrap_or_else(|_| TO::default())),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(M::Distance::one())))
    }
}

#[cfg(test)]
mod test_manipulations {

    use super::*;
    use crate::dist::{SymmetricDistance, HammingDistance};
    use crate::core::ChainTT;

    #[test]
    fn test_unclamp() {
        let clamp = Clamp::<SymmetricDistance, Vec<u8>>::make2(2, 3).unwrap();
        let unclamp = Unclamp::<SymmetricDistance, Vec<u8>>::make2(2, 3).unwrap();
        ChainTT::make(&clamp, &unclamp).unwrap();
    }

    #[test]
    fn test_cast() {
        macro_rules! test_pair {
            ($from:ty, $to:ty) => {
                let caster = Cast::<SymmetricDistance, Vec<$from>, Vec<$to>>::make().unwrap();
                caster.function.eval(&vec!(<$from>::default())).unwrap();
                let caster = Cast::<HammingDistance, Vec<$from>, Vec<$to>>::make().unwrap();
                caster.function.eval(&vec!(<$from>::default())).unwrap();
            }
        }
        macro_rules! test_cartesian {
            ([];[$first:ty, $($end:ty),*]) => {
                test_pair!($first, $first);
                $(test_pair!($first, $end);)*

                test_cartesian!{[$first];[$($end),*]}
            };
            ([$($start:ty),*];[$mid:ty, $($end:ty),*]) => {
                $(test_pair!($mid, $start);)*
                test_pair!($mid, $mid);
                $(test_pair!($mid, $end);)*

                test_cartesian!{[$($start),*, $mid];[$($end),*]}
            };
            ([$($start:ty),*];[$last:ty]) => {
                test_pair!($last, $last);
                $(test_pair!($last, $start);)*
            };
        }
        test_cartesian!{[];[u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, bool]}
    }

    #[test]
    fn test_cast_unsigned() {
        let caster = Cast::<SymmetricDistance, Vec<f64>, Vec<u8>>::make().unwrap();
        assert_eq!(caster.function.eval(&vec![-1.]).unwrap(), vec![u8::default()]);
    }
    #[test]
    fn test_cast_parse() {
        let data = vec!["2".to_string(), "3".to_string(), "a".to_string(), "".to_string()];

        let caster = Cast::<SymmetricDistance, Vec<String>, Vec<u8>>::make().unwrap();
        assert_eq!(caster.function.eval(&data).unwrap(), vec![2, 3, u8::default(), u8::default()]);

        let caster = Cast::<SymmetricDistance, Vec<String>, Vec<f64>>::make().unwrap();
        assert_eq!(caster.function.eval(&data).unwrap(), vec![2., 3., f64::default(), f64::default()]);
    }

    #[test]
    fn test_cast_floats() {
        let data = vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY];
        let caster = Cast::<SymmetricDistance, Vec<f64>, Vec<String>>::make().unwrap();
        assert_eq!(
            caster.function.eval(&data).unwrap(),
            vec!["NaN".to_string(), "-inf".to_string(), "inf".to_string()]);

        let caster = Cast::<SymmetricDistance, Vec<f64>, Vec<u8>>::make().unwrap();
        assert_eq!(
            caster.function.eval(&vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY]).unwrap(),
            vec![u8::default(), u8::default(), u8::default()]);

        let data = vec!["1e+2", "1e2", "1e+02", "1.e+02", "1.0E+02", "1.0E+00002", "01.E+02", "1.0E2"]
            .into_iter().map(|v| v.to_string()).collect();
        let caster = Cast::<SymmetricDistance, Vec<String>, Vec<f64>>::make().unwrap();
        assert!(caster.function.eval(&data).unwrap().into_iter().all(|v| v == 100.));
    }

    #[test]
    fn test_identity() {
        let identity = Identity::make(AllDomain::new(), HammingDistance).unwrap();
        let arg = 99;
        let ret = identity.function.eval(&arg).unwrap();
        assert_eq!(ret, 99);
    }


    #[test]
    fn test_make_clamp() {
        let transformation = Clamp::<HammingDistance, Vec<i32>>::make(0, 10).unwrap();
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }

}
