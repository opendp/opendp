#[cfg(feature = "ffi")]
mod ffi;

use std::collections::HashMap;
use std::iter::FromIterator;

use opendp_derive::bootstrap;

use crate::core::Transformation;
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::error::Fallible;
use crate::metrics::SymmetricDistance;
use crate::traits::{Hashable, Number, Primitive};
use crate::transformations::make_row_by_row;

#[bootstrap(features("contrib"))]
/// Find the index of a data value in a set of categories.
///
/// For each value in the input vector, finds the index of the value in `categories`.
/// If an index is found, returns `Some(index)`, else `None`.
/// Chain with `make_impute_constant` or `make_drop_null` to handle nullity.
///
/// # Arguments
/// * `categories` - The set of categories to find indexes from.
///
/// # Generics
/// * `TIA` - Atomic Input Type that is categorical/hashable
pub fn make_find<TIA>(
    categories: Vec<TIA>,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<OptionDomain<AtomDomain<usize>>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    TIA: Hashable,
{
    let categories_len = categories.len();
    let indexes =
        HashMap::<TIA, usize>::from_iter(categories.into_iter().enumerate().map(|(i, v)| (v, i)));

    if indexes.len() != categories_len {
        return fallible!(MakeTransformation, "categories must be unique");
    }

    make_row_by_row(
        AtomDomain::default(),
        OptionDomain::new(AtomDomain::default()),
        move |v| indexes.get(v).cloned(),
    )
}

#[bootstrap(features("contrib"))]
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
/// * `edges` - The set of edges to split bins by.
///
/// # Generics
/// * `TIA` - Atomic Input Type that is numeric
pub fn make_find_bin<TIA>(
    edges: Vec<TIA>,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<AtomDomain<usize>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    TIA: Number,
{
    if !edges.windows(2).all(|pair| pair[0] < pair[1]) {
        return fallible!(MakeTransformation, "edges must be unique and ordered");
    }
    make_row_by_row(AtomDomain::default(), AtomDomain::default(), move |v| {
        edges
            .iter()
            .enumerate()
            .find(|(_, edge)| v < edge)
            .map(|(i, _)| i)
            .unwrap_or(edges.len())
    })
}

#[bootstrap(features("contrib"))]
/// Make a transformation that treats each element as an index into a vector of categories.
///
/// # Arguments
/// * `categories` - The set of categories to index into.
/// * `null` - Category to return if the index is out-of-range of the category set.
///
/// # Generics
/// * `TOA` - Atomic Output Type. Output data will be `Vec<TOA>`.
pub fn make_index<TOA>(
    categories: Vec<TOA>,
    null: TOA,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<usize>>,
        VectorDomain<AtomDomain<TOA>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    TOA: Primitive,
{
    make_row_by_row(AtomDomain::default(), AtomDomain::default(), move |v| {
        categories.get(*v).unwrap_or(&null).clone()
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find() -> Fallible<()> {
        let find = make_find(vec!["1", "3", "4"])?;
        assert_eq!(
            find.invoke(&vec!["1", "2", "3", "4", "5"])?,
            vec![Some(0), None, Some(1), Some(2), None]
        );
        Ok(())
    }

    #[test]
    fn test_bin() -> Fallible<()> {
        let bin = make_find_bin(vec![2, 3, 5])?;
        assert_eq!(
            bin.invoke(&(1..10).collect())?,
            vec![0, 1, 2, 2, 3, 3, 3, 3, 3]
        );
        Ok(())
    }

    #[test]
    fn test_index() -> Fallible<()> {
        let index = make_index(vec!["A", "B", "C"], "NA")?;
        assert_eq!(
            index.invoke(&vec![0, 1, 3, 1, 5])?,
            vec!["A", "B", "NA", "B", "NA"]
        );
        Ok(())
    }
}
