use std::cmp::Ordering;

use ieee754::Ieee754;
use num::Zero;
use rug::rand::ThreadRandState;

use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{L1Sensitivity, MaxDivergence};
use crate::dom::AllDomain;
use crate::error::Fallible;
use crate::samplers::{GeneratorOpenSSL, SampleRademacher};

// snapping for scalar-valued query
pub fn make_base_snapping(
    scale: f64, sensitivity: f64, min: f64, max: f64, precision: u32,
) -> Fallible<Measurement<AllDomain<f64>, AllDomain<f64>, L1Sensitivity<f64>, MaxDivergence<f64>>> {
    if min > max {
        return fallible!(MakeMeasurement, "lower may not be greater than upper");
    }
    let b = (max - min) / 2.;
    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(move |arg: &f64|
            snapping_mechanism(*arg, scale, sensitivity, min, max, precision)),
        L1Sensitivity::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_fallible(move |&d_in: &f64, &eps: &f64| {
            if eps.is_sign_negative() || eps.is_zero() {
                return fallible!(FailedRelation, "cause: epsilon <= 0");
            }
            if d_in.is_sign_negative() {
                return fallible!(FailedRelation, "sensitivity ({}) must be non-negative", d_in);
            }

            // must be computed before redefining epsilon
            let ideal_precision = compute_precision(eps)?;
            if precision < ideal_precision {
                return fallible!(RelationDebug, "precision must be at least {:?} for this choice of epsilon", ideal_precision);
            }

            // ensure that precision is supported by the OS
            if precision > rug::float::prec_max() {
                return fallible!(FailedRelation, "Operating system does not support sufficient precision to use the Snapping Mechanism");
            }

            // effective epsilon is reduced due to snapping mechanism
            let epsilon = redefine_epsilon(eps, b, precision);
            if epsilon == 0.0 {
                return fallible!(FailedFunction, "epsilon is zero due to floating-point round-off");
            }
            Ok(epsilon > d_in / scale)
        })))
}

/// Computes privatized value according to the Snapping mechanism
///
/// Developed as a variant of the Laplace mechanism which does not suffer from floating-point side channel attacks.
/// For more information, see [Mironov (2012)](http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.366.5957&rep=rep1&type=pdf)
///
/// # Arguments
/// * `value` - Non-private value of the statistic to be privatized.
/// * `epsilon` - Desired privacy guarantee.
/// * `sensitivity` - l1 Sensitivity of function to which mechanism is being applied.
/// * `min` - Lower bound on function value being privatized.
/// * `max` - Upper bound on function value being privatized.
/// * `binding_probability` - Optional. Probability of binding on the final clamp
/// * `enforce_constant_time` - Whether or not to enforce the algorithm to run in constant time
///
/// # Returns
/// Result of snapping mechanism
///
/// # Example
/// ```
/// use opendp::meas::snapping::snapping_mechanism;
/// let value: f64 = 50.0;
/// let scale: f64 = 1.0;
/// let min: f64 = -50.;
/// let max: f64 = 150.0;
/// let sensitivity: f64 = 1.0/1000.0;
/// let precision: i64 = 118;
/// snapping_mechanism(value, scale, sensitivity, min, max, 118).unwrap();
/// println!("snapped value: {}", value);
/// ```
pub fn snapping_mechanism(
    mut value: f64, scale: f64,
    sensitivity: f64, min: f64, max: f64,
    precision: u32,
) -> Fallible<f64> {
    let mut b = (max - min) / 2.;
    let shift = min + b;

    // ~~ preprocess ~~
    // (A) shift mechanism input to be about zero
    value -= shift;
    // (B) clamp by b
    value = value.clamp(-b.abs(), b.abs());
    // (C) scale by sensitivity, to convert quantity to sensitivity-1
    value /= sensitivity;
    b /= sensitivity;

    // ~~ internals ~~
    value = apply_snapping_noise(value, scale, precision)?;

    // ~~ postprocess ~~
    // (C) return to original scale
    value *= sensitivity;
    b *= sensitivity;
    // (B) re-clamp by b
    value = value.clamp(-b.abs(), b.abs());
    // (A) shift mechanism output back to original location
    value += shift;

    Ok(value)
}

#[cfg(feature = "use-mpfr")]
pub fn apply_snapping_noise(
    mut value: f64, scale: f64, precision: u32,
) -> Fallible<f64> {
    macro_rules! to_rug {($v:expr) => {rug::Float::with_val(precision, $v)}}

    let sign = f64::sample_standard_rademacher()?;

    // draw from {d: d in Doubles && d in (0, 1)} with probability based on unit of least precision
    let u_star_sample = {
        let mut rng = GeneratorOpenSSL {};
        let mut state = ThreadRandState::new_custom(&mut rng);
        to_rug!(rug::Float::random_cont(&mut state))
    };
    // let u_star_sample = to_rug!(f64::sample_standard_uniform(enforce_constant_time)?);

    // add noise
    //    rug is mandatory for ln
    //    rug is optional for sign * lambda
    value += (to_rug!(sign * scale) * u_star_sample.ln()).to_f64();

    // snap to lambda
    let m = get_smallest_greater_or_eq_power_of_two(scale)?;
    value = get_closest_multiple_of_lambda(value, m)?;

    Ok(value)
}


/// Finds the smallest integer m such that 2^m is equal to or greater than x.
///
/// # Arguments
/// * `x` - The number for which we want the next power of two.
///
/// # Returns
/// The found power of two
pub fn get_smallest_greater_or_eq_power_of_two(x: f64) -> Fallible<i16> {
    if x.is_sign_negative() {
        return fallible!(FailedFunction, "get_smallest_greater_or_equal_power_of_two must have a positive argument");
    }
    let (_sign, exponent, mantissa) = x.decompose();
    Ok(exponent + if mantissa == 0 { 0 } else { 1 })
}

/// Gets functional epsilon for Snapping mechanism such that privacy loss does not exceed the user's proposed budget.
/// Described in https://github.com/opendp/smartnoise-core/blob/develop/whitepapers/mechanisms/snapping/snapping_implementation_notes.pdf
///
/// # Arguments
/// * `epsilon` - Desired privacy guarantee.
/// * `b` - Upper bound on function value being privatized.
/// * `precision` - Number of bits of precision to which arithmetic inside the mechanism has access.
///
/// # Returns
/// Functional epsilon that will determine amount of noise.
pub fn redefine_epsilon(epsilon: f64, b: f64, precision: u32) -> f64 {
    let eta = (-(precision as f64)).exp2();
    (epsilon - 2.0 * eta) / (1.0 + 12.0 * b * eta)
}

/// Finds accuracy that is achievable given desired epsilon and confidence requirements. Described in
/// https://github.com/opendp/smartnoise-core/blob/develop/whitepapers/mechanisms/snapping/snapping_implementation_notes.pdf
///
/// # Arguments
/// * `alpha` - Desired confidence level.
/// * `epsilon` - Desired privacy guarantee.
/// * `sensitivity` - l1 Sensitivity of function to which mechanism is being applied.
/// * `b` - Upper bound on function value being privatized.
///
/// # Returns
/// Accuracy of the Snapping mechanism.
#[allow(non_snake_case)]
pub fn epsilon_to_accuracy(
    alpha: f64, epsilon: f64, sensitivity: f64, b: f64,
) -> Fallible<f64> {
    let precision = compute_precision(epsilon)?;
    let epsilon = redefine_epsilon(epsilon, b, precision);
    let Lambda = (get_smallest_greater_or_eq_power_of_two(1.0 / epsilon)? as f64).exp2(); // 2^m
    Ok((Lambda / 2. - alpha.ln() / epsilon) * sensitivity)
}

/// Finds epsilon that will achieve desired accuracy and confidence requirements. Described in
/// https://github.com/opendp/smartnoise-core/blob/develop/whitepapers/mechanisms/snapping/snapping_implementation_notes.pdf
///
/// Note that not all accuracies have an epsilon, due to the clamping in the snapping mechanism.
/// In these cases, accuracy is treated as an upper bound,
///   and a larger epsilon is returned that guarantees a tighter accuracy.
///
/// # Arguments
/// * `accuracy` - Desired accuracy level (upper bound).
/// * `alpha` - Desired confidence level.
/// * `sensitivity` - l1 Sensitivity of function to which mechanism is being applied.
///
/// # Returns
/// Epsilon to use for the Snapping mechanism.
pub fn accuracy_to_epsilon(
    accuracy: f64, alpha: f64, sensitivity: f64, b: f64,
) -> Fallible<f64> {

    // bounds for valid epsilon are derived in the whitepaper
    let mut eps_inf = 0.;
    let mut eps_sup = 1. / accuracy;

    let mut acc_prior = f64::NAN;
    let tol = 1e-20f64;

    loop {
        let eps_mid = eps_inf + (eps_sup - eps_inf) / 2.;
        let acc_candidate = epsilon_to_accuracy(alpha, eps_mid, sensitivity, b)?;

        match accuracy.partial_cmp(&acc_candidate) {
            Some(Ordering::Less) => eps_inf = eps_mid,
            Some(Ordering::Greater) => eps_sup = eps_mid,
            Some(Ordering::Equal) => return Ok(eps_mid),
            None => return fallible!(FailedFunction, "non-comparable accuracy")
        }

        let is_stuck = acc_prior == acc_candidate;
        let is_close = acc_candidate < accuracy && (accuracy - acc_candidate) <= tol;

        if is_close || is_stuck {
            return Ok(eps_sup);
        }
        acc_prior = acc_candidate;
    }
}


/// Finds the necessary precision for the snapping mechanism
/// 118 bits required for LN
/// -epsilon.log2().ceil() + 2 bits required for non-zero epsilon
///
/// # Arguments
/// * `epsilon` - privacy usage before redefinition
pub fn compute_precision(epsilon: f64) -> Fallible<u32> {
    Ok(118.max(get_smallest_greater_or_eq_power_of_two(epsilon)? + 2) as u32)
}

/// Finds the closest number to x that is a multiple of Lambda.
///
/// # Arguments
/// * `x` - Number to be rounded to closest multiple of Lambda.
/// * `m` - Integer such that Lambda = 2^m.
///
/// # Returns
/// Closest multiple of Lambda to x.
pub fn get_closest_multiple_of_lambda(x: f64, m: i16) -> Fallible<f64> {
    let (sign, mut exponent, mantissa) = x.decompose();
    exponent -= m;

    let (sign, mut exponent, mantissa) = match exponent {
        // original components already represent an integer (decimal shifted >= 52 places on mantissa)
        exponent if exponent >= 52 => (sign, exponent, mantissa),
        // round int to +- 1
        exponent if exponent == -1 => (sign, 0, 0),
        // round int to 0, and keep it zero after adding m
        exponent if exponent < -1 => (sign, -1023 - m, 0),
        // round to int when decimal is within range of mantissa
        _ => {
            // get elements of mantissa that represent integers (after decimal is shifted by "exponent" places)
            //     shift 1 "exponent" places to the left (no overflow because exponent < 64)
            //     subtract one to set "exponent" bits to one
            //     shift the mask to the left for a 52-bit mask that keeps the top #"exponent" bits
            let integer_mask: u64 = ((1u64 << exponent) - 1) << (52 - exponent);
            let integer_mantissa: u64 = mantissa & integer_mask;

            // check if digit after exponent point is set
            if mantissa & (1u64 << (52 - (exponent + 1))) == 0u64 {
                (sign, exponent, integer_mantissa)
            } else {
                // if integer part of mantissa is all 1s, rounding needs to be reflected in the exponent instead
                if integer_mantissa == integer_mask {
                    (sign, exponent + 1, 0)
                } else {
                    (sign, exponent, integer_mantissa + (1u64 << (52 - exponent)))
                }
            }
        }
    };

    exponent += m;
    Ok(f64::recompose(sign, exponent, mantissa))
}

#[cfg(test)]
mod test_get_closest_multiple_of_lambda {
    use crate::meas::snapping::get_closest_multiple_of_lambda;

    #[test]
    fn test_get_closest_multiple_of_lambda_range() {
        (0..100).for_each(|i| {
            let x = 1. - 0.01 * (i as f64);
            println!("{}: {}", x, get_closest_multiple_of_lambda(x, -1).unwrap())
        });
    }

    #[test]
    fn test_get_closest_multiple_of_lambda() {
        let input = vec![-30.01, -2.51, -1.01, -0.76, -0.51, -0.26, 0.0, 0.26, 0.51, 0.76, 1.01, 2.51, 30.01];

        vec![
            (-2, vec![-30., -2.5, -1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0, 2.5, 30.0]),
            (-1, vec![-30., -2.5, -1.0, -1.0, -0.5, -0.5, 0.0, 0.5, 0.5, 1.0, 1.0, 2.5, 30.0]),
            (0, vec![-30., -3.0, -1.0, -1.0, -1.0, -0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 3.0, 30.0]),
            (1, vec![-30., -2.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 2.0, 30.0]),
            (2, vec![-32., -4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 32.0])
        ].into_iter().for_each(|(m, outputs)| {
            input.iter().copied().zip(outputs.into_iter())
                .for_each(|(input, expected)| {
                    let actual = get_closest_multiple_of_lambda(input, m).unwrap();
                    println!("m: {:?}, input: {:?}, actual: {:?}, expected: {:?}",
                             m, input, actual, expected);
                    assert_eq!(actual, expected)
                })
        });
    }
}