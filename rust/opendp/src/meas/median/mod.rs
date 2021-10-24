use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ops::Sub;

use num::Float;

use crate::core::{Function, Measurement, Metric, PrivacyRelation};
use crate::dist::{IntDistance, L1Distance, L2Distance, SmoothedMaxDivergence, SymmetricDistance};
use crate::dom::{AllDomain, BoundedDomain, VectorDomain};
use crate::error::{ExplainUnwrap, Fallible};
use crate::samplers::{SampleGaussian, SampleLaplace, SampleTwoSidedGeometric};
use crate::traits::{CheckNull, ExactIntCast, InfCast, TotalOrd};

fn partial_max<T: PartialOrd>(x: &T, y: &T) -> Ordering {
    x.partial_cmp(y).unwrap_or(Ordering::Equal)
}

fn fallible_max<T: PartialOrd>(x: &Fallible<T>, y: &Fallible<T>) -> Ordering {
    let x = if let Ok(x) = x {x} else {return Ordering::Greater};
    let y = if let Ok(y) = y {y} else {return Ordering::Less};
    partial_max(x, y)
}

struct Median<T>(PhantomData<T>);
struct Minimum<T>(PhantomData<T>);
struct Maximum<T>(PhantomData<T>);

pub trait SmoothSensitivityQuery {
    type Atom;
    fn compute(sorted_data: &Vec<Self::Atom>) -> Self::Atom;
    fn a_k(sorted_data: &Vec<Self::Atom>, bounds: &(Self::Atom, Self::Atom), k: usize) -> Self::Atom;
}

impl<T: Clone + Sub<Output=T> + PartialOrd> SmoothSensitivityQuery for Median<T> {
    type Atom = T;
    fn compute(sorted_data: &Vec<T>) -> T {
        sorted_data[(sorted_data.len() + 1) / 2].clone()
    }

    fn a_k(sorted_data: &Vec<T>, bounds: &(T, T), k: usize) -> T {
        let m = (sorted_data.len() + 1) / 2;
        // function to compute the local sensitivity of the median on the given data
        // at median_index + t, when up to k entries may be changed
        (0..=k).map(move |t| {
            // return lower bound if index is negative
            let l = if m + t < k + 1 { &bounds.0 } else { &sorted_data[m + t - k - 1] }.clone();
            // return upper bound if index is past array length
            let u = if m + t >= sorted_data.len() { &bounds.1 } else { &sorted_data[m + t] }.clone();
            u - l
        })
            .max_by(partial_max)
            .unwrap_assert("dataset consists of at least one element")
    }
}

impl<T: Clone + PartialOrd + Sub<Output=T>> SmoothSensitivityQuery for Minimum<T> {
    type Atom = T;
    fn compute(sorted_data: &Vec<T>) -> T {
        sorted_data[0].clone()
    }

    fn a_k(sorted_data: &Vec<T>, bounds: &(T, T), k: usize) -> T {
        let n = sorted_data.len();
        let t1 = if k == n { bounds.1.clone() } else { sorted_data[k].clone() - bounds.0.clone() };
        let t2 = if k + 1 == n { &bounds.1 } else { &sorted_data[k + 1] }.clone()
            - sorted_data[0].clone();

        if t1 > t2 { t1 } else { t2 }
    }
}

impl<T: Clone + PartialOrd + Sub<Output=T>> SmoothSensitivityQuery for Maximum<T> {
    type Atom = T;
    fn compute(sorted_data: &Vec<T>) -> T {
        sorted_data[sorted_data.len() - 1].clone()
    }

    fn a_k(sorted_data: &Vec<T>, bounds: &(T, T), k: usize) -> T {
        let n = sorted_data.len();
        let t1 = if k == n { bounds.0.clone() } else { bounds.1.clone() - sorted_data[n - k - 1].clone() };
        let t2 = if k + 1 == n { &bounds.0 } else { &sorted_data[n - k - 2] }.clone()
            - sorted_data[n - 1].clone();

        if t1 > t2 { t1 } else { t2 }
    }
}

pub trait SmoothSensitivityNoise: Metric {
    type Unit: ExactIntCast<usize> + Float;
    fn alpha(budget: (Self::Unit, Self::Unit)) -> Fallible<Self::Unit>;
    fn beta(budget: (Self::Unit, Self::Unit)) -> Fallible<Self::Unit> {
        let _1 = Self::Unit::exact_int_cast(1)?;
        let _2 = Self::Unit::exact_int_cast(2)?;
        let _4 = Self::Unit::exact_int_cast(4)?;
        // the same beta is shared between all l1 and l2 noise distributions
        let (epsilon, delta) = budget;
        Ok(epsilon / (_4 * (_1 + (_2 / delta).ln())))
    }
    fn sample(shift: Self::Distance, scale: Self::Unit) -> Fallible<Self::Distance>;
}
impl SmoothSensitivityNoise for L1Distance<f64> {
    type Unit = f64;
    fn alpha(budget: (Self::Unit, Self::Unit)) -> Fallible<Self::Unit> {
        Ok(budget.0 / 2.)
    }
    fn sample(shift: Self::Distance, scale: Self::Unit) -> Fallible<Self::Distance> {
        Self::Distance::sample_laplace(shift, scale, false)
    }
}
impl SmoothSensitivityNoise for L1Distance<i32> {
    type Unit = f64;
    fn alpha(budget: (Self::Unit, Self::Unit)) -> Fallible<Self::Unit> {
        Ok(budget.0 / 2.)
    }
    fn sample(shift: Self::Distance, scale: Self::Unit) -> Fallible<Self::Distance> {
        Self::Distance::sample_two_sided_geometric(shift, scale, None)
    }
}

impl SmoothSensitivityNoise for L2Distance<f64> {
    type Unit = f64;
    fn alpha(budget: (Self::Unit, Self::Unit)) -> Fallible<Self::Unit> {
        Ok(budget.0 / (5. * (2. * (2. / budget.1).ln()).sqrt()))
    }
    fn sample(shift: Self::Distance, scale: Self::Unit) -> Fallible<Self::Distance> {
        Self::Distance::sample_gaussian(shift, scale, false)
    }
}

fn compute_smooth_sensitivity<Query, Noise>(
    sorted_data: Vec<Query::Atom>,
    bounds: (Query::Atom, Query::Atom),
    budget: (Noise::Unit, Noise::Unit),
) -> Fallible<Noise::Unit>
    where Query: SmoothSensitivityQuery,
          Noise: SmoothSensitivityNoise,
          Query::Atom: Clone + Sub<Output=Query::Atom>,
          Noise::Unit: Float + InfCast<Query::Atom> + ExactIntCast<usize> {
    let beta = Noise::beta(budget)?;

    (0..sorted_data.len())
        .map(move |k|
            Ok(Noise::Unit::inf_cast(Query::a_k(&sorted_data, &bounds, k))? * (-Noise::Unit::exact_int_cast(k)? * beta).exp()))
        // upgrade NaN to an Err
        .map(|v|
            v.and_then(|v| if v.is_nan() {fallible!(FailedFunction)} else {Ok(v)}))
        // get max, or if Err, return Inf
        .max_by(fallible_max).unwrap_or(Ok(Noise::Unit::infinity()))
}

pub fn make_ss_estimate<Query, Noise>(
    bounds: (Query::Atom, Query::Atom),
    influence: IntDistance,
    budget: (Noise::Unit, Noise::Unit),
) -> Fallible<Measurement<VectorDomain<BoundedDomain<Query::Atom>>, AllDomain<Query::Atom>, SymmetricDistance, SmoothedMaxDivergence<Noise::Unit>>>
    where Query: SmoothSensitivityQuery,
          Noise: SmoothSensitivityNoise<Distance=Query::Atom>,
          Query::Atom: 'static + CheckNull + TotalOrd + Clone + Sub<Output=Query::Atom>,
          Noise::Unit: 'static + Float + InfCast<Query::Atom> + ExactIntCast<usize> + ExactIntCast<IntDistance> {

    if influence < 1 { return fallible!(MakeMeasurement, "influence must be positive") }
    let (epsilon, delta) = budget;
    if epsilon.is_sign_negative() { return fallible!(MakeMeasurement, "epsilon must be non-negative") }
    if delta.is_sign_negative() { return fallible!(MakeMeasurement, "delta must be non-negative") }

    let epsilon_prime = epsilon.clone() / Noise::Unit::exact_int_cast(influence)?;
    let delta_prime = epsilon / Noise::Unit::exact_int_cast(influence)?;
    let budget_prime = (epsilon_prime.clone(), delta_prime);

    Ok(Measurement::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds.clone())?),
        AllDomain::new(),
        Function::new_fallible(move |data: &Vec<Query::Atom>| {
            let mut sorted_data = data.clone();
            sorted_data.sort_by(partial_max);

            let value = Query::compute(&sorted_data);

            let sens: Noise::Unit = compute_smooth_sensitivity::<Query, Noise>(sorted_data, bounds.clone(), budget_prime)?;

            Noise::sample(value, sens / Noise::alpha(budget_prime)?)
        }),
        SymmetricDistance::default(),
        SmoothedMaxDivergence::default(),
        PrivacyRelation::new(
            move |&d_in, &d_out: &(Noise::Unit, Noise::Unit)| d_in <= influence && budget <= d_out),
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_function() -> Fallible<()> {
        let meas = make_ss_estimate::<Median<f64>, L1Distance<f64>>(
            (-1., 10.), 1, (1., 1e-6))?;
        let res = meas.invoke(&vec![-0.05, 2., 3., 0., 4., 1.2, 0.7, 1.3])?;
        println!("fp median {}", res);

        let meas = make_ss_estimate::<Median<i32>, L1Distance<i32>>(
            (-1, 10), 1, (1., 1e-6))?;
        let res = meas.invoke(&vec![1,2,3,4,4,4,4,5,6,7,8])?;
        println!("int median {}", res);

        let meas = make_ss_estimate::<Minimum<i32>, L1Distance<i32>>(
            (-1, 10), 1, (1., 1e-6))?;
        let res = meas.invoke(&vec![1,1,2,3,4,4,4,4,5,6,7,8])?;
        println!("int min {}", res);
        Ok(())
    }
}
