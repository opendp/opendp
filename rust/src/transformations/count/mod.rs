#[cfg(feature = "ffi")]
mod ffi;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use num::One;
use opendp_derive::bootstrap;

use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, MapDomain, VectorDomain};
use crate::error::*;
use crate::metrics::{AbsoluteDistance, LpDistance, SymmetricDistance};
use crate::traits::{CollectionSize, Hashable, Number, Primitive};

#[bootstrap(features("contrib"), generics(TIA(suppress), TO(default = "int")))]
/// Make a Transformation that computes a count of the number of records in data.
///
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
///
/// # Generics
/// * `TIA` - Atomic Input Type. Input data is expected to be of the form `Vec<TIA>`.
/// * `TO` - Output Type. Must be numeric.
pub fn make_count<TIA, TO>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: SymmetricDistance,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        AtomDomain<TO>,
        SymmetricDistance,
        AbsoluteDistance<TO>,
    >,
>
where
    TIA: Primitive,
    TO: Number,
{
    Transformation::new(
        input_domain,
        AtomDomain::default(),
        // think of this as: min(arg.len(), TO::max_value())
        Function::new(move |arg: &Vec<TIA>| {
            // get size via the CollectionSize trait
            let size = arg.size();

            // cast to TO, and if cast fails (due to overflow) fill with largest value
            TO::exact_int_cast(size).unwrap_or(TO::MAX_CONSECUTIVE)
        }),
        input_metric,
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(TO::one()),
    )
}

#[bootstrap(features("contrib"), generics(TIA(suppress), TO(default = "int")))]
/// Make a Transformation that computes a count of the number of unique, distinct records in data.
///
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
///
/// # Generics
/// * `TIA` - Atomic Input Type. Input data is expected to be of the form `Vec<TIA>`.
/// * `TO` - Output Type. Must be numeric.
pub fn make_count_distinct<TIA, TO>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: SymmetricDistance,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        AtomDomain<TO>,
        SymmetricDistance,
        AbsoluteDistance<TO>,
    >,
>
where
    TIA: Hashable,
    TO: Number,
{
    Transformation::new(
        input_domain,
        AtomDomain::default(),
        Function::new(move |arg: &Vec<TIA>| {
            let len = arg.iter().collect::<HashSet<_>>().len();
            TO::exact_int_cast(len).unwrap_or(TO::MAX_CONSECUTIVE)
        }),
        input_metric,
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(TO::one()),
    )
}

#[doc(hidden)]
pub trait CountByCategoriesConstant<QO> {
    fn get_stability_constant() -> QO;
}
impl<const P: usize, Q: One> CountByCategoriesConstant<Q> for LpDistance<P, Q> {
    fn get_stability_constant() -> Q {
        Q::one()
    }
}

#[bootstrap(
    features("contrib"),
    arguments(null_category(default = true)),
    generics(
        MO(hint = "SensitivityMetric", default = "L1Distance<int>"),
        TIA(suppress),
        TOA(default = "int")
    ),
    derived_types(TIA = "$get_atom(get_type(input_domain))")
)]
/// Make a Transformation that computes the number of times each category appears in the data.
/// This assumes that the category set is known.
///
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
/// * [BV17 Differential Privacy on Finite Computers](https://arxiv.org/abs/1709.05396)
///
/// # Arguments
/// * `categories` - The set of categories to compute counts for.
/// * `null_category` - Include a count of the number of elements that were not in the category set at the end of the vector.
///
/// # Generics
/// * `MO` - Output Metric.
/// * `TIA` - Atomic Input Type that is categorical/hashable. Input data must be `Vec<TIA>`
/// * `TOA` - Atomic Output Type that is numeric.
///
/// # Returns
/// The carrier type is `Vec<TOA>`, a vector of the counts (`TOA`).
pub fn make_count_by_categories<MO, TIA, TOA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: SymmetricDistance,
    categories: Vec<TIA>,
    null_category: bool,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<AtomDomain<TOA>>,
        SymmetricDistance,
        MO,
    >,
>
where
    MO: CountByCategoriesConstant<MO::Distance> + Metric,
    MO::Distance: Number,
    TIA: Hashable,
    TOA: Number,
    (VectorDomain<AtomDomain<TIA>>, SymmetricDistance): MetricSpace,
    (VectorDomain<AtomDomain<TOA>>, MO): MetricSpace,
{
    let mut uniques = HashSet::new();
    if categories.iter().any(move |x| !uniques.insert(x)) {
        return fallible!(MakeTransformation, "categories must be distinct");
    }
    Transformation::new(
        input_domain,
        VectorDomain::new(AtomDomain::default()),
        Function::new(move |data: &Vec<TIA>| {
            let mut counts = categories
                .iter()
                .map(|cat| (cat, TOA::zero()))
                .collect::<HashMap<&TIA, TOA>>();
            let mut null_count = TOA::zero();

            data.iter().for_each(|v| {
                let count = match counts.entry(v) {
                    Entry::Occupied(v) => v.into_mut(),
                    Entry::Vacant(_v) => &mut null_count,
                };
                *count = TOA::one().saturating_add(count)
            });

            categories
                .iter()
                .map(|cat| {
                    counts
                        .remove(cat)
                        .unwrap_assert("categories are distinct and every category is in the map")
                })
                .chain(if null_category {
                    vec![null_count]
                } else {
                    vec![]
                })
                .collect()
        }),
        input_metric,
        MO::default(),
        StabilityMap::new_from_constant(MO::get_stability_constant()),
    )
}

#[doc(hidden)]
pub trait CountByConstant<QO> {
    fn get_stability_constant() -> Fallible<QO>;
}
impl<const P: usize, Q: One> CountByConstant<Q> for LpDistance<P, Q> {
    fn get_stability_constant() -> Fallible<Q> {
        if P == 0 {
            return fallible!(MakeTransformation, "P must be positive");
        }
        Ok(Q::one())
    }
}

#[bootstrap(
    features("contrib"),
    generics(MO(hint = "SensitivityMetric"), TK(suppress), TV(default = "int"))
)]
/// Make a Transformation that computes the count of each unique value in data.
/// This assumes that the category set is unknown.
///
/// # Citations
/// * [BV17 Differential Privacy on Finite Computers](https://arxiv.org/abs/1709.05396)
///
/// # Generics
/// * `MO` - Output Metric.
/// * `TK` - Type of Key. Categorical/hashable input data type. Input data must be `Vec<TK>`.
/// * `TV` - Type of Value. Express counts in terms of this integral type.
///
/// # Returns
/// The carrier type is `HashMap<TK, TV>`, a hashmap of the count (`TV`) for each unique data input (`TK`).
pub fn make_count_by<MO, TK, TV>(
    input_domain: VectorDomain<AtomDomain<TK>>,
    input_metric: SymmetricDistance,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TK>>,
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        SymmetricDistance,
        MO,
    >,
>
where
    MO: CountByConstant<MO::Distance> + Metric,
    MO::Distance: Number,
    TK: Hashable,
    TV: Number,
    (VectorDomain<AtomDomain<TK>>, SymmetricDistance): MetricSpace,
    (MapDomain<AtomDomain<TK>, AtomDomain<TV>>, MO): MetricSpace,
{
    Transformation::new(
        input_domain.clone(),
        MapDomain::new(input_domain.element_domain, AtomDomain::default()),
        Function::new(move |data: &Vec<TK>| {
            let mut counts = HashMap::new();
            data.iter().for_each(|v| {
                let count = counts.entry(v.clone()).or_insert_with(TV::zero);
                *count = TV::one().saturating_add(count);
            });
            counts
        }),
        input_metric,
        MO::default(),
        StabilityMap::new_from_constant(MO::get_stability_constant()?),
    )
}

#[cfg(test)]
mod test;
