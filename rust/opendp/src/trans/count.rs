use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::convert::TryFrom;
use std::hash::Hash;
use std::ops::AddAssign;

use num::{Bounded, Integer, NumCast, One, Zero};
use num::traits::FloatConst;

use crate::core::{DatasetMetric, Function, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{AbsoluteDistance, HammingDistance, L1Distance, L2Distance, SymmetricDistance};
use crate::dom::{AllDomain, MapDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::DistanceConstant;

pub fn make_count<MI, TI, TO>() -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, AllDomain<TO>, MI, AbsoluteDistance<TO>>>
    where MI: DatasetMetric,
          TO: TryFrom<usize> + Bounded + One + DistanceConstant {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        // min(arg.len(), u32::MAX)
        Function::new(move |arg: &Vec<TI>| TO::try_from(arg.len()).unwrap_or(TO::max_value())),
        MI::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(TO::one())))
}


pub fn make_count_distinct<MI, MO, TI>() -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, AllDomain<MO::Distance>, MI, MO>>
    where MI: DatasetMetric,
          MO: SensitivityMetric,
          MO::Distance: TryFrom<usize> + Bounded + One + DistanceConstant,
          TI: Eq + Hash {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new(move |arg: &Vec<TI>| {
            let len = arg.iter().collect::<HashSet<_>>().len();
            MO::Distance::try_from(len).unwrap_or(MO::Distance::max_value())
        }),
        MI::default(),
        MO::default(),
        StabilityRelation::new_from_constant(MO::Distance::one())))
}

pub trait CountByConstant<MI: DatasetMetric, MO: SensitivityMetric> {
    fn get_stability_constant() -> Fallible<MO::Distance>;
}

impl<QO: NumCast> CountByConstant<HammingDistance, L1Distance<QO>> for (HammingDistance, L1Distance<QO>) {
    fn get_stability_constant() -> Fallible<QO> {
        num_cast!(2.; QO)
    }
}

impl<QO: FloatConst> CountByConstant<HammingDistance, L2Distance<QO>> for (HammingDistance, L2Distance<QO>) {
    fn get_stability_constant() -> Fallible<QO> {
        Ok(QO::SQRT_2())
    }
}

impl<MO: SensitivityMetric> CountByConstant<SymmetricDistance, MO> for (SymmetricDistance, MO)
    where MO::Distance: One {
    fn get_stability_constant() -> Fallible<MO::Distance> {
        Ok(MO::Distance::one())
    }
}

// count with unknown n, known categories
pub fn make_count_by_categories<MI, MO, TI, TO>(categories: Vec<TI>) -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, MI, MO>>
    where MI: DatasetMetric,
          MO: SensitivityMetric,
          MO::Distance: DistanceConstant,
          TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign,
          (MI, MO): CountByConstant<MI, MO> {
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
        MI::default(),
        MO::default(),
        StabilityRelation::new_from_constant(<(MI, MO)>::get_stability_constant()?)))
}

// count with known n, unknown categories
pub fn make_count_by<MI, MO, TI, TO>(n: usize) -> Fallible<Transformation<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, MI, MO>>
    where MI: DatasetMetric,
          MO: SensitivityMetric,
          MO::Distance: DistanceConstant,
          TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign,
          (MI, MO): CountByConstant<MI, MO> {
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
        MI::default(),
        MO::default(),
        StabilityRelation::new_from_constant(<(MI, MO)>::get_stability_constant()?)))
}


#[cfg(test)]
mod tests {
    use crate::dist::SymmetricDistance;
    use crate::trans::count::make_count_by_categories;

    use super::*;

    #[test]
    fn test_make_count_l1() {
        let transformation = make_count::<SymmetricDistance, i64, u32>().unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_l2() {
        let transformation = make_count::<SymmetricDistance, u32, i32>().unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_distinct() {
        let transformation = make_count_distinct::<SymmetricDistance, L1Distance<i32>, _>().unwrap_test();
        let arg = vec![1, 1, 3, 4, 4];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 3;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_by_categories() {
        let transformation = make_count_by_categories::<SymmetricDistance, L2Distance<f64>, i64, i8>(
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
        let transformation = make_count_by::<SymmetricDistance, L2Distance<f64>, bool, i8>(arg.len())?;
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