use num::One;

use crate::core::{Domain, Function, Metric, StabilityRelation, Transformation, DatasetMetric};
use crate::error::*;
use crate::traits::{DistanceConstant};
use crate::dom::VectorDomain;


/// Constructs a [`Transformation`] representing an arbitrary row-by-row transformation.
pub(crate) fn make_row_by_row<'a, DIA, DOA, M, F: 'static + Fn(&DIA::Carrier) -> DOA::Carrier>(
    atom_input_domain: DIA,
    atom_output_domain: DOA,
    atom_function: F
) -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<DOA>, M, M>>
    where DIA: Domain, DOA: Domain,
          DIA::Carrier: 'static, DOA::Carrier: 'static,
          M: DatasetMetric {
    Ok(Transformation::new(
        VectorDomain::new(atom_input_domain),
        VectorDomain::new(atom_output_domain),
        Function::new(move |arg: &Vec<DIA::Carrier>|
            arg.iter().map(|v| atom_function(v)).collect()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(1_u32)))
}

/// Constructs a [`Transformation`] representing an arbitrary row-by-row transformation.
pub(crate) fn make_row_by_row_fallible<DIA, DOA, M, F: 'static + Fn(&DIA::Carrier) -> Fallible<DOA::Carrier>>(
    atom_input_domain: DIA,
    atom_output_domain: DOA,
    atom_function: F
) -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<DOA>, M, M>>
    where DIA: Domain, DOA: Domain,
          DIA::Carrier: 'static, DOA::Carrier: 'static,
          M: DatasetMetric {
    Ok(Transformation::new(
        VectorDomain::new(atom_input_domain),
        VectorDomain::new(atom_output_domain),
        Function::new_fallible(move |arg: &Vec<DIA::Carrier>|
            arg.iter().map(|v| atom_function(v)).collect()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(1_u32)))
}

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
