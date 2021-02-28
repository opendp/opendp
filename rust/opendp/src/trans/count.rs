use core::marker::PhantomData;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::f64::consts::SQRT_2;
use std::hash::Hash;
use std::ops::AddAssign;

use num::{Integer, NumCast, One, Zero};

use crate::core::{DatasetMetric, Metric, SensitivityMetric, Transformation};
use crate::dist::{HammingDistance, L1Sensitivity, SymmetricDistance, L2Sensitivity};
use crate::dom::{AllDomain, SizedDomain, VectorDomain, HashMapDomain};
use crate::trans::{MakeTransformation0, MakeTransformation1};

pub struct Count<MI, MO, T> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    data: PhantomData<T>,
}

impl<MI, MO, T> MakeTransformation0<VectorDomain<AllDomain<T>>, AllDomain<u32>, MI, MO> for Count<MI, MO, T>
    where MI: Metric<Distance=u32> + DatasetMetric,
          MO: Metric<Distance=u32> + SensitivityMetric {
    fn make0() -> Transformation<VectorDomain<AllDomain<T>>, AllDomain<u32>, MI, MO> {
        Transformation::new(
            VectorDomain::new_all(),
            AllDomain::new(),
            move |arg: &Vec<T>| arg.len() as u32,
            MI::new(),
            MO::new(),
            |d_in: &u32, d_out: &u32| *d_out >= *d_in)
    }
}

pub struct CountBy<MI, MO, T> {
    input_metric: PhantomData<MI>,
    output_metric: PhantomData<MO>,
    data: PhantomData<T>,
}

fn count_by_categories<T, QO>(data: &Vec<T>, categories: &Vec<T>) -> Vec<QO>
    where T: Eq + Hash,
          QO: Integer + Zero + One + AddAssign<QO> {
    let mut counts = categories.iter()
        .map(|cat| (cat, QO::zero())).collect::<HashMap<&T, QO>>();
    let mut null_count = QO::zero();

    data.into_iter().for_each(|v|
        *match counts.entry(v) {
            Entry::Occupied(v) => v.into_mut(),
            Entry::Vacant(_v) => &mut null_count
        } += QO::one());

    categories.iter().map(|cat| counts.remove(cat))
        .chain(vec![Some(null_count)])
        .collect::<Option<_>>().unwrap()
}


impl<TI, TO> MakeTransformation1<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, HammingDistance, L1Sensitivity<u32>, Vec<TI>> for CountBy<HammingDistance, L1Sensitivity<u32>, TI>
    where TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign {
    fn make1(categories: Vec<TI>) -> Transformation<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, HammingDistance, L1Sensitivity<u32>> {
        Transformation::new(
            VectorDomain::new_all(),
            SizedDomain::new(VectorDomain::new_all(), categories.len() + 1),
            move |data: &Vec<TI>| count_by_categories(data, &categories),
            HammingDistance::new(),
            L1Sensitivity::new(),
            |d_in: &u32, d_out: &u32| *d_out >= d_in * 2)
    }
}

impl<TI, TO> MakeTransformation1<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, HammingDistance, L1Sensitivity<f64>, Vec<TI>> for CountBy<HammingDistance, L1Sensitivity<f64>, TI>
    where TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign {
    fn make1(categories: Vec<TI>) -> Transformation<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, HammingDistance, L1Sensitivity<f64>> {
        Transformation::new(
            VectorDomain::new_all(),
            SizedDomain::new(VectorDomain::new_all(), categories.len() + 1),
            move |data: &Vec<TI>| count_by_categories(data, &categories),
            HammingDistance::new(),
            L1Sensitivity::new(),
            |d_in: &u32, d_out: &f64| *d_out >= *d_in as f64 * SQRT_2)
    }
}

impl<TI, TO, MO, QO> MakeTransformation1<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, SymmetricDistance, MO, Vec<TI>> for CountBy<SymmetricDistance, MO, TI>
    where TI: 'static + Eq + Hash,
          TO: Integer + Zero + One + AddAssign,
          MO: Metric<Distance=QO> + SensitivityMetric,
          QO: NumCast + Clone {
    fn make1(categories: Vec<TI>) -> Transformation<VectorDomain<AllDomain<TI>>, SizedDomain<VectorDomain<AllDomain<TO>>>, SymmetricDistance, MO> {
        Transformation::new(
            VectorDomain::new_all(),
            SizedDomain::new(VectorDomain::new_all(), categories.len() + 1),
            move |data: &Vec<TI>| count_by_categories(data, &categories),
            SymmetricDistance::new(),
            MO::new(),
            |d_in: &u32, d_out: &QO| <u32 as NumCast>::from(d_out.clone()).unwrap() >= *d_in)
    }
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
impl<TI, TO> MakeTransformation1<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<HashMapDomain<AllDomain<TI>, AllDomain<TO>>>, HammingDistance, L1Sensitivity<f64>, usize> for CountBy<HammingDistance, L1Sensitivity<f64>, TI>
    where TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign {
    fn make1(n: usize) -> Transformation<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<HashMapDomain<AllDomain<TI>, AllDomain<TO>>>, HammingDistance, L1Sensitivity<f64>> {
        Transformation::new(
            SizedDomain::new(VectorDomain::new_all(), n),
            SizedDomain::new(HashMapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            move |data: &Vec<TI>| count_by(data),
            HammingDistance::new(),
            L1Sensitivity::new(),
            move |&d_in: &u32, &d_out: &f64|
                d_out >= d_in as f64 * 2.)
    }
}

// Hamming / L2
impl<TI, TO> MakeTransformation1<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<HashMapDomain<AllDomain<TI>, AllDomain<TO>>>, HammingDistance, L2Sensitivity<f64>, usize> for CountBy<HammingDistance, L2Sensitivity<f64>, TI>
    where TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign {
    fn make1(n: usize) -> Transformation<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<HashMapDomain<AllDomain<TI>, AllDomain<TO>>>, HammingDistance, L2Sensitivity<f64>> {
        Transformation::new(
            SizedDomain::new(VectorDomain::new_all(), n),
            SizedDomain::new(HashMapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            move |data: &Vec<TI>| count_by(data),
            HammingDistance::new(),
            L2Sensitivity::new(),
            move |&d_in: &u32, &d_out: &f64|
                d_out >= d_in as f64 * SQRT_2)
    }
}

// Symmetric / LP
impl<TI, TO, MO> MakeTransformation1<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<HashMapDomain<AllDomain<TI>, AllDomain<TO>>>, SymmetricDistance, MO, usize> for CountBy<SymmetricDistance, MO, TI>
    where TI: 'static + Eq + Hash + Clone,
          TO: Integer + Zero + One + AddAssign,
          MO: Metric<Distance=f64> + SensitivityMetric {
    fn make1(n: usize) -> Transformation<SizedDomain<VectorDomain<AllDomain<TI>>>, SizedDomain<HashMapDomain<AllDomain<TI>, AllDomain<TO>>>, SymmetricDistance, MO> {
        Transformation::new(
            SizedDomain::new(VectorDomain::new_all(), n),
            SizedDomain::new(HashMapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            move |data: &Vec<TI>| count_by(data),
            SymmetricDistance::new(),
            MO::new(),
            move |&d_in: &u32, &d_out: &f64|
                d_out >= d_in as f64)
    }
}


#[cfg(test)]
mod tests {
    use crate::dist::{L1Sensitivity, L2Sensitivity, SymmetricDistance};

    use super::*;

    #[test]
    fn test_make_count_l1() {
        let transformation = Count::<SymmetricDistance, L1Sensitivity<u32>, u32>::make();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg);
        let expected = 5;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_count_l2() {
        let transformation = Count::<SymmetricDistance, L2Sensitivity<u32>, u32>::make();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg);
        let expected = 5;
        assert_eq!(ret, expected);
    }
}
