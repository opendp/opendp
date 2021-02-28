use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::AddAssign;

use num::{Integer, One, Zero};

use crate::core::Measurement;
use crate::dist::{L1Sensitivity, L2Sensitivity, SmoothedMaxDivergence};
use crate::dom::{AllDomain, HashMapDomain, SizedDomain};
use crate::meas::{MakeMeasurement3, sample_gaussian, sample_laplace};

fn privacy_relation(d_in: f64, (eps, del): (f64, f64), n: usize, sigma: f64, threshold: f64) -> bool {
    let n = n as f64;
    if eps >= n.ln() || del >= 1. / n {
        return false
    }
    // check that sigma is large enough
    if sigma < d_in / (eps * n) {
        return false
    }
    // check that threshold is large enough
    if threshold < (2. / del).ln() * sigma + 1. / n {
        return false
    }
    return true
}

fn stability_mechanism<TI, TO, F: Fn() -> Result<f64, &'static str>>(
    counts: &HashMap<TI, TO>,
    get_noise: F,
    threshold: f64,
) -> Result<HashMap<TI, f64>, &'static str>
    where TI: Eq + Hash + Clone,
          TO: Clone,
          f64: From<TO> {
    Ok(counts.into_iter()
        .map(|(k, q_y)| Ok((k, get_noise().map(|noise| f64::from(q_y.clone()) + noise)?)))
        .collect::<Result<Vec<(&TI, f64)>, _>>()?.into_iter()
        .filter(|(_, a_y)| a_y >= &threshold)
        .map(|(k, v)| (k.clone(), v))
        .collect())
}

pub struct StabilityMechanism<MI, TIK, TIC> {
    input_metric: PhantomData<MI>,
    data_key: PhantomData<TIK>,
    data_count: PhantomData<TIC>,
}

type CountDomain<TIK, TIC> = SizedDomain<HashMapDomain<AllDomain<TIK>, AllDomain<TIC>>>;

// L1
impl<TIK, TIC> MakeMeasurement3<CountDomain<TIK, TIC>, CountDomain<TIK, f64>, L1Sensitivity<f64>, SmoothedMaxDivergence, usize, f64, f64> for StabilityMechanism<L1Sensitivity<f64>, TIK, TIC>
    where TIK: 'static + Eq + Hash + Clone,
          TIC: Integer + Zero + One + AddAssign + Clone,
          f64: From<TIC> {
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
impl<TIK, TIC> MakeMeasurement3<CountDomain<TIK, TIC>, CountDomain<TIK, f64>, L2Sensitivity<f64>, SmoothedMaxDivergence, usize, f64, f64> for StabilityMechanism<L2Sensitivity<f64>, TIK, TIC>
    where TIK: 'static + Eq + Hash + Clone,
          TIC: Integer + Zero + One + AddAssign + Clone,
          f64: From<TIC> {
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