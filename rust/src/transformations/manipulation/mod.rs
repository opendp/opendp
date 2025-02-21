#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::*;
use crate::metrics::{
    ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
};
use crate::traits::{CheckAtom, CheckNull};

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

pub trait DatasetMetric: Metric<Distance = IntDistance> {
    const ORDERED: bool;
    const SIZED: bool;
}

impl DatasetMetric for SymmetricDistance {
    const ORDERED: bool = false;
    const SIZED: bool = false;
}
impl DatasetMetric for InsertDeleteDistance {
    const ORDERED: bool = true;
    const SIZED: bool = false;
}
impl DatasetMetric for ChangeOneDistance {
    const ORDERED: bool = false;
    const SIZED: bool = true;
}
impl DatasetMetric for HammingDistance {
    const ORDERED: bool = true;
    const SIZED: bool = false;
}

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
        + Fn(&<DI::ElementDomain as Domain>::Carrier) -> <DO::ElementDomain as Domain>::Carrier
        + Send
        + Sync,
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
        ) -> Fallible<<DO::ElementDomain as Domain>::Carrier>
        + Send
        + Sync,
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

#[bootstrap(
    features("contrib", "honest-but-curious"),
    generics(D(suppress), M(suppress))
)]
/// Make a Transformation representing the identity function.
///
/// WARNING: In Python, this function does not ensure that the domain and metric form a valid metric space.
/// However, if the domain and metric do not form a valid metric space,
/// then the resulting Transformation won't be chainable with any valid Transformation,
/// so it cannot be used to introduce an invalid metric space into a chain of valid Transformations.
///
/// # Arguments
/// * `domain` - Domain of input data
/// * `metric` - Metric on input domain
///
/// # Generics
/// * `D` - Domain of the identity function. Must be `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `M` - Metric. Must be a dataset metric if D is a VectorDomain or a sensitivity metric if D is an AtomDomain
///
/// # Why honest-but-curious?
/// For the result to be a valid transformation, the `input_domain` and `input_metric` pairing must form a valid metric space.
/// For instance, the symmetric distance metric and atom domain do not form a valid metric space,
/// because the metric cannot be used to measure distances between any two elements of an atom domain.
/// Whereas, the symmetric distance metric and vector domain,
/// or absolute distance metric and atom domain on a scalar type, both form valid metric spaces.
pub fn make_identity<D, M>(domain: D, metric: M) -> Fallible<Transformation<D, D, M, M>>
where
    D: Domain,
    D::Carrier: Clone,
    M: Metric,
    M::Distance: Clone,
    (D, M): MetricSpace,
{
    Transformation::new(
        domain.clone(),
        domain,
        Function::new(|arg: &D::Carrier| arg.clone()),
        metric.clone(),
        metric,
        StabilityMap::new(|d_in: &M::Distance| d_in.clone()),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *"),
        input_metric(c_type = "AnyMetric *")
    ),
    generics(TIA(suppress), M(suppress)),
    derived_types(
        TIA = "$get_atom(get_type(input_domain))",
        M = "$get_type(input_metric)"
    )
)]
/// Make a Transformation that checks if each element is equal to `value`.
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
/// * `value` - value to check against
///
/// # Generics
/// * `TIA` - Atomic Input Type. Type of elements in the input vector
pub fn make_is_equal<TIA, M>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: M,
    value: TIA,
) -> Fallible<Transformation<VectorDomain<AtomDomain<TIA>>, VectorDomain<AtomDomain<bool>>, M, M>>
where
    TIA: 'static + PartialEq + CheckAtom,
    M: DatasetMetric,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
{
    make_row_by_row(
        input_domain,
        input_metric,
        AtomDomain::default(),
        move |v| v == &value,
    )
}

#[bootstrap(features("contrib"), generics(DIA(suppress), M(suppress)))]
/// Make a Transformation that checks if each element in a vector is null.
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
///
/// # Generics
/// * `DIA` - Atomic Input Domain. Can be any domain for which the carrier type has a notion of nullity.
pub fn make_is_null<M, DIA>(
    input_domain: VectorDomain<DIA>,
    input_metric: M,
) -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<AtomDomain<bool>>, M, M>>
where
    DIA: Domain + Default,
    DIA::Carrier: 'static + CheckNull,
    M: DatasetMetric,
    (VectorDomain<DIA>, M): MetricSpace,
    (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
{
    make_row_by_row(input_domain, input_metric, AtomDomain::default(), |v| {
        v.is_null()
    })
}

#[cfg(test)]
mod test;
