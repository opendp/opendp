use opendp_derive::bootstrap;

use crate::{
    combinators::IsSizedDomain,
    core::{Domain, Function, MetricSpace, StabilityMap, Transformation},
    error::Fallible,
    metrics::IntDistance,
    traits::samplers::Shuffle,
};

use self::traits::{BoundedMetric, OrderedMetric, UnboundedMetric, UnorderedMetric};

#[cfg(feature = "ffi")]
mod ffi;
pub(crate) mod traits;

#[bootstrap(features("contrib"), generics(D(suppress), MI(suppress)))]
/// Make a Transformation that converts the unordered dataset metric `SymmetricDistance`
/// to the respective ordered dataset metric `InsertDeleteDistance` by assigning a random permutation.
///
/// | `MI`              | `MI::OrderedMetric`  |
/// | ----------------- | -------------------- |
/// | SymmetricDistance | InsertDeleteDistance |
/// | ChangeOneDistance | HammingDistance      |
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
///
/// # Generics
/// * `D` - Domain
/// * `MI` - Input Metric
pub fn make_ordered_random<D, MI>(
    input_domain: D,
    input_metric: MI,
) -> Fallible<Transformation<D, D, MI, MI::OrderedMetric>>
where
    D: Domain,
    D::Carrier: Clone + Shuffle,
    MI: UnorderedMetric<Distance = IntDistance>,
    (D, MI): MetricSpace,
    (D, MI::OrderedMetric): MetricSpace,
{
    Transformation::new(
        input_domain.clone(),
        input_domain,
        Function::new_fallible(|arg: &D::Carrier| {
            let mut data = arg.clone();
            data.shuffle()?;
            Ok(data)
        }),
        input_metric,
        MI::OrderedMetric::default(),
        StabilityMap::new_from_constant(1),
    )
}

#[bootstrap(features("contrib"), generics(D(suppress), MI(suppress)))]
/// Make a Transformation that converts the ordered dataset metric `MI`
/// to the respective ordered dataset metric with a no-op.
///
/// | `MI`                 | `MI::UnorderedMetric` |
/// | -------------------- | --------------------- |
/// | InsertDeleteDistance | SymmetricDistance     |
/// | HammingDistance      | ChangeOneDistance     |
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
///
/// # Generics
/// * `D` - Domain
/// * `MI` - Input Metric
pub fn make_unordered<D, MI>(
    input_domain: D,
    input_metric: MI,
) -> Fallible<Transformation<D, D, MI, MI::UnorderedMetric>>
where
    D: Domain,
    D::Carrier: Clone,
    MI: OrderedMetric<Distance = IntDistance>,
    (D, MI): MetricSpace,
    (D, MI::UnorderedMetric): MetricSpace,
{
    Transformation::new(
        input_domain.clone(),
        input_domain,
        Function::new(|val: &D::Carrier| val.clone()),
        input_metric,
        MI::UnorderedMetric::default(),
        StabilityMap::new_from_constant(1),
    )
}

#[bootstrap(features("contrib"), generics(D(suppress), MI(suppress)))]
/// Make a Transformation that converts the bounded dataset metric `MI`
/// to the respective unbounded dataset metric with a no-op.
///
/// | `MI`              | `MI::UnboundedMetric` |
/// | ----------------- | --------------------- |
/// | ChangeOneDistance | SymmetricDistance     |
/// | HammingDistance   | InsertDeleteDistance  |
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
/// * `size` - Number of records in input data.
///
/// # Generics
/// * `D` - Domain. The function is a no-op so input and output domains are the same.
/// * `MI` - Input Metric.
pub fn make_metric_unbounded<D, MI>(
    input_domain: D,
    input_metric: MI,
) -> Fallible<Transformation<D, D, MI, MI::UnboundedMetric>>
where
    D: IsSizedDomain,
    D::Carrier: Clone,
    MI: BoundedMetric<Distance = IntDistance>,
    (D, MI): MetricSpace,
    (D, MI::UnboundedMetric): MetricSpace,
{
    input_domain.get_size()?;
    Transformation::new(
        input_domain.clone(),
        input_domain,
        Function::new(|arg: &D::Carrier| arg.clone()),
        input_metric,
        MI::UnboundedMetric::default(),
        StabilityMap::new(|d_in| d_in * 2),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(domain(c_type = "AnyDomain *")),
    generics(D(suppress), MI(suppress))
)]
/// Make a Transformation that converts the unbounded dataset metric `MI`
/// to the respective bounded dataset metric with a no-op.
///
/// The constructor enforces that the input domain has known size,
/// because it must have known size to be valid under a bounded dataset metric.
///
/// | `MI`                 | `MI::BoundedMetric` |
/// | -------------------- | ------------------- |
/// | SymmetricDistance    | ChangeOneDistance   |
/// | InsertDeleteDistance | HammingDistance     |
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
/// * `size` - Number of records in input data.
///
/// # Generics
/// * `D` - Domain
/// * `MI` - Input Metric
pub fn make_metric_bounded<D, MI>(
    input_domain: D,
    input_metric: MI,
) -> Fallible<Transformation<D, D, MI, MI::BoundedMetric>>
where
    D: IsSizedDomain,
    D::Carrier: Clone,
    MI: UnboundedMetric<Distance = IntDistance>,
    (D, MI): MetricSpace,
    (D, MI::BoundedMetric): MetricSpace,
{
    input_domain.get_size()?;
    Transformation::new(
        input_domain.clone(),
        input_domain,
        Function::new(|arg: &D::Carrier| arg.clone()),
        input_metric,
        MI::BoundedMetric::default(),
        StabilityMap::new(|d_in| d_in / 2),
    )
}

#[cfg(test)]
mod test;
