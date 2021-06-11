use std::collections::Bound;

use num::One;

use crate::core::{DatasetMetric, Function, Metric, StabilityRelation, Transformation, SensitivityMetric};
use crate::dom::{AllDomain, IntervalDomain, VectorDomain};
use crate::error::*;
use crate::traits::{DistanceConstant, DistanceCast};
use std::ops::Sub;

pub fn make_clamp_vec<M, T>(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<AllDomain<T>>, VectorDomain<IntervalDomain<T>>, M, M>>
    where M: DatasetMetric,
          T: 'static + Clone + PartialOrd,
          M::Distance: DistanceConstant + One {
    if lower > upper { return fallible!(MakeTransformation, "lower may not be greater than upper") }
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))),
        Function::new(move |arg: &Vec<T>| arg.iter().map(|e| clamp(&lower, &upper, e)).collect()),
        M::default(),
        M::default(),
        // clamping has a c-stability of one, as well as a lipschitz constant of one
        StabilityRelation::new_from_constant(M::Distance::one())))
}

fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b {a} else {b} }

pub fn make_clamp_sensitivity<M, T>(lower: T, upper: T) -> Fallible<Transformation<AllDomain<T>, IntervalDomain<T>, M, M>>
    where M: SensitivityMetric,
          T: 'static + Clone + PartialOrd + DistanceCast + Sub<Output=T>,
          M::Distance: DistanceConstant + One {
    if lower > upper { return fallible!(MakeTransformation, "lower may not be greater than upper") }
    Ok(Transformation::new(
        AllDomain::new(),
        IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone())),
        Function::new(enclose!((lower, upper), move |arg: &T| clamp(&lower, &upper, arg))),
        M::default(),
        M::default(),
        // the sensitivity is at most upper - lower
        StabilityRelation::new_all(
            // relation
            enclose!((lower, upper), move |d_in: &M::Distance, d_out: &M::Distance|
                Ok(d_out.clone() >= min(d_in.clone(), M::Distance::distance_cast(upper.clone() - lower.clone())?))),
            // forward map
            Some(enclose!((lower, upper), move |d_in: &M::Distance|
                Ok(Box::new(min(d_in.clone(), M::Distance::distance_cast(upper.clone() - lower.clone())?))))),
            // backward map
            None::<fn(&_)->_>
        )))
}

fn clamp<T: Clone + PartialOrd>(lower: &T, upper: &T, x: &T) -> T {
    (if x < lower { lower } else if x > upper { upper } else { x }).clone()
}


pub fn make_unclamp_vec<M, T>(lower: T, upper: T) -> Fallible<Transformation<VectorDomain<IntervalDomain<T>>, VectorDomain<AllDomain<T>>, M, M>>
    where M: Metric,
          T: 'static + Clone + PartialOrd,
          M::Distance: DistanceConstant + One {
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
          M::Distance: DistanceConstant + One {
    Ok(Transformation::new(
        IntervalDomain::new(lower, upper),
        AllDomain::new(),
        Function::new(move |arg: &T| arg.clone()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(M::Distance::one())
    ))
}



#[cfg(test)]
mod test_manipulations {

    use super::*;
    use crate::dist::{SymmetricDistance, HammingDistance};
    use crate::trans::{make_clamp_vec, make_unclamp_vec};

    #[test]
    fn test_unclamp() -> Fallible<()> {
        let clamp = make_clamp_vec::<SymmetricDistance, u8>(2, 3)?;
        let unclamp = make_unclamp_vec(2, 3)?;

        (clamp >> unclamp).map(|_| ())
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
