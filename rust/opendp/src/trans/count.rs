use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::convert::TryFrom;
use std::hash::Hash;
use std::ops::AddAssign;

use num::{Bounded, Integer, One, Zero};

use crate::core::{Function, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{AbsoluteDistance, SymmetricDistance, LpDistance, IntDistance};
use crate::dom::{AllDomain, MapDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{DistanceConstant, MaxConsecutiveInt, ExactCast, InfCast};

pub fn make_count<TIA, TO>(
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, AllDomain<TO>, SymmetricDistance, AbsoluteDistance<TO>>>
    where TO: ExactCast<usize> + One + DistanceConstant<IntDistance> + MaxConsecutiveInt, IntDistance: InfCast<TO> {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        // think of this as: min(arg.len(), TO::max_value())
        Function::new(move |arg: &Vec<TIA>|
            TO::exact_cast(arg.len()).unwrap_or(TO::MAX_CONSECUTIVE)),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(TO::one())))
}


pub fn make_count_distinct<TIA, TO>(
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, AllDomain<TO>, SymmetricDistance, AbsoluteDistance<TO>>>
    where TIA: Eq + Hash,
          TO: TryFrom<usize> + Bounded + One + DistanceConstant<IntDistance>,
          IntDistance: InfCast<TO> {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new(move |arg: &Vec<TIA>| {
            let len = arg.iter().collect::<HashSet<_>>().len();
            TO::try_from(len).unwrap_or(TO::max_value())
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(TO::one())))
}

pub trait CountByConstant<QO> {
    fn get_stability_constant() -> QO;
}
impl<Q: One, const P: usize> CountByConstant<Q> for LpDistance<Q, P> {
    fn get_stability_constant() -> Q { Q::one() }
}

// count with unknown n, known categories
pub fn make_count_by_categories<MO, TI, TO>(
    categories: Vec<TI>
) -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, SymmetricDistance, MO>>
    where MO: CountByConstant<MO::Distance> + SensitivityMetric,
          MO::Distance: DistanceConstant<IntDistance> + One,
          TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign,
          IntDistance: InfCast<MO::Distance>{
    let mut uniques = HashSet::new();
    if categories.iter().any(move |x| !uniques.insert(x)) {
        return fallible!(MakeTransformation, "categories must be distinct")
    }
    Ok(Transformation::new(
        VectorDomain::new_all(),
        SizedDomain::new(VectorDomain::new_all(), categories.len() + 1),
        Function::new(move |data: &Vec<TI>| {
            let mut counts = categories.iter()
                .map(|cat| (cat, TO::zero())).collect::<HashMap<&TI, TO>>();
            let mut null_count = TO::zero();

            data.iter().for_each(|v|
                *match counts.entry(v) {
                    Entry::Occupied(v) => v.into_mut(),
                    Entry::Vacant(_v) => &mut null_count
                } += TO::one());

            categories.iter().map(|cat| counts.remove(cat)
                .unwrap_assert("categories are distinct and every category is in the map"))
                .chain(vec![null_count])
                .collect()
        }),
        SymmetricDistance::default(),
        MO::default(),
        StabilityRelation::new_from_constant(MO::get_stability_constant())))
}

// count with known n, unknown categories
// This implementation could be made tighter with the relation in the spreadsheet for known n.
// Need to double-check if stability-based histograms have any additional stability requirements.
pub fn make_count_by<MO, TI, TO>(
    n: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, SymmetricDistance, MO>>
    where MO: CountByConstant<MO::Distance> + SensitivityMetric,
          MO::Distance: DistanceConstant<IntDistance>,
          TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign,
          IntDistance: InfCast<MO::Distance> {
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new_all(), n),
        SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
        Function::new(move |data: &Vec<TI>| {
            let mut counts = HashMap::new();
            data.iter().for_each(|v|
                *counts.entry(v.clone()).or_insert_with(TO::zero) += TO::one()
            );
            counts
        }),
        SymmetricDistance::default(),
        MO::default(),
        StabilityRelation::new_from_constant(MO::get_stability_constant())))
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
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_l2() {
        let transformation = make_count::<u32, i32>().unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_distinct() {
        let transformation = make_count_distinct::<_, i32>().unwrap_test();
        let arg = vec![1, 1, 3, 4, 4];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3 ;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_by_categories() {
        let transformation = make_count_by_categories::<L2Distance<f64>, i64, i8>(
            vec![2, 1, 3]
        ).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5, 1, 1, 1, 2];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = vec![2, 4, 1, 2];
        assert_eq!(ret, expected);

        assert!(!transformation.stability_relation.eval(&5, &4.999).unwrap_test());
        assert!(transformation.stability_relation.eval(&5, &5.0).unwrap_test());
    }

    #[test]
    fn test_make_count_by() -> Fallible<()> {
        let arg = vec![true, true, true, false, true, false, false, false, true, true];
        let transformation = make_count_by::<L2Distance<f64>, bool, i8>(arg.len())?;
        let ret = transformation.function.eval(&arg)?;
        let mut expected = HashMap::new();
        expected.insert(true, 6);
        expected.insert(false, 4);
        assert_eq!(ret, expected);
        assert!(!transformation.stability_relation.eval(&5, &4.999)?);
        assert!(transformation.stability_relation.eval(&5, &5.0)?);
        Ok(())
    }
}