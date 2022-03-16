#[cfg(feature="ffi")]
mod ffi;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::hash::Hash;

use num::{One, Zero, Float};

use crate::core::{Function, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{AbsoluteDistance, SymmetricDistance, LpDistance, IntDistance};
use crate::dom::{AllDomain, MapDomain, VectorDomain};
use crate::error::*;
use crate::traits::{DistanceConstant, InfCast, ExactIntCast, SaturatingAdd, CheckNull};

pub fn make_count<TIA, TO>(
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, AllDomain<TO>, SymmetricDistance, AbsoluteDistance<TO>>>
    where TIA: CheckNull,
          TO: ExactIntCast<usize> + One + DistanceConstant<IntDistance> + CheckNull,
          IntDistance: InfCast<TO> {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        // think of this as: min(arg.len(), TO::max_value())
        Function::new(move |arg: &Vec<TIA>|
            TO::exact_int_cast(arg.len()).unwrap_or(TO::MAX_CONSECUTIVE)),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(TO::one())))
}


pub fn make_count_distinct<TIA, TO>(
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, AllDomain<TO>, SymmetricDistance, AbsoluteDistance<TO>>>
    where TIA: Eq + Hash + CheckNull,
          TO: ExactIntCast<usize> + One + DistanceConstant<IntDistance> + CheckNull,
          IntDistance: InfCast<TO> {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new(move |arg: &Vec<TIA>| {
            let len = arg.iter().collect::<HashSet<_>>().len();
            TO::exact_int_cast(len).unwrap_or(TO::MAX_CONSECUTIVE)
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(TO::one())))
}

pub trait CountByCategoriesConstant<QO> {
    fn get_stability_constant() -> QO;
}
impl<Q: One, const P: usize> CountByCategoriesConstant<Q> for LpDistance<Q, P> {
    fn get_stability_constant() -> Q { Q::one() }
}

// count with unknown n, known categories
pub fn make_count_by_categories<MO, TI, TO>(
    categories: Vec<TI>
) -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, VectorDomain<AllDomain<TO>>, SymmetricDistance, MO>>
    where MO: CountByCategoriesConstant<MO::Distance> + SensitivityMetric,
          MO::Distance: DistanceConstant<IntDistance> + One,
          TI: 'static + Eq + Hash + CheckNull,
          TO: Zero + One + SaturatingAdd + CheckNull,
          IntDistance: InfCast<MO::Distance>{
    let mut uniques = HashSet::new();
    if categories.iter().any(move |x| !uniques.insert(x)) {
        return fallible!(MakeTransformation, "categories must be distinct")
    }
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |data: &Vec<TI>| {
            let mut counts = categories.iter()
                .map(|cat| (cat, TO::zero())).collect::<HashMap<&TI, TO>>();
            let mut null_count = TO::zero();

            data.iter().for_each(|v| {
                let count = match counts.entry(v) {
                    Entry::Occupied(v) => v.into_mut(),
                    Entry::Vacant(_v) => &mut null_count
                };
                *count = TO::one().saturating_add(count)
            });

            categories.iter().map(|cat| counts.remove(cat)
                .unwrap_assert("categories are distinct and every category is in the map"))
                .chain(vec![null_count])
                .collect()
        }),
        SymmetricDistance::default(),
        MO::default(),
        StabilityRelation::new_from_constant(MO::get_stability_constant())))
}

pub trait CountByConstant<QO> {
    fn get_stability_constant() -> Fallible<QO>;
}
impl<Q: One + Float + ExactIntCast<usize>, const P: usize> CountByConstant<Q> for LpDistance<Q, P> {
    fn get_stability_constant() -> Fallible<Q> {
        if P == 0 {return fallible!(MakeTransformation, "P must be positive")}
        let p = Q::exact_int_cast(P)?;
        let _2 = Q::exact_int_cast(2)?;
        Ok(_2.powf(Q::one() / p - Q::one()))
    }
}

// count with known n, unknown categories
pub fn make_count_by<MO, TK, TV>(
) -> Fallible<Transformation<VectorDomain<AllDomain<TK>>, MapDomain<AllDomain<TK>, AllDomain<TV>>, SymmetricDistance, MO>>
    where MO: CountByConstant<MO::Distance> + SensitivityMetric,
          MO::Distance: DistanceConstant<IntDistance>,
          TK: 'static + Eq + Hash + Clone + CheckNull,
          TV: Zero + One + SaturatingAdd + CheckNull,
          IntDistance: InfCast<MO::Distance> {
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
        StabilityRelation::new_from_constant(MO::get_stability_constant()?)))
}


#[cfg(test)]
mod tests {
    use crate::dist::{L2Distance};
    use crate::trans::count::make_count_by_categories;

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
            vec![2, 1, 3]
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
        assert!(!transformation.check(&6, &4.2426)?);
        assert!(transformation.check(&6, &4.24265)?);
        Ok(())
    }
}