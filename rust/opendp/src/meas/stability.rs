use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::core::Measurement;
use crate::dist::{HammingDistance, SmoothedMaxDivergence};
use crate::dom::{AllDomain, HashMapDomain, VectorDomain};
use crate::meas::{MakeMeasurement3, sample_laplace};

fn privacy_relation(d_in: u32, (eps, del): (f64, f64), n: usize, sigma: f64, threshold: f64) -> bool {
    let n = n as f64;
    if eps >= n.ln() || del >= 1. / n {
        return false
    }
    // check that sigma is large enough
    if sigma < d_in as f64 * 2. / (eps * n) {
        return false
    }
    // check that threshold is large enough
    if threshold < (2. / del).ln() * sigma + 1. / n {
        return false
    }
    return true
}

// This is a tool to find the threshold
struct TauThreshold<MI> {
    input_metric: PhantomData<MI>
}

impl MakeMeasurement3<(), (), HammingDistance, SmoothedMaxDivergence, usize, f64, f64> for TauThreshold<HammingDistance> {
    fn make3(n: usize, sigma: f64, threshold: f64) -> Measurement<(), (), HammingDistance, SmoothedMaxDivergence> {
        Measurement::new(
            (),
            (),
            |_data: &()| (),
            HammingDistance::new(),
            SmoothedMaxDivergence::new(),
            move |&d_in: &u32, &d_out: &(f64, f64)|
                privacy_relation(d_in, d_out, n, sigma, threshold))
    }
}


pub struct StabilityCountBy<MI, T> {
    input_metric: PhantomData<MI>,
    data: PhantomData<T>,
}


fn count_by<TI>(data: &Vec<TI>) -> HashMap<TI, u32>
    where TI: Eq + Hash + Clone {
    let mut counts = HashMap::new();

    data.into_iter().for_each(|v|
        *counts.entry(v.clone()).or_insert(0) += 1
    );

    counts
}

fn stability_mechanism<TI, TO>(mut counts: HashMap<TI, TO>, sigma: f64, threshold: f64) -> Result<HashMap<TI, f64>, &'static str>
    where TI: Eq + Hash,
          f64: From<TO> {
    Ok(counts.drain()
        .map(|(k, q_y)| Ok((k, sample_laplace(sigma).map(|noise| f64::from(q_y) + noise)?)))
        .collect::<Result<Vec<(TI, f64)>, _>>()?.into_iter()
        .filter(|(_, a_y)| a_y >= &threshold)
        .collect())
}

impl<TI> MakeMeasurement3<VectorDomain<AllDomain<TI>>, HashMapDomain<AllDomain<TI>, AllDomain<f64>>, HammingDistance, SmoothedMaxDivergence, usize, f64, f64> for StabilityCountBy<HammingDistance, TI>
    where TI: 'static + Eq + Hash + Clone {
    fn make3(n: usize, sigma: f64, threshold: f64) -> Measurement<VectorDomain<AllDomain<TI>>, HashMapDomain<AllDomain<TI>, AllDomain<f64>>, HammingDistance, SmoothedMaxDivergence> {
        Measurement::new(
            VectorDomain::new_all(),
            HashMapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() },
            move |data: &Vec<TI>|
                stability_mechanism(count_by(data), sigma, threshold).unwrap(),
            HammingDistance::new(),
            SmoothedMaxDivergence::new(),
            move |&d_in: &u32, &d_out: &(f64, f64)|
                privacy_relation(d_in, d_out, n, sigma, threshold))
    }
}