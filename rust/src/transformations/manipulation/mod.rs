#[cfg(feature = "ffi")]
mod ffi;

use num::One;
use opendp_derive::bootstrap;

use crate::core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::*;
use crate::metrics::{IntDistance, SymmetricDistance};
use crate::traits::{CheckAtom, CheckNull, DistanceConstant};

/// Constructs a [`Transformation`] representing an arbitrary row-by-row transformation.
pub(crate) fn make_row_by_row<DIA, DOA, M>(
    atom_input_domain: DIA,
    atom_output_domain: DOA,
    atom_function: impl 'static + Fn(&DIA::Carrier) -> DOA::Carrier,
) -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<DOA>, M, M>>
where
    DIA: Domain,
    DOA: Domain,
    DIA::Carrier: 'static,
    M: Metric<Distance = IntDistance>,
    (VectorDomain<DIA>, M): MetricSpace,
    (VectorDomain<DOA>, M): MetricSpace,
{
    Transformation::new(
        VectorDomain::new(atom_input_domain),
        VectorDomain::new(atom_output_domain),
        Function::new(move |arg: &Vec<DIA::Carrier>| arg.iter().map(&atom_function).collect()),
        M::default(),
        M::default(),
        StabilityMap::new_from_constant(1),
    )
}

/// Constructs a [`Transformation`] representing an arbitrary row-by-row transformation.
pub(crate) fn make_row_by_row_fallible<DIA, DOA, M>(
    atom_input_domain: DIA,
    atom_output_domain: DOA,
    atom_function: impl 'static + Fn(&DIA::Carrier) -> Fallible<DOA::Carrier>,
) -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<DOA>, M, M>>
where
    DIA: Domain,
    DOA: Domain,
    DIA::Carrier: 'static,
    M: Metric<Distance = IntDistance>,
    (VectorDomain<DIA>, M): MetricSpace,
    (VectorDomain<DOA>, M): MetricSpace,
{
    Transformation::new(
        VectorDomain::new(atom_input_domain),
        VectorDomain::new(atom_output_domain),
        Function::new_fallible(move |arg: &Vec<DIA::Carrier>| {
            arg.iter().map(&atom_function).collect()
        }),
        M::default(),
        M::default(),
        StabilityMap::new_from_constant(1),
    )
}

/// Constructs a [`Transformation`] representing the identity function.
pub fn make_identity<D, M>(domain: D, metric: M) -> Fallible<Transformation<D, D, M, M>>
where
    D: Domain,
    D::Carrier: Clone,
    M: Metric,
    M::Distance: DistanceConstant<M::Distance> + One + Clone,
    (D, M): MetricSpace,
{
    Transformation::new(
        domain.clone(),
        domain,
        Function::new(|arg: &D::Carrier| arg.clone()),
        metric.clone(),
        metric,
        StabilityMap::new_from_constant(M::Distance::one()),
    )
}

#[bootstrap(features("contrib"))]
/// Make a Transformation that checks if each element is equal to `value`.
///
/// # Arguments
/// * `value` - value to check against
///
/// # Generics
/// * `TIA` - Atomic Input Type. Type of elements in the input vector
pub fn make_is_equal<TIA>(
    value: TIA,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<AtomDomain<bool>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    TIA: 'static + PartialEq + CheckAtom,
{
    make_row_by_row(AtomDomain::default(), AtomDomain::default(), move |v| {
        v == &value
    })
}

#[bootstrap(
    features("contrib"),
    arguments(input_atom_domain(c_type = "AnyDomain *"))
)]
/// Make a Transformation that checks if each element in a vector is null.
///
/// # Generics
/// * `DIA` - Atomic Input Domain. Can be any domain for which the carrier type has a notion of nullity.
pub fn make_is_null<DIA>(
    input_atom_domain: DIA,
) -> Fallible<
    Transformation<
        VectorDomain<DIA>,
        VectorDomain<AtomDomain<bool>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    DIA: Domain + Default,
    DIA::Carrier: 'static + CheckNull,
{
    make_row_by_row(input_atom_domain, AtomDomain::default(), |v| v.is_null())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domains::AtomDomain;

    #[test]
    fn test_identity() {
        let identity = make_identity(VectorDomain::new(AtomDomain::default()), SymmetricDistance)
            .unwrap_test();
        let arg = vec![99];
        let ret = identity.invoke(&arg).unwrap_test();
        assert_eq!(ret, arg);
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
}
