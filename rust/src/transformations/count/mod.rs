#[cfg(feature="ffi")]
mod ffi;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;

use num::One;
use opendp_derive::bootstrap;

use crate::core::{Function, Metric, StabilityMap, Transformation};
use crate::metrics::{AbsoluteDistance, SymmetricDistance, LpDistance};
use crate::domains::{AllDomain, MapDomain, VectorDomain};
use crate::error::*;
use crate::traits::{Number, Hashable, Primitive, Float, CollectionSize};

#[bootstrap(features("contrib"), generics(TO(default = "int")))]
/// Make a Transformation that computes a count of the number of records in data.
/// 
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
/// 
/// # Generics
/// * `TIA` - Atomic Input Type. Input data is expected to be of the form `Vec<TIA>`.
/// * `TO` - Output Type. Must be numeric.
pub fn make_count<TIA, TO>(
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, AllDomain<TO>, SymmetricDistance, AbsoluteDistance<TO>>>
    where TIA: Primitive,
          TO: Number {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        // think of this as: min(arg.len(), TO::max_value())
        Function::new(move |arg: &Vec<TIA>| {
            // get size via the CollectionDomain trait
            let size = arg.size();
            
            // cast to TO, and if cast fails (due to overflow) fill with largest value
            TO::exact_int_cast(size).unwrap_or(TO::MAX_CONSECUTIVE)
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(TO::one())))
}


#[bootstrap(features("contrib"), generics(TO(default = "int")))]
/// Make a Transformation that computes a count of the number of unique, distinct records in data.
/// 
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
/// 
/// # Generics
/// * `TIA` - Atomic Input Type. Input data is expected to be of the form Vec<TIA>.
/// * `TO` - Output Type. Must be numeric.
pub fn make_count_distinct<TIA, TO>(
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, AllDomain<TO>, SymmetricDistance, AbsoluteDistance<TO>>>
    where TIA: Hashable,
          TO: Number {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new(move |arg: &Vec<TIA>| {
            let len = arg.iter().collect::<HashSet<_>>().len();
            TO::exact_int_cast(len).unwrap_or(TO::MAX_CONSECUTIVE)
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(TO::one())))
}

#[doc(hidden)]
pub trait CountByCategoriesConstant<QO> {
    fn get_stability_constant() -> QO;
}
impl<const P: usize, Q: One> CountByCategoriesConstant<Q> for LpDistance<P, Q> {
    fn get_stability_constant() -> Q { Q::one() }
}

#[bootstrap(
    features("contrib"), 
    arguments(
        null_category(default = true)),
    generics(
        MO(hint = "SensitivityMetric", default = "L1Distance<int>"), 
        TOA(default = "int"))
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
/// The carrier type is `HashMap<TK, TV>`, a hashmap of the count (`TV`) for each unique data input (`TK`).
pub fn make_count_by_categories<MO, TIA, TOA>(
    categories: Vec<TIA>,
    null_category: bool
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, VectorDomain<AllDomain<TOA>>, SymmetricDistance, MO>>
    where MO: CountByCategoriesConstant<MO::Distance> + Metric,
          MO::Distance: Number,
          TIA: Hashable,
          TOA: Number {
    let mut uniques = HashSet::new();
    if categories.iter().any(move |x| !uniques.insert(x)) {
        return fallible!(MakeTransformation, "categories must be distinct")
    }
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |data: &Vec<TIA>| {
            let mut counts = categories.iter()
                .map(|cat| (cat, TOA::zero())).collect::<HashMap<&TIA, TOA>>();
            let mut null_count = TOA::zero();

            data.iter().for_each(|v| {
                let count = match counts.entry(v) {
                    Entry::Occupied(v) => v.into_mut(),
                    Entry::Vacant(_v) => &mut null_count
                };
                *count = TOA::one().saturating_add(count)
            });

            categories.iter().map(|cat| counts.remove(cat)
                .unwrap_assert("categories are distinct and every category is in the map"))
                .chain(if null_category {vec![null_count]} else {vec![]})
                .collect()
        }),
        SymmetricDistance::default(),
        MO::default(),
        StabilityMap::new_from_constant(MO::get_stability_constant())))
}

#[doc(hidden)]
pub trait CountByConstant<QO> {
    fn get_stability_constant() -> Fallible<QO>;
}
impl<const P: usize, Q: One> CountByConstant<Q> for LpDistance<P, Q> {
    fn get_stability_constant() -> Fallible<Q> {
        if P == 0 {return fallible!(MakeTransformation, "P must be positive")}
        Ok(Q::one())
    }
}


#[bootstrap(
    features("contrib"), 
    generics(MO(hint = "SensitivityMetric"), TV(default = "int"))
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
) -> Fallible<Transformation<VectorDomain<AllDomain<TK>>, MapDomain<AllDomain<TK>, AllDomain<TV>>, SymmetricDistance, MO>>
    where MO: CountByConstant<MO::Distance> + Metric,
          MO::Distance: Float,
          TK: Hashable,
          TV: Number {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        MapDomain::new(AllDomain::new(), AllDomain::new()),
        Function::new(move |data: &Vec<TK>| {
            let mut counts = HashMap::new();
            data.iter().for_each(|v| {
                let count = counts.entry(v.clone()).or_insert_with(TV::zero);
                *count = TV::one().saturating_add(count);
            });
            counts
        }),
        SymmetricDistance::default(),
        MO::default(),
        StabilityMap::new_from_constant(MO::get_stability_constant()?)))
}


#[cfg(test)]
mod tests {
    use crate::metrics::L2Distance;
    use crate::transformations::count::make_count_by_categories;

    use super::*;

    #[test]
    fn test_make_count_l1() {
        let transformation = make_count::<i64, u32>().unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_l2() {
        let transformation = make_count::<u32, i32>().unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_distinct() {
        let transformation = make_count_distinct::<_, i32>().unwrap_test();
        let arg = vec![1, 1, 3, 4, 4];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 3 ;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_by_categories() {
        let transformation = make_count_by_categories::<L2Distance<f64>, i64, i8>(
            vec![2, 1, 3], true
        ).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5, 1, 1, 1, 2];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = vec![2, 4, 1, 2];
        assert_eq!(ret, expected);

        assert!(!transformation.check(&5, &4.999).unwrap_test());
        assert!(transformation.check(&5, &5.0).unwrap_test());
    }

    #[test]
    fn test_make_count_by() -> Fallible<()> {
        let arg = vec![true, true, true, false, true, false, false, false, true, true];
        let transformation = make_count_by::<L2Distance<f64>, bool, i8>()?;
        let ret = transformation.invoke(&arg)?;
        let mut expected = HashMap::new();
        expected.insert(true, 6);
        expected.insert(false, 4);
        assert_eq!(ret, expected);
        assert!(transformation.check(&6, &6.)?);
        Ok(())
    }
}