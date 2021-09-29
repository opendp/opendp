use std::cmp::Ordering;

use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{MaxDivergence, SymmetricDistance};
use crate::dom::{AllDomain, BoundedDomain, VectorDomain};
use crate::error::Fallible;
use crate::samplers::SampleLaplace;

fn partial_max<T: PartialOrd>(x: &T, y: &T) -> Ordering {
    x.partial_cmp(y).unwrap_or(Ordering::Equal)
}

fn median_smooth_sensitivity(
    sorted_data: Vec<f64>,
    bounds: (f64, f64),
    epsilon: f64,
) -> f64 {
    let (lower, upper) = bounds;
    let m = (sorted_data.len() + 1) / 2;

    let difference = |k, t| {
        // return lower bound if index is negative
        let l = if m + t < k + 1 { lower } else { sorted_data[m + t - k - 1] };
        // return upper bound if index is past array length
        let u = if m + t >= sorted_data.len() { upper } else { sorted_data[m + t] };
        u - l
    };

    (0..sorted_data.len()).flat_map(|k|
        (0..=k).map(move |t| difference(k, t) * (-(k as f64) * epsilon).exp()))
        .max_by(partial_max).unwrap_or(f64::INFINITY)
}

pub fn make_smooth_sensitivity_median(
    bounds: (f64, f64), influence: u32, epsilon: f64,
) -> Fallible<Measurement<VectorDomain<BoundedDomain<f64>>, AllDomain<f64>, SymmetricDistance, MaxDivergence<f64>>> {
    if influence < 1 { return fallible!(MakeMeasurement, "influence must be positive") }
    if epsilon.is_sign_negative() { return fallible!(MakeMeasurement, "epsilon must be non-negative") }
    let epsilon_prime = epsilon / influence as f64;

    Ok(Measurement::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new_fallible(move |data: &Vec<f64>| {
            let mut sorted_data = data.clone();
            sorted_data.sort_by(partial_max);
            let median = sorted_data[(sorted_data.len() + 1) / 2];

            let sensitivity = median_smooth_sensitivity(sorted_data, bounds, epsilon_prime);
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
    fn test_function() -> Fallible<()> {
        let sens = median_smooth_sensitivity(vec![2., 3., 4.], (1., 0.3), 1.);
        println!("{}", sens);

        let meas = make_smooth_sensitivity_median((-1., 5.), 1, 1.)?;

        let res= meas.invoke(&vec![-0.05, 2., 3., 0., 4., 1.2, 0.7, 1.3])?;
        println!("{}", res);
        Ok(())
    }
}
