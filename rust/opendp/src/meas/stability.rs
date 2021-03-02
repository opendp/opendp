use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::AddAssign;

use num::{Integer, One, Zero};

use crate::core::{Measurement, ChainMT};
use crate::dist::{L1Sensitivity, L2Sensitivity, SmoothedMaxDivergence, HammingDistance};
use crate::dom::{AllDomain, HashMapDomain, SizedDomain, VectorDomain};
use crate::trans::count::CountBy;
use crate::meas::{MakeMeasurement3, sample_gaussian, sample_laplace};
use std::convert::TryFrom;
use std::fmt::Debug;
use crate::trans::MakeTransformation1;

fn privacy_relation(d_in: f64, (eps, del): (f64, f64), n: usize, sigma: f64, threshold: f64) -> bool {
    let n = n as f64;
    let ideal_sigma = d_in / (eps * n);
    let ideal_threshold = (2. / del).ln() * ideal_sigma + 1. / n;

    if eps <= 0. || eps >= n.ln() {
        println!("failed:   eps >= n.ln()");
        return false
    }
    if del <= 0. || del >= 1. / n {
        println!("failed:   del >= 1. / n");
        return false
    }
    // check that sigma is large enough
    if sigma < ideal_sigma {
        println!("failed:   sigma < d_in / (eps * n)");
        return false
    }
    // check that threshold is large enough
    if threshold < ideal_threshold {
        println!("failed:   threshold < (2. / del).ln() * sigma + 1. / n");
        return false
    }
    return true
}

fn stability_mechanism<TIK, TIC, F: Fn() -> Result<f64, &'static str>>(
    counts: &HashMap<TIK, TIC>,
    get_noise: F,
    threshold: f64,
) -> Result<HashMap<TIK, f64>, &'static str>
    where TIK: Eq + Hash + Clone,
          TIC: Clone,
          f64: TryFrom<TIC>,
          <f64 as TryFrom<TIC>>::Error: Debug {
    Ok(counts.into_iter()
        .map(|(k, q_y)| Ok((k, get_noise().map(|noise| f64::try_from(q_y.clone()).unwrap() + noise)?)))
        .collect::<Result<Vec<(&TIK, f64)>, _>>()?.into_iter()
        .filter(|(_, a_y)| a_y >= &threshold)
        .map(|(k, v)| (k.clone(), v))
        .collect())
}

pub struct BaseStability<MI, TIK, TIC> {
    input_metric: PhantomData<MI>,
    data_key: PhantomData<TIK>,
    data_count: PhantomData<TIC>,
}

type CountDomain<TIK, TIC> = SizedDomain<HashMapDomain<AllDomain<TIK>, AllDomain<TIC>>>;

// L1
impl<TIK, TIC> MakeMeasurement3<CountDomain<TIK, TIC>, CountDomain<TIK, f64>, L1Sensitivity<f64>, SmoothedMaxDivergence, usize, f64, f64> for BaseStability<L1Sensitivity<f64>, TIK, TIC>
    where TIK: 'static + Eq + Hash + Clone,
          TIC: Integer + Zero + One + AddAssign + Clone,
          f64: TryFrom<TIC>,
          <f64 as TryFrom<TIC>>::Error: Debug {
    fn make3(n: usize, sigma: f64, threshold: f64) -> Measurement<CountDomain<TIK, TIC>, CountDomain<TIK, f64>, L1Sensitivity<f64>, SmoothedMaxDivergence> {
        Measurement::new(
            SizedDomain::new(HashMapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            SizedDomain::new(HashMapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            move |data: &HashMap<TIK, TIC>|
                stability_mechanism(data, || sample_laplace(sigma), threshold).unwrap(),
            L1Sensitivity::new(),
            SmoothedMaxDivergence::new(),
            move |&d_in: &f64, &d_out: &(f64, f64)|
                privacy_relation(d_in, d_out, n, sigma, threshold))
    }
}

// L2
impl<TIK, TIC> MakeMeasurement3<CountDomain<TIK, TIC>, CountDomain<TIK, f64>, L2Sensitivity<f64>, SmoothedMaxDivergence, usize, f64, f64> for BaseStability<L2Sensitivity<f64>, TIK, TIC>
    where TIK: 'static + Eq + Hash + Clone,
          TIC: Integer + Zero + One + AddAssign + Clone,
          f64: TryFrom<TIC>,
          <f64 as TryFrom<TIC>>::Error: Debug {
    fn make3(n: usize, sigma: f64, threshold: f64) -> Measurement<CountDomain<TIK, TIC>, CountDomain<TIK, f64>, L2Sensitivity<f64>, SmoothedMaxDivergence> {
        Measurement::new(
            SizedDomain::new(HashMapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            SizedDomain::new(HashMapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
            move |data: &HashMap<TIK, TIC>|
                stability_mechanism(data, || Ok(sample_gaussian(0., sigma, false)), threshold).unwrap(),
            L2Sensitivity::new(),
            SmoothedMaxDivergence::new(),
            move |&d_in: &f64, &d_out: &(f64, f64)|
                privacy_relation(d_in, d_out, n, sigma, threshold))
    }
}


// fuse count and stability with a custom hint
pub struct StabilityMechanism<MI, TIK, TIC> {
    input_metric: PhantomData<MI>,
    data_key: PhantomData<TIK>,
    data_count: PhantomData<TIC>,
}

impl<TIK, TIC> MakeMeasurement3<SizedDomain<VectorDomain<AllDomain<TIK>>>, CountDomain<TIK, f64>, HammingDistance, SmoothedMaxDivergence, usize, f64, f64> for StabilityMechanism<HammingDistance, TIK, TIC>
    where TIK: 'static + Eq + Hash + Clone,
          TIC: 'static + Integer + Zero + One + AddAssign + Clone,
          f64: TryFrom<TIC>,
          <f64 as TryFrom<TIC>>::Error: Debug {
    fn make3(n: usize, sigma: f64, threshold: f64) -> Measurement<SizedDomain<VectorDomain<AllDomain<TIK>>>, CountDomain<TIK, f64>, HammingDistance, SmoothedMaxDivergence> {
        ChainMT::make3(
            &BaseStability::make(n, sigma, threshold),
            &CountBy::<HammingDistance, L1Sensitivity<f64>, TIK, TIC>::make(n),
            |&d_in: &u32, _d_out: &(f64, f64)| {
                d_in as f64 * 2.
            })
    }
}