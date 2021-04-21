use std::collections::Bound;
use std::ops::{Div, Mul};

use num::One;

use crate::core::{DatasetMetric, Domain, Function, Metric, StabilityRelation, Transformation};
use crate::dom::{AllDomain, IntervalDomain, VectorDomain};
use crate::error::*;
use crate::traits::{CastFrom, DistanceCast, Distance};


/// Constructs a [`Transformation`] representing the identity function.
pub fn make_identity<D, M>(domain: D, metric: M) -> Fallible<Transformation<D, D, M, M>>
    where D: Domain, D::Carrier: Clone,
          M: Metric, M::Distance: 'static + Distance + One {
    Ok(Transformation::new(
        domain.clone(),
        domain,
        Function::new(|arg: &D::Carrier| arg.clone()),
        metric.clone(),
        metric,
        StabilityRelation::new_from_constant(M::Distance::one())))
}

pub fn make_clamp_vec<M, T>(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>, M, M>>
    where M: Metric,
          T: 'static + Clone + PartialOrd,
          M::Distance: 'static + One + Mul<Output=M::Distance> + Div<Output=M::Distance> + PartialOrd + DistanceCast {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
        Function::new(move |arg: &Vec<T>| arg.into_iter().map(|e| clamp(&lower, &upper, e)).collect()),
        M::default(),
        M::default(),
        // clamping has a c-stability of one, as well as a lipschitz constant of one
        StabilityRelation::new_from_constant(M::Distance::one())))
}

pub fn make_clamp<M, T>(lower: T, upper: T) -> Fallible<Transformation<AllDomain<T>, IntervalDomain<T>, M, M>>
    where M: Metric,
          T: 'static + Clone + PartialOrd,
          M::Distance: 'static + One + Mul<Output=M::Distance> + Div<Output=M::Distance> + PartialOrd + DistanceCast {
    Ok(Transformation::new(
        AllDomain::new(),
        IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone())),
        Function::new(move |arg: &T| clamp(&lower, &upper, arg)),
        M::default(),
        M::default(),
        // clamping has a c-stability of one, as well as a lipschitz constant of one
        StabilityRelation::new_from_constant(M::Distance::one())))
}

fn clamp<T: Clone + PartialOrd>(lower: &T, upper: &T, x: &T) -> T {
    (if x < &lower { lower } else if x > &upper { upper } else { x }).clone()
}


pub fn make_unclamp_vec<M, T>(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<IntervalDomain<T>>, VectorDomain<AllDomain<T>>, M, M>>
    where M: Metric,
          T: 'static + Clone + PartialOrd,
          M::Distance: 'static + Default + DistanceCast + One + Div<Output=M::Distance> + Mul<Output=M::Distance> + PartialOrd {
    Ok(Transformation::new(
        VectorDomain::new(IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<T>| arg.clone()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(M::Distance::one())
    ))
}

pub fn make_unclamp<M, T>(lower: Bound<T>, upper: Bound<T>) -> Fallible<Transformation<IntervalDomain<T>, AllDomain<T>, M, M>>
    where M: Metric,
          T: 'static + Clone + PartialOrd,
          M::Distance: 'static + Default + DistanceCast + One + Div<Output=M::Distance> + Mul<Output=M::Distance> + PartialOrd{
    Ok(Transformation::new(
        IntervalDomain::new(lower, upper),
        AllDomain::new(),
        Function::new(move |arg: &T| arg.clone()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(M::Distance::one())
    ))
}


pub fn make_cast_vec<M, TI, TO>() -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, VectorDomain<AllDomain<TO>>, M, M>>
    where M: DatasetMetric<Distance=u32>,
          TI: Clone, TO: CastFrom<TI> + Default {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<TI>| arg.into_iter()
            .map(|v| TO::cast(v.clone()).unwrap_or_else(|_| TO::default()))
            .collect()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(1_u32)))
}

// casting primitive types is not exposed over ffi.
// Need a way to also cast M::Distance that doesn't allow changing M
pub fn make_cast<M, TI, TO>() -> Fallible<Transformation<AllDomain<TI>, AllDomain<TO>, M, M>>
    where M: Metric,
          M::Distance: 'static + One + DistanceCast + Div<Output=M::Distance> + Mul<Output=M::Distance> + PartialOrd,
          TI: Clone,
          TO: 'static + CastFrom<TI> + Default {
    Ok(Transformation::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new(move |v: &TI| TO::cast(v.clone()).unwrap_or_else(|_| TO::default())),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(M::Distance::one())))
}

#[cfg(test)]
mod test_manipulations {

    use super::*;
    use crate::dist::{SymmetricDistance, HammingDistance};
    use crate::trans::manipulation::{make_identity};

    #[test]
    fn test_unclamp() -> Fallible<()> {
        let clamp = make_clamp_vec::<SymmetricDistance, u8>(2, 3)?;
        let unclamp = make_unclamp_vec(2, 3)?;

        (clamp >> unclamp).map(|_| ())
    }

    #[test]
    fn test_cast() {
        macro_rules! test_pair {
            ($from:ty, $to:ty) => {
                let caster = make_cast_vec::<SymmetricDistance, $from, $to>().unwrap_test();
                caster.function.eval(&vec!(<$from>::default())).unwrap_test();
                let caster = make_cast_vec::<HammingDistance, $from, $to>().unwrap_test();
                caster.function.eval(&vec!(<$from>::default())).unwrap_test();
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
    fn test_cast_unsigned() -> Fallible<()> {
        let caster = make_cast_vec::<SymmetricDistance, f64, u8>()?;
        assert_eq!(caster.function.eval(&vec![-1.])?, vec![u8::default()]);
        Ok(())
    }
    #[test]
    fn test_cast_parse() -> Fallible<()> {
        let data = vec!["2".to_string(), "3".to_string(), "a".to_string(), "".to_string()];

        let caster = make_cast_vec::<SymmetricDistance, String, u8>()?;
        assert_eq!(caster.function.eval(&data)?, vec![2, 3, u8::default(), u8::default()]);

        let caster = make_cast_vec::<SymmetricDistance, String, f64>()?;
        assert_eq!(caster.function.eval(&data)?, vec![2., 3., f64::default(), f64::default()]);
        Ok(())
    }

    #[test]
    fn test_cast_floats() -> Fallible<()> {
        let data = vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY];
        let caster = make_cast_vec::<SymmetricDistance, f64, String>()?;
        assert_eq!(
            caster.function.eval(&data)?,
            vec!["NaN".to_string(), "-inf".to_string(), "inf".to_string()]);

        let caster = make_cast_vec::<SymmetricDistance, f64, u8>()?;
        assert_eq!(
            caster.function.eval(&vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY])?,
            vec![u8::default(), u8::default(), u8::default()]);

        let data = vec!["1e+2", "1e2", "1e+02", "1.e+02", "1.0E+02", "1.0E+00002", "01.E+02", "1.0E2"]
            .into_iter().map(|v| v.to_string()).collect();
        let caster = make_cast_vec::<SymmetricDistance, String, f64>()?;
        assert!(caster.function.eval(&data)?.into_iter().all(|v| v == 100.));
        Ok(())
    }

    #[test]
    fn test_identity() {
        let identity = make_identity(AllDomain::new(), HammingDistance).unwrap_test();
        let arg = 99;
        let ret = identity.function.eval(&arg).unwrap_test();
        assert_eq!(ret, 99);
    }


    #[test]
    fn test_make_clamp() {
        let transformation = make_clamp_vec::<HammingDistance, i32>(0, 10).unwrap_test();
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }
}
