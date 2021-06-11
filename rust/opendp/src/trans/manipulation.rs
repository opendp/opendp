use num::One;

use crate::core::{Domain, Function, Metric, StabilityRelation, Transformation};
use crate::error::*;
use crate::traits::{DistanceConstant};


/// Constructs a [`Transformation`] representing the identity function.
pub fn make_identity<D, M>(domain: D, metric: M) -> Fallible<Transformation<D, D, M, M>>
    where D: Domain, D::Carrier: Clone,
          M: Metric, M::Distance: DistanceConstant + One {
    Ok(Transformation::new(
        domain.clone(),
        domain,
        Function::new(|arg: &D::Carrier| arg.clone()),
        metric.clone(),
        metric,
        StabilityRelation::new_from_constant(M::Distance::one())))
}



#[cfg(test)]
mod test_manipulations {

    use super::*;
    use crate::dist::{HammingDistance};
    use crate::dom::AllDomain;

    #[test]
    fn test_identity() {
        let identity = make_identity(AllDomain::new(), HammingDistance).unwrap_test();
        let arg = 99;
        let ret = identity.function.eval(&arg).unwrap_test();
        assert_eq!(ret, 99);
    }
}
