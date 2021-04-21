use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::convert::TryFrom;
use std::hash::Hash;
use std::ops::{AddAssign, Div, Mul};

use num::{Integer, NumCast, One, Zero};
use num::traits::FloatConst;

use crate::core::{DatasetMetric, Function, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use crate::dom::{AllDomain, MapDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::DistanceCast;


pub fn make_count<MI, MO, T>() -> Fallible<Transformation<VectorDomain<AllDomain<T>>, AllDomain<u32>, MI, MO>>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric<Distance=u32> {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        // min(arg.len(), u32::MAX)
        Function::new(move |arg: &Vec<T>| u32::try_from(arg.len()).unwrap_or(u32::MAX)),
        MI::default(),
        MO::default(),
        StabilityRelation::new_from_constant(1_u32)))
}

pub trait CountByCategoriesConstant<MI: DatasetMetric, MO: SensitivityMetric> {
    fn get_stability_constant() -> Fallible<MO::Distance>;
}

impl<QO: NumCast> CountByCategoriesConstant<HammingDistance, L1Sensitivity<QO>> for (HammingDistance, L1Sensitivity<QO>) {
    fn get_stability_constant() -> Fallible<QO> {
        c!(2.; QO)
    }
}

impl<QO: FloatConst> CountByCategoriesConstant<HammingDistance, L2Sensitivity<QO>> for (HammingDistance, L2Sensitivity<QO>) {
    fn get_stability_constant() -> Fallible<QO> {
        Ok(QO::SQRT_2())
    }
}

impl<MO: SensitivityMetric> CountByCategoriesConstant<SymmetricDistance, MO> for (SymmetricDistance, MO)
    where MO::Distance: One {
    fn get_stability_constant() -> Fallible<MO::Distance> {
        Ok(MO::Distance::one())
    }
}

// count with unknown n, known categories
pub fn make_count_by_categories<MI, MO, TI, TO>(categories: Vec<TI>) -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, MI, MO>>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric,
          TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign,
          MO::Distance: 'static + Clone + DistanceCast + Mul<Output=MO::Distance> + Div<Output=MO::Distance> + PartialOrd,
          (MI, MO): CountByCategoriesConstant<MI, MO> {
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

            data.into_iter().for_each(|v|
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


// this entire trait is duplicated code (only changed the struct it is impl'ed for)
pub trait CountByConstant<MI: DatasetMetric, MO: SensitivityMetric> {
    fn get_stability_constant() -> Fallible<MO::Distance>;
}

impl<QO: NumCast> CountByConstant<HammingDistance, L1Sensitivity<QO>> for (HammingDistance, L1Sensitivity<QO>) {
    fn get_stability_constant() -> Fallible<QO> {
        c!(2.; QO)
    }
}

impl<QO: FloatConst> CountByConstant<HammingDistance, L2Sensitivity<QO>> for (HammingDistance, L2Sensitivity<QO>) {
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

// count with known n, unknown categories
pub fn make_count_by<MI, MO, TI, TO>(n: usize) -> Fallible<Transformation<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, MI, MO>>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric,
          TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign,
          MO::Distance: 'static + Clone + DistanceCast + Mul<Output=MO::Distance> + Div<Output=MO::Distance> + PartialOrd,
          (MI, MO): CountByConstant<MI, MO> {
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new_all(), n),
        SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
        Function::new(move |data: &Vec<TI>| {
            let mut counts = HashMap::new();
            data.into_iter().for_each(|v|
                *counts.entry(v.clone()).or_insert(TO::zero()) += TO::one()
            );
            counts
        }),
        MI::default(),
        MO::default(),
        StabilityRelation::new_from_constant(<(MI, MO)>::get_stability_constant()?)))
}


#[cfg(test)]
mod tests {
    use crate::dist::{L1Sensitivity, SymmetricDistance};

    use super::*;
    use crate::trans::count::make_count_by_categories;

    #[test]
    fn test_make_count_l1() {
        let transformation = make_count::<SymmetricDistance, L1Sensitivity<_>, _>().unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_l2() {
        let transformation = make_count::<SymmetricDistance, L2Sensitivity<_>, i32>().unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_by_categories() {
        let transformation = make_count_by_categories::<SymmetricDistance, L2Sensitivity<f64>, i64, i8>(
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
        let transformation = make_count_by::<SymmetricDistance, L2Sensitivity<f64>, bool, i8>(arg.len())?;
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