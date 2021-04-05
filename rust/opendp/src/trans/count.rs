use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{AddAssign, Div, Mul};

use num::{Integer, One, Zero, NumCast};
use num::traits::FloatConst;

use crate::core::{DatasetMetric, Function, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use crate::dom::{AllDomain, MapDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::DistanceCast;
use crate::trans::{MakeTransformation0, MakeTransformation1};

pub struct Count<MI, MO, T> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    data: PhantomData<T>,
}

impl<MI, MO, T> MakeTransformation0<VectorDomain<AllDomain<T>>, AllDomain<u32>, MI, MO> for Count<MI, MO, T>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric<Distance=u32> {
    fn make0() -> Fallible<Transformation<VectorDomain<AllDomain<T>>, AllDomain<u32>, MI, MO>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            AllDomain::new(),
            // min(arg.len(), u32::MAX)
            Function::new(move |arg: &Vec<T>| u32::try_from(arg.len()).unwrap_or(u32::MAX)),
            MI::new(),
            MO::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}

// count with unknown n, known categories
pub struct CountByCategories<MI, MO, TI, TO> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    input_data: PhantomData<TI>,
    output_data: PhantomData<TO>,
}

pub trait CountByCategoriesConstant<MI: DatasetMetric, MO: SensitivityMetric> {
    fn get_stability_constant() -> Fallible<MO::Distance>;
}

impl<TI, TO, QO: NumCast> CountByCategoriesConstant<HammingDistance, L1Sensitivity<QO>> for CountByCategories<HammingDistance, L1Sensitivity<QO>, TI, TO> {
    fn get_stability_constant() -> Fallible<QO> {
        QO::from(2.).ok_or_else(|| err!(FailedCast))
    }
}

impl<TI, TO, QO: FloatConst> CountByCategoriesConstant<HammingDistance, L2Sensitivity<QO>> for CountByCategories<HammingDistance, L2Sensitivity<QO>, TI, TO> {
    fn get_stability_constant() -> Fallible<QO> {
        Ok(QO::SQRT_2())
    }
}

impl<MO: SensitivityMetric<Distance=QO>, TI, TO, QO: One> CountByCategoriesConstant<SymmetricDistance, MO> for CountByCategories<SymmetricDistance, MO, TI, TO> {
    fn get_stability_constant() -> Fallible<QO> {
        Ok(QO::one())
    }
}

impl<MI, MO, TI, TO, QO> MakeTransformation1<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, MI, MO, Vec<TI>> for CountByCategories<MI, MO, TI, TO>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric<Distance=QO>,
          TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign,
          QO: 'static + Clone + DistanceCast + Mul<Output=QO> + Div<Output=QO> + PartialOrd,
          Self: CountByCategoriesConstant<MI, MO> {
    fn make1(categories: Vec<TI>) -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, MI, MO>> {
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

                categories.iter().map(|cat| counts.remove(cat))
                    .chain(vec![Some(null_count)])
                    .collect::<Option<_>>().unwrap_assert()
            }),
            MI::new(),
            MO::new(),
            StabilityRelation::new_from_constant(Self::get_stability_constant()?)))
    }
}

// count with known n, unknown categories
pub struct CountBy<MI, MO, TI, TO> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    input_data: PhantomData<TI>,
    output_data: PhantomData<TO>,
}

// this entire trait is duplicated code (only changed the struct it is impl'ed for)
pub trait CountByConstant<MI: DatasetMetric, MO: SensitivityMetric> {
    fn get_stability_constant() -> Fallible<MO::Distance>;
}

impl<TI, TO, QO: NumCast> CountByConstant<HammingDistance, L1Sensitivity<QO>> for CountBy<HammingDistance, L1Sensitivity<QO>, TI, TO> {
    fn get_stability_constant() -> Fallible<QO> {
        QO::from(2.).ok_or_else(|| err!(FailedCast))
    }
}

impl<TI, TO, QO: FloatConst> CountByConstant<HammingDistance, L2Sensitivity<QO>> for CountBy<HammingDistance, L2Sensitivity<QO>, TI, TO> {
    fn get_stability_constant() -> Fallible<QO> {
        Ok(QO::SQRT_2())
    }
}

impl<MO: SensitivityMetric<Distance=QO>, TI, TO, QO: One> CountByConstant<SymmetricDistance, MO> for CountBy<SymmetricDistance, MO, TI, TO> {
    fn get_stability_constant() -> Fallible<QO> {
        Ok(QO::one())
    }
}

impl<MI, MO, TI, TO, QO> MakeTransformation1<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, MI, MO, usize> for CountBy<MI, MO, TI, TO>
    where MI: DatasetMetric<Distance=u32>,
          MO: SensitivityMetric<Distance=QO>,
          TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign,
          QO: 'static + Clone + DistanceCast + Mul<Output=QO> + Div<Output=QO> + PartialOrd,
          Self: CountByConstant<MI, MO> {
    fn make1(n: usize) -> Fallible<Transformation<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, MI, MO>> {
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
            MI::new(),
            MO::new(),
            StabilityRelation::new_from_constant(Self::get_stability_constant()?)))
    }
}


#[cfg(test)]
mod tests {
    use crate::dist::{L1Sensitivity, SymmetricDistance};

    use super::*;

    #[test]
    fn test_make_count_l1() {
        let transformation = Count::<SymmetricDistance, L1Sensitivity<_>, i32>::make().unwrap_assert();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_assert();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_l2() {
        let transformation = Count::<SymmetricDistance, L2Sensitivity<_>, i32>::make().unwrap_assert();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_assert();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_by_categories() {
        let transformation = CountByCategories::<SymmetricDistance, L2Sensitivity<f64>, i64, i8>::make(
            vec![2, 1, 3]
        ).unwrap();
        let arg = vec![1, 2, 3, 4, 5, 1, 1, 1, 2];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = vec![2, 4, 1, 2];
        assert_eq!(ret, expected);

        assert!(!transformation.stability_relation.eval(&5, &4.999).unwrap());
        assert!(transformation.stability_relation.eval(&5, &5.0).unwrap());
    }

    #[test]
    fn test_make_count_by() {
        let arg = vec![true, true, true, false, true, false, false, false, true, true];
        let transformation = CountBy::<SymmetricDistance, L2Sensitivity<f64>, bool, i8>::make(arg.len()).unwrap();
        let ret = transformation.function.eval(&arg).unwrap();
        let mut expected = HashMap::new();
        expected.insert(true, 6);
        expected.insert(false, 4);
        assert_eq!(ret, expected);
        assert!(!transformation.stability_relation.eval(&5, &4.999).unwrap());
        assert!(transformation.stability_relation.eval(&5, &5.0).unwrap());
    }
}