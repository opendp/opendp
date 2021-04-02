use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{AddAssign, Mul, Div};

use num::{Integer, One, Zero};

use crate::core::{DatasetMetric, Function, Metric, SensitivityMetric, StabilityRelation, Transformation};
use crate::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use crate::dom::{AllDomain, MapDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::trans::{MakeTransformation0, MakeTransformation1};
use crate::traits::{DistanceCast};
use num::traits::FloatConst;

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


pub struct CountByCategories<MI, MO, TI, TO, QO> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    input_data: PhantomData<TI>,
    output_data: PhantomData<TO>,
    output_distance: PhantomData<QO>
}

fn count_by_categories<TI, TO>(data: &Vec<TI>, categories: &Vec<TI>) -> Vec<TO>
    where TI: Eq + Hash,
          TO: Integer + Zero + One + AddAssign<TO> {
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
}


impl<TI, TO, QO> MakeTransformation1<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, HammingDistance, L1Sensitivity<QO>, Vec<TI>> for CountByCategories<HammingDistance, L1Sensitivity<QO>, TI, TO, QO>
    where TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign,
          QO: 'static + From<u32> + Clone + DistanceCast + Mul<Output=QO> + Div<Output=QO> + PartialOrd {
    fn make1(categories: Vec<TI>) -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, HammingDistance, L1Sensitivity<QO>>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            SizedDomain::new(VectorDomain::new_all(), categories.len() + 1),
            Function::new(move |data: &Vec<TI>| count_by_categories(data, &categories)),
            HammingDistance::new(),
            L1Sensitivity::new(),
            StabilityRelation::new_from_constant(<QO as From<u32>>::from(2_u32))))
    }
}

impl<TI, TO, QO> MakeTransformation1<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, HammingDistance, L2Sensitivity<QO>, Vec<TI>> for CountByCategories<HammingDistance, L2Sensitivity<QO>, TI, TO, QO>
    where TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign,
          QO: 'static + FloatConst + From<u32> + Clone + DistanceCast + Mul<Output=QO> + Div<Output=QO> + PartialOrd {
    fn make1(categories: Vec<TI>) -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, HammingDistance, L2Sensitivity<QO>>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            SizedDomain::new(VectorDomain::new_all(), categories.len() + 1),
            Function::new(move |data: &Vec<TI>| count_by_categories(data, &categories)),
            HammingDistance::new(),
            L2Sensitivity::new(),
            StabilityRelation::new_from_constant(QO::SQRT_2())))
    }
}

impl<TI, TO, MO, QO> MakeTransformation1<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, SymmetricDistance, MO, Vec<TI>> for CountByCategories<SymmetricDistance, MO, TI, TO, QO>
    where TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign,
          MO: SensitivityMetric<Distance=QO>,
          QO: 'static + Clone + One + Clone + DistanceCast + Mul<Output=QO> + Div<Output=QO> + PartialOrd {
    fn make1(categories: Vec<TI>) -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, SymmetricDistance, MO>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            SizedDomain::new(VectorDomain::new_all(), categories.len() + 1),
            Function::new(move |data: &Vec<TI>| count_by_categories(data, &categories)),
            SymmetricDistance::new(),
            MO::new(),
            StabilityRelation::new_from_constant(QO::one())))
    }
}


pub struct CountBy<MI, MO, TI, TO, QO> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    input_data: PhantomData<TI>,
    output_data: PhantomData<TO>,
    output_distance: PhantomData<QO>
}


// count with known n, unknown categories
fn count_by<TI, TO>(data: &Vec<TI>) -> HashMap<TI, TO>
    where TI: Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign {
    let mut counts = HashMap::new();

    data.into_iter().for_each(|v|
        *counts.entry(v.clone()).or_insert(TO::zero()) += TO::one()
    );

    counts
}

// Hamming / L1
impl<TI, TO, QO> MakeTransformation1<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, HammingDistance, L1Sensitivity<QO>, usize> for CountBy<HammingDistance, L1Sensitivity<QO>, TI, TO, QO>
    where TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign,
          QO: 'static + From<f64> + Clone + DistanceCast + Mul<Output=QO> + Div<Output=QO> + PartialOrd {
    fn make1(n: usize) -> Fallible<Transformation<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, HammingDistance, L1Sensitivity<QO>>> {
        Ok(Transformation::new(
            SizedDomain::new(VectorDomain::new_all(), n),
            SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            Function::new(move |data: &Vec<TI>| count_by(data)),
            HammingDistance::new(),
            L1Sensitivity::new(),
            StabilityRelation::new_from_constant(<QO as From<f64>>::from(2.))))
    }
}

// Hamming / L2
impl<TI, TO, QO> MakeTransformation1<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, HammingDistance, L2Sensitivity<QO>, usize> for CountBy<HammingDistance, L2Sensitivity<QO>, TI, TO, QO>
    where TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign,
          QO: 'static + FloatConst + Clone + One + Clone + DistanceCast + Mul<Output=QO> + Div<Output=QO> + PartialOrd {
    fn make1(n: usize) -> Fallible<Transformation<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, HammingDistance, L2Sensitivity<QO>>> {
        Ok(Transformation::new(
            SizedDomain::new(VectorDomain::new_all(), n),
            SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            Function::new(move |data: &Vec<TI>| count_by(data)),
            HammingDistance::new(),
            L2Sensitivity::new(),
            StabilityRelation::new_from_constant(QO::SQRT_2())))
    }
}

// Symmetric / LP
impl<TI, TO, MO, QO> MakeTransformation1<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, SymmetricDistance, MO, usize> for CountBy<SymmetricDistance, MO, TI, TO, QO>
    where TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign,
          MO: Metric<Distance=QO> + SensitivityMetric,
          QO: 'static + One + Clone + DistanceCast + Mul<Output=QO> + Div<Output=QO> + PartialOrd {
    fn make1(n: usize) -> Fallible<Transformation<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<MapDomain<AllDomain<TI>, AllDomain<TO>>>, SymmetricDistance, MO>> {
        Ok(Transformation::new(
            SizedDomain::new(VectorDomain::new_all(), n),
            SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            Function::new(move |data: &Vec<TI>| count_by(data)),
            SymmetricDistance::new(),
            MO::new(),
            StabilityRelation::new_from_constant(QO::one())))
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