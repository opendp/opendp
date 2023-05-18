#[cfg(feature = "ffi")]
mod ffi;

use num::One;
use opendp_derive::bootstrap;

use crate::core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::*;
use crate::metrics::{
    ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
};
use crate::traits::{CheckAtom, CheckNull, DistanceConstant};

/// A [`Domain`] representing a dataset.
///
/// This is distinguished from other domains
/// because each element in the dataset corresponds to an individual.
pub trait DatasetDomain: Domain {
    /// The domain of each element in the dataset.
    ///
    /// For vectors, this is the domain of the vector elements,
    /// for dataframes, this is the domain of the dataframe rows,
    /// and so on.
    type ElementDomain: Domain;
}

impl<D: Domain> DatasetDomain for VectorDomain<D> {
    type ElementDomain = D;
}

pub trait DatasetMetric: Metric<Distance = IntDistance> {}

impl DatasetMetric for SymmetricDistance {}
impl DatasetMetric for InsertDeleteDistance {}
impl DatasetMetric for ChangeOneDistance {}
impl DatasetMetric for HammingDistance {}

pub trait RowByRowDomain<DO: DatasetDomain>: DatasetDomain {
    fn translate(&self, output_row_domain: DO::ElementDomain) -> DO;
    fn apply_rows(
        value: &Self::Carrier,
        row_function: &impl Fn(
            &<Self::ElementDomain as Domain>::Carrier,
        ) -> Fallible<<DO::ElementDomain as Domain>::Carrier>,
    ) -> Fallible<DO::Carrier>;
}

impl<DIA: Domain, DOA: Domain> RowByRowDomain<VectorDomain<DOA>> for VectorDomain<DIA> {
    fn translate(
        &self,
        output_row_domain: <VectorDomain<DOA> as DatasetDomain>::ElementDomain,
    ) -> VectorDomain<DOA> {
        VectorDomain {
            element_domain: output_row_domain,
            size: self.size,
        }
    }

    fn apply_rows(
        value: &Self::Carrier,
        row_function: &impl Fn(&DIA::Carrier) -> Fallible<DOA::Carrier>,
    ) -> Fallible<Vec<DOA::Carrier>> {
        value.iter().map(row_function).collect()
    }
}

/// Constructs a [`Transformation`] representing an arbitrary row-by-row transformation.
pub(crate) fn make_row_by_row<DI, DO, M>(
    input_domain: DI,
    input_metric: M,
    output_row_domain: DO::ElementDomain,
    row_function: impl 'static
        + Fn(&<DI::ElementDomain as Domain>::Carrier) -> <DO::ElementDomain as Domain>::Carrier,
) -> Fallible<Transformation<DI, DO, M, M>>
where
    DI: RowByRowDomain<DO>,
    DO: DatasetDomain,
    M: DatasetMetric<Distance = IntDistance>,
    (DI, M): MetricSpace,
    (DO, M): MetricSpace,
{
    let row_function = move |arg: &<DI::ElementDomain as Domain>::Carrier| Ok(row_function(arg));
    make_row_by_row_fallible(input_domain, input_metric, output_row_domain, row_function)
}

/// Constructs a [`Transformation`] representing an arbitrary row-by-row transformation.
pub(crate) fn make_row_by_row_fallible<DI, DO, M>(
    input_domain: DI,
    input_metric: M,
    output_row_domain: DO::ElementDomain,
    row_function: impl 'static
        + Fn(
            &<DI::ElementDomain as Domain>::Carrier,
        ) -> Fallible<<DO::ElementDomain as Domain>::Carrier>,
) -> Fallible<Transformation<DI, DO, M, M>>
where
    DI: RowByRowDomain<DO>,
    DO: DatasetDomain,
    M: DatasetMetric<Distance = IntDistance>,
    (DI, M): MetricSpace,
    (DO, M): MetricSpace,
{
    let output_domain = input_domain.translate(output_row_domain);
    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |arg: &DI::Carrier| DI::apply_rows(arg, &row_function)),
        input_metric.clone(),
        input_metric,
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
    make_row_by_row(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
        AtomDomain::default(),
        move |v| v == &value,
    )
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
    make_row_by_row(
        VectorDomain::new(input_atom_domain),
        SymmetricDistance::default(),
        AtomDomain::default(),
        |v| v.is_null(),
    )
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
