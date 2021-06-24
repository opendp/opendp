use std::collections::Bound;

use num::One;

use crate::core::{DatasetMetric, Function, Metric, StabilityRelation, Transformation, Domain};
use crate::dom::{AllDomain, IntervalDomain, VectorDomain};
use crate::error::*;
use crate::traits::{DistanceConstant, DistanceCast};
use std::ops::Sub;
use crate::dist::AbsoluteDistance;


fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b {a} else {b} }
fn clamp<'a, T: PartialOrd>(lower: &'a T, upper: &'a T, x: &'a T) -> &'a T {
    if x < lower { lower } else if x > upper { upper } else { x }
}

pub trait ClampableDomain<M>: Domain
    where M: Metric {
    type Atom;
    type OutputDomain: Domain;
    fn new_input_domain() -> Self;
    fn new_output_domain(lower: Self::Atom, upper: Self::Atom) -> Fallible<Self::OutputDomain>;
    fn clamp_function(lower: Self::Atom, upper: Self::Atom) -> Function<Self, Self::OutputDomain>;
    fn stability_relation(lower: Self::Atom, upper: Self::Atom) -> StabilityRelation<M, M>;
}

impl<M, T> ClampableDomain<M> for VectorDomain<AllDomain<T>>
    where M: DatasetMetric,
          T: 'static + PartialOrd + Clone, {
    type Atom = T;
    type OutputDomain = VectorDomain<IntervalDomain<T>>;

    fn new_input_domain() -> Self { VectorDomain::new_all() }
    fn new_output_domain(lower: Self::Atom, upper: Self::Atom) -> Fallible<Self::OutputDomain> {
        IntervalDomain::new(Bound::Included(lower.clone()), Bound::Included(upper.clone()))
            .map(VectorDomain::new)
    }
    fn clamp_function(lower: Self::Atom, upper: Self::Atom) -> Function<Self, Self::OutputDomain> {
        Function::new(move |arg: &Vec<T>| arg.iter().map(|v| clamp(&lower, &upper, v)).cloned().collect())
    }
    fn stability_relation(_lower: Self::Atom, _upper: Self::Atom) -> StabilityRelation<M, M> {
        StabilityRelation::new_from_constant(M::Distance::one())
    }
}

impl<T, Q> ClampableDomain<AbsoluteDistance<Q>> for AllDomain<T>
    where Q: DistanceConstant + One,
          T: 'static + Clone + PartialOrd + DistanceCast + Sub<Output=T> {
    type Atom = T;
    type OutputDomain = IntervalDomain<T>;

    fn new_input_domain() -> Self { AllDomain::new() }
    fn new_output_domain(lower: Self::Atom, upper: Self::Atom) -> Fallible<Self::OutputDomain> {
        IntervalDomain::new(Bound::Included(lower), Bound::Included(upper))
    }
    fn clamp_function(lower: Self::Atom, upper: Self::Atom) -> Function<Self, Self::OutputDomain> {
        Function::new(move |arg: &T| clamp(&lower, &upper, arg).clone())
    }
    fn stability_relation(lower: Self::Atom, upper: Self::Atom) -> StabilityRelation<AbsoluteDistance<Q>, AbsoluteDistance<Q>> {
        // the sensitivity is at most upper - lower
        StabilityRelation::new_all(
            // relation
            enclose!((lower, upper), move |d_in: &Q, d_out: &Q|
                Ok(d_out.clone() >= min(d_in.clone(), Q::distance_cast(upper.clone() - lower.clone())?))),
            // forward map
            Some(move |d_in: &Q|
                Ok(Box::new(min(d_in.clone(), Q::distance_cast(upper.clone() - lower.clone())?)))),
            // backward map
            None::<fn(&_)->_>
        )
    }
}

pub fn make_clamp<DI, M>(lower: DI::Atom, upper: DI::Atom) -> Fallible<Transformation<DI, DI::OutputDomain, M, M>>
    where DI: ClampableDomain<M>,
          DI::Atom: Clone + PartialOrd,
          M: Metric {
    Ok(Transformation::new(
        DI::new_input_domain(),
        DI::new_output_domain(lower.clone(), upper.clone())?,
        DI::clamp_function(lower.clone(), upper.clone()),
        M::default(),
        M::default(),
        DI::stability_relation(lower, upper)))
}


pub trait UnclampableDomain: Domain {
    type Atom;
    type OutputDomain: Domain<Carrier=Self::Carrier>;
    fn new_input_domain(lower: Bound<Self::Atom>, upper: Bound<Self::Atom>) -> Fallible<Self>;
    fn new_output_domain() -> Self::OutputDomain;
}

impl<T> UnclampableDomain for VectorDomain<IntervalDomain<T>>
    where T: PartialOrd + Clone {
    type Atom = T;
    type OutputDomain = VectorDomain<AllDomain<T>>;

    fn new_input_domain(lower: Bound<Self::Atom>, upper: Bound<Self::Atom>) -> Fallible<Self> {
        IntervalDomain::new(lower, upper).map(VectorDomain::new)
    }
    fn new_output_domain() -> Self::OutputDomain {
        VectorDomain::new_all()
    }
}

impl<T> UnclampableDomain for IntervalDomain<T>
    where T: PartialOrd + Clone, {
    type Atom = T;
    type OutputDomain = AllDomain<T>;

    fn new_input_domain(lower: Bound<Self::Atom>, upper: Bound<Self::Atom>) -> Fallible<Self> {
        IntervalDomain::new(lower, upper)
    }
    fn new_output_domain() -> Self::OutputDomain {
        AllDomain::new()
    }
}

pub fn make_unclamp<DI, M>(lower: Bound<DI::Atom>, upper: Bound<DI::Atom>) -> Fallible<Transformation<DI, DI::OutputDomain, M, M>>
    where DI: UnclampableDomain,
          DI::Carrier: Clone,
          M: Metric,
          DI::Atom: 'static + Clone + PartialOrd,
          M::Distance: DistanceConstant + One {
    Ok(Transformation::new(
        DI::new_input_domain(lower.clone(), upper.clone())?,
        DI::new_output_domain(),
        Function::new(|arg: &DI::Carrier| arg.clone()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(M::Distance::one())
    ))
}



#[cfg(test)]
mod tests {

    use super::*;
    use crate::dist::{SymmetricDistance, HammingDistance};
    use crate::trans::{make_clamp, make_unclamp};

    #[test]
    fn test_make_clamp() {
        let transformation = make_clamp::<VectorDomain<_>, HammingDistance>(0, 10).unwrap_test();
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_unclamp() -> Fallible<()> {
        let clamp = make_clamp::<VectorDomain<_>, SymmetricDistance>(2, 3)?;
        let unclamp = make_unclamp(Bound::Included(2), Bound::Included(3))?;
        let chained = (clamp >> unclamp)?;
        chained.function.eval(&vec![1, 2, 3])?;
        assert!(chained.stability_relation.eval(&1, &1)?);
        Ok(())
    }

    #[test]
    fn test_make_clamp_scalar() -> Fallible<()> {
        let transformation = make_clamp::<AllDomain<_>, _>(0, 10)?;
        assert_eq!(transformation.function.eval(&15)?, 10);
        assert!(!transformation.stability_relation.eval(&15, &9)?);
        assert!(transformation.stability_relation.eval(&15, &10)?);
        assert!(!transformation.stability_relation.eval(&5, &4)?);
        assert!(transformation.stability_relation.eval(&5, &5)?);
        Ok(())
    }

    #[test]
    fn test_make_unclamp_scalar() -> Fallible<()> {
        let transformation = make_unclamp::<IntervalDomain<_>, AbsoluteDistance<_>>(Bound::Included(0), Bound::Included(10))?;
        assert_eq!(transformation.function.eval(&15)?, 15);
        assert!(transformation.stability_relation.eval(&15, &15)?);
        Ok(())
    }
}
