use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{AddAssign, Div, Mul};

use num::{Integer, One, Zero};
use num::traits::FloatConst;

use crate::core::{DatasetMetric, Function, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use crate::dom::{AllDomain, MapDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
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


pub trait CountByStability<MI: DatasetMetric, MO: SensitivityMetric> {
    fn get_stability() -> MO::Distance;
}

impl<TI, TO, QI, QO: From<u32>> CountByStability<HammingDistance, L1Sensitivity<QO>> for CountByCategories<HammingDistance, L1Sensitivity<QO>, TI, TO, QI, QO> {
    fn get_stability() -> QO {
        <QO as From<u32>>::from(2_u32)
    }
}

impl<TI, TO, QI, QO: FloatConst> CountByStability<HammingDistance, L2Sensitivity<QO>> for CountByCategories<HammingDistance, L2Sensitivity<QO>, TI, TO, QI, QO> {
    fn get_stability() -> QO {
        QO::SQRT_2()
    }
}

impl<MO: SensitivityMetric<Distance=QO>, TI, TO, QI, QO: One> CountByStability<SymmetricDistance, MO> for CountByCategories<HammingDistance, MO, TI, TO, QI, QO> {
    fn get_stability() -> QO {
        QO::one()
    }
}

// count with unknown n, known categories
pub struct CountByCategories<MI, MO, TI, TO, QI, QO> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    input_data: PhantomData<TI>,
    output_data: PhantomData<TO>,
    input_distance: PhantomData<QI>,
    output_distance: PhantomData<QO>,
}

impl<MI, MO, TI, TO, QI, QO> MakeTransformation1<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, MI, MO, Vec<TI>> for CountByCategories<MI, MO, TI, TO, QI, QO>
    where MI: DatasetMetric<Distance=QI>,
          MO: SensitivityMetric<Distance=QO>,
          TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign,
          QI: DistanceCast,
          QO: 'static + Clone + DistanceCast + Mul<Output=QO> + Div<Output=QO> + PartialOrd,
          Self: CountByStability<MI, MO> {
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
                    // this is a "safe" unwrap
                    .collect::<Option<_>>().unwrap()
            }),
            MI::new(),
            MO::new(),
            StabilityRelation::new_from_constant(Self::get_stability())))
    }
}

// count with known n, unknown categories
pub struct CountBy<MI, MO, TI, TO, QI, QO> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    input_data: PhantomData<TI>,
    output_data: PhantomData<TO>,
    input_distance: PhantomData<QI>,
    output_distance: PhantomData<QO>,
}

impl<MI, MO, TI, TO, QI, QO> MakeTransformation1<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, MI, MO, usize> for CountBy<MI, MO, TI, TO, QI, QO>
    where MI: DatasetMetric<Distance=QI>,
          MO: SensitivityMetric<Distance=QO>,
          TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign,
          QI: DistanceCast,
          QO: 'static + Clone + DistanceCast + Mul<Output=QO> + Div<Output=QO> + PartialOrd,
          Self: CountByStability<MI, MO> {
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
            StabilityRelation::new_from_constant(Self::get_stability())))
    }
}


#[cfg(test)]
mod tests {
    use crate::dist::{L1Sensitivity, SymmetricDistance};

    use super::*;

    #[test]
    fn test_make_count_l1() {
        let transformation = Count::<SymmetricDistance, L1Sensitivity<_>, i32>::make().unwrap();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_l2() {
        let transformation = Count::<SymmetricDistance, L2Sensitivity<_>, i32>::make().unwrap();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = 5;
        assert_eq!(ret, expected);
    }
}