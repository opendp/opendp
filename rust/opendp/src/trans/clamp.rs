use std::collections::Bound;

use num::One;

use crate::core::{DatasetMetric, Function, Metric, StabilityRelation, Transformation, SensitivityMetric, Domain};
use crate::dom::{AllDomain, IntervalDomain, VectorDomain};
use crate::error::*;
use crate::traits::{DistanceConstant, DistanceCast};
use std::ops::Sub;

pub trait ClampableDomain<M>: Domain
    where M: Metric {
    type Atom;
    type OutputDomain: Domain;
    fn new_input_domain() -> Self;
    fn new_output_domain(lower: Self::Atom, upper: Self::Atom) -> Self::OutputDomain;
    fn clamp_function(lower: Self::Atom, upper: Self::Atom) -> Function<Self, Self::OutputDomain>;
    fn stability_relation(lower: Self::Atom, upper: Self::Atom) -> StabilityRelation<M, M>;
}

impl<M, T> ClampableDomain<M> for VectorDomain<AllDomain<T>>
    where M: DatasetMetric,
          T: 'static + PartialOrd + Clone, {
    type Atom = T;
    type OutputDomain = VectorDomain<IntervalDomain<T>>;

    fn new_input_domain() -> Self { VectorDomain::new_all() }
    fn new_output_domain(lower: Self::Atom, upper: Self::Atom) -> Self::OutputDomain {
        VectorDomain::new(IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone())))
    }
    fn clamp_function(lower: Self::Atom, upper: Self::Atom) -> Function<Self, Self::OutputDomain> {
        Function::new(move |arg: &Vec<T>| arg.iter().map(|v| clamp(&lower, &upper, v)).cloned().collect())
    }
    fn stability_relation(_lower: Self::Atom, _upper: Self::Atom) -> StabilityRelation<M, M> {
        StabilityRelation::new_from_constant(M::Distance::one())
    }
}

impl<M, T> ClampableDomain<M> for AllDomain<T>
    where M: SensitivityMetric,
          M::Distance: DistanceConstant + One,
          T: 'static + Clone + PartialOrd + DistanceCast + Sub<Output=T> {
    type Atom = Self::Carrier;
    type OutputDomain = IntervalDomain<T>;

    fn new_input_domain() -> Self { AllDomain::new() }
    fn new_output_domain(lower: Self::Atom, upper: Self::Atom) -> Self::OutputDomain {
        IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))
    }
    fn clamp_function(lower: Self::Atom, upper: Self::Atom) -> Function<Self, Self::OutputDomain> {
        Function::new(move |arg: &T| clamp(&lower, &upper, arg).clone())
    }
    fn stability_relation(lower: Self::Atom, upper: Self::Atom) -> StabilityRelation<M, M> {
        // the sensitivity is at most upper - lower
        StabilityRelation::new_all(
            // relation
            enclose!((lower, upper), move |d_in: &M::Distance, d_out: &M::Distance|
                Ok(d_out.clone() >= min(d_in.clone(), M::Distance::distance_cast(upper.clone() - lower.clone())?))),
            // forward map
            Some(move |d_in: &M::Distance|
                Ok(Box::new(min(d_in.clone(), M::Distance::distance_cast(upper.clone() - lower.clone())?)))),
            // backward map
            None::<fn(&_)->_>
        )
    }
}

pub fn make_clamp<DI, M>(lower: DI::Atom, upper: DI::Atom) -> Fallible<Transformation<DI, DI::OutputDomain, M, M>>
    where DI: ClampableDomain<M>,
          DI::Atom: Clone + PartialOrd,
          M: Metric {
    if lower > upper { return fallible!(MakeTransformation, "lower may not be greater than upper") }
    Ok(Transformation::new(
        DI::new_input_domain(),
        DI::new_output_domain(lower.clone(), upper.clone()),
        DI::clamp_function(lower.clone(), upper.clone()),
        M::default(),
        M::default(),
        DI::stability_relation(lower, upper)))
}

fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b {a} else {b} }
fn clamp<'a, T: PartialOrd>(lower: &'a T, upper: &'a T, x: &'a T) -> &'a T {
    if x < lower { lower } else if x > upper { upper } else { x }
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
    use crate::trans::{make_clamp, make_unclamp_vec};

    #[test]
    fn test_make_unclamp() -> Fallible<()> {
        let clamp = make_clamp::<VectorDomain<_>, SymmetricDistance>(2, 3)?;
        let unclamp = make_unclamp_vec(2, 3)?;

        (clamp >> unclamp).map(|_| ())
    }

    #[test]
    fn test_make_clamp() {
        let transformation = make_clamp::<VectorDomain<_>, HammingDistance>(0, 10).unwrap_test();
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }
}
