#[cfg(feature="ffi")]
mod ffi;

use num::One;

use crate::core::{Domain, Function, Metric, StabilityRelation, Transformation, DatasetMetric};
use crate::error::*;
use crate::traits::{DistanceConstant, CheckNull};
use crate::dom::{VectorDomain, AllDomain};
use crate::dist::SymmetricDistance;


/// Constructs a [`Transformation`] representing an arbitrary row-by-row transformation.
pub(crate) fn make_row_by_row<'a, DIA, DOA, M>(
    atom_input_domain: DIA,
    atom_output_domain: DOA,
    atom_function: impl 'static + Fn(&DIA::Carrier) -> DOA::Carrier
) -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<DOA>, M, M>>
    where DIA: Domain, DOA: Domain,
          DIA::Carrier: 'static,
          M: DatasetMetric {
    Ok(Transformation::new(
        VectorDomain::new(atom_input_domain),
        VectorDomain::new(atom_output_domain),
        Function::new(move |arg: &Vec<DIA::Carrier>|
            arg.iter().map(|v| atom_function(v)).collect()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(1)))
}

/// Constructs a [`Transformation`] representing an arbitrary row-by-row transformation.
pub(crate) fn make_row_by_row_fallible<DIA, DOA, M>(
    atom_input_domain: DIA,
    atom_output_domain: DOA,
    atom_function: impl 'static + Fn(&DIA::Carrier) -> Fallible<DOA::Carrier>
) -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<DOA>, M, M>>
    where DIA: Domain, DOA: Domain,
          DIA::Carrier: 'static,
          M: DatasetMetric {
    Ok(Transformation::new(
        VectorDomain::new(atom_input_domain),
        VectorDomain::new(atom_output_domain),
        Function::new_fallible(move |arg: &Vec<DIA::Carrier>|
            arg.iter().map(|v| atom_function(v)).collect()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(1)))
}

/// Constructs a [`Transformation`] representing the identity function.
pub fn make_identity<D, M>(domain: D, metric: M) -> Fallible<Transformation<D, D, M, M>>
    where D: Domain, D::Carrier: Clone,
          M: Metric, M::Distance: DistanceConstant<M::Distance> + One {
    Ok(Transformation::new(
        domain.clone(),
        domain,
        Function::new(|arg: &D::Carrier| arg.clone()),
        metric.clone(),
        metric,
        StabilityRelation::new_from_constant(M::Distance::one())))
}

/// A [`Transformation`] that checks equality elementwise with `value`.
/// Maps a Vec<TIA> -> Vec<bool>
pub fn make_is_equal<TIA>(
    value: TIA
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, VectorDomain<AllDomain<bool>>, SymmetricDistance, SymmetricDistance>>
    where TIA: 'static + PartialEq + CheckNull {
    make_row_by_row(
        AllDomain::new(),
        AllDomain::new(),
        move |v| v == &value)
}

pub fn make_is_null<DIA>() -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<AllDomain<bool>>, SymmetricDistance, SymmetricDistance>>
    where DIA: Domain + Default,
          DIA::Carrier: 'static + CheckNull {
    make_row_by_row(
        DIA::default(),
        AllDomain::default(),
        |v| v.is_null())
}


#[cfg(test)]
mod tests {

    use super::*;
    use crate::dist::{SubstituteDistance};
    use crate::dom::{AllDomain, InherentNullDomain};

    #[test]
    fn test_identity() {
        let identity = make_identity(AllDomain::new(), SubstituteDistance).unwrap_test();
        let arg = 99;
        let ret = identity.invoke(&arg).unwrap_test();
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_is_equal() -> Fallible<()> {
        let is_equal = make_is_equal("alpha".to_string())?;
        let arg = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let ret = is_equal.invoke(&arg)?;
        assert_eq!(ret, vec![true, false, false]);
        assert!(is_equal.check(&1, &1)?);
        Ok(())
    }

    #[test]
    fn test_is_null() -> Fallible<()> {
        let is_equal = make_is_null::<InherentNullDomain<AllDomain<_>>>()?;
        let arg = vec![f64::NAN, 1., 2.];
        let ret = is_equal.invoke(&arg)?;
        assert_eq!(ret, vec![true, false, false]);
        assert!(is_equal.check(&1, &1)?);
        Ok(())
    }
}
