#[cfg(feature = "ffi")]
mod ffi;

use std::collections::HashMap;
use std::iter::FromIterator;

use opendp_derive::bootstrap;

use crate::core::{MetricSpace, Transformation};
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{Hashable, Number, Primitive};
use crate::transformations::make_row_by_row;

use super::DatasetMetric;

#[bootstrap(
    features("contrib"),
    generics(TIA(suppress), M(suppress)),
    derived_types(TIA = "$get_atom(get_type(input_domain))")
)]
/// Find the index of a data value in a set of categories.
///
/// For each value in the input vector, finds the index of the value in `categories`.
/// If an index is found, returns `Some(index)`, else `None`.
/// Chain with `make_impute_constant` or `make_drop_null` to handle nullity.
///
/// # Arguments
/// * `input_domain` - The domain of the input vector.
/// * `input_metric` - The metric of the input vector.
/// * `categories` - The set of categories to find indexes from.
///
/// # Generics
/// * `M` - Metric Type
/// * `TIA` - Atomic Input Type that is categorical/hashable
pub fn make_find<M, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: M,
    categories: Vec<TIA>,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<OptionDomain<AtomDomain<usize>>>,
        M,
        M,
    >,
>
where
    TIA: Hashable,
    M: DatasetMetric,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<OptionDomain<AtomDomain<usize>>>, M): MetricSpace,
{
    let categories_len = categories.len();
    let indexes =
        HashMap::<TIA, usize>::from_iter(categories.into_iter().enumerate().map(|(i, v)| (v, i)));

    if indexes.len() != categories_len {
        return fallible!(MakeTransformation, "categories must be unique");
    }

    make_row_by_row(
        input_domain,
        input_metric,
        OptionDomain::new(AtomDomain::default()),
        move |v| indexes.get(v).cloned(),
    )
}

#[bootstrap(
    features("contrib"),
    generics(TIA(suppress), M(suppress)),
    derived_types(TIA = "$get_atom(get_type(input_domain))")
)]
/// Make a transformation that finds the bin index in a monotonically increasing vector of edges.
///
/// For each value in the input vector, finds the index of the bin the value falls into.
/// `edges` splits the entire range of `TIA` into bins.
/// The first bin at index zero ranges from negative infinity to the first edge, non-inclusive.
/// The last bin at index `edges.len()` ranges from the last bin, inclusive, to positive infinity.
///
/// To be valid, `edges` must be unique and ordered.
/// `edges` are left inclusive, right exclusive.
///
/// # Arguments
/// * `input_domain` - The domain of the input vector.
/// * `input_metric` - The metric of the input vector.
/// * `edges` - The set of edges to split bins by.
///
/// # Generics
/// * `M` - Metric Type
/// * `TIA` - Atomic Input Type that is numeric
pub fn make_find_bin<M, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: M,
    edges: Vec<TIA>,
) -> Fallible<Transformation<VectorDomain<AtomDomain<TIA>>, VectorDomain<AtomDomain<usize>>, M, M>>
where
    TIA: Number,
    M: DatasetMetric,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<usize>>, M): MetricSpace,
{
    if !edges.windows(2).all(|pair| pair[0] < pair[1]) {
        return fallible!(MakeTransformation, "edges must be unique and ordered");
    }
    make_row_by_row(
        input_domain,
        input_metric,
        AtomDomain::default(),
        move |v| {
            edges
                .iter()
                .enumerate()
                .find(|(_, edge)| v < edge)
                .map(|(i, _)| i)
                .unwrap_or(edges.len())
        },
    )
}

#[bootstrap(features("contrib"), generics(M(suppress)))]
/// Make a transformation that treats each element as an index into a vector of categories.
///
/// # Arguments
/// * `input_domain` - The domain of the input vector.
/// * `input_metric` - The metric of the input vector.
/// * `categories` - The set of categories to index into.
/// * `null` - Category to return if the index is out-of-range of the category set.
///
/// # Generics
/// * `M` - Metric Type
/// * `TOA` - Atomic Output Type. Output data will be `Vec<TOA>`.
pub fn make_index<M, TOA>(
    input_domain: VectorDomain<AtomDomain<usize>>,
    input_metric: M,
    categories: Vec<TOA>,
    null: TOA,
) -> Fallible<Transformation<VectorDomain<AtomDomain<usize>>, VectorDomain<AtomDomain<TOA>>, M, M>>
where
    TOA: Primitive,
    M: DatasetMetric,
    (VectorDomain<AtomDomain<usize>>, M): MetricSpace,
    (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
{
    make_row_by_row(
        input_domain,
        input_metric,
        AtomDomain::default(),
        move |v| categories.get(*v).unwrap_or(&null).clone(),
    )
}

#[cfg(test)]
mod test;
