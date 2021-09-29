use crate::core::{Domain, Function, Measurement, Metric, PrivacyRelation};
use crate::dist::{MaxDivergence, SymmetricDistance};
use crate::dom::{AllDomain, BoundedDomain, VectorDomain};
use crate::error::Fallible;
use std::cmp::Ordering;
use crate::samplers::SampleLaplace;

fn median_smooth_sensitivity(
    epsilon: f64,
    bounds: (f64, f64),
    sorted_data: Vec<f64>,
) -> f64 {
    let (lower, upper) = bounds;

    let m = (sorted_data.len() + 1) / 2;
    let difference = |t, k|
        sorted_data.get(m + t).unwrap_or(&upper) -
            sorted_data.get(m + t - k - 1).unwrap_or(&lower);

    (0..sorted_data.len()).flat_map(move |k|
        (0..=k).filter(move |t| m + t > k + 1).map(move |t|
            difference(t, k) * (-(k as f64) * epsilon).exp()))
        .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
        .unwrap_or(f64::INFINITY)
}

pub fn make_smooth_sensitivity_median<DIA: Domain, DO: Domain, MI: 'static + Metric>(
    bounds: (f64, f64), influence: u32, epsilon: f64,
) -> Fallible<Measurement<VectorDomain<BoundedDomain<f64>>, AllDomain<f64>, SymmetricDistance, MaxDivergence<f64>>> {
    if influence < 1 { return fallible!(MakeMeasurement, "influence must be positive") }
    if epsilon.is_sign_negative() { return fallible!(MakeMeasurement, "budget must be non-negative") }
    let epsilon_prime = epsilon / influence as f64;

    Ok(Measurement::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new_fallible(move |data: &Vec<f64>| {
            let mut sorted_data = data.clone();
            sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            let median = sorted_data[(sorted_data.len() + 1) / 2];

            let sensitivity = median_smooth_sensitivity(epsilon_prime, bounds, sorted_data);
            f64::sample_laplace(median, sensitivity / epsilon_prime, false)
        }),
        SymmetricDistance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new(
            move |&d_in, &d_out: &f64| d_in <= influence && epsilon <= d_out),
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_function() {
        let sens = median_smooth_sensitivity(1., (1., 0.3), vec![2., 3., 4.]);
        println!("{}", sens);
    }
}
