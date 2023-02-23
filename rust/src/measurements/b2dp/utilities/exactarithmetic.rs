//! Implements methods requiring exact arithmetic, and encapsulates all
//! `unsafe` code to access `mpfr::flags` to determine whether computations
//! are exact.

use rug::{rand::ThreadRandGen, rand::ThreadRandState, Assign, Float};

use crate::error::Fallible;

use super::params::Eta;
use gmp_mpfr_sys::mpfr;

/// Randomized Rounding
/// ## Arguments
///   * `x`: the value to round
///   * `arithmetic_config`: the arithmetic configuration to use
/// ## Returns
/// `x` rounded to the nearest smaller or larger integer by drawing a random value
/// `rho` in `[0,1]` and rounding down if `rho > x_fract`, rounding up otherwise.
pub fn randomized_round<R: ThreadRandGen>(
    x: f64,
    arithmetic_config: &ArithmeticConfig,
    rng: &mut R,
) -> u32 {
    // if x is already integer, return it
    if x.trunc() == x {
        return x as u32;
    }

    let x_fract = x.fract(); // fractional part of x
    let x_trunc = x.trunc() as u32; // integer part of x
                                    // Draw a random value
    let rho = arithmetic_config.get_rand_float(rng);
    if rho > x_fract {
        return x_trunc; // round down
    } else {
        return x_trunc + 1; // round up
    }
}

/// Determine smallest `k` such that `2^k >= total_weight`.
/// Returns zero if `total_weight` <= 0.
fn get_power_bound(total_weight: &Float, arithmetic_config: &ArithmeticConfig) -> i32 {
    let mut k: i32 = 0;
    if *total_weight <= 0 {
        return 0;
    }
    if *total_weight > 1 {
        // increase `k` until `2^k >= total_weight`.
        let mut two_exp_k = Float::i_pow_u(2, k as u32);
        while arithmetic_config.get_float(two_exp_k) < *total_weight {
            k += 1;
            two_exp_k = Float::i_pow_u(2, k as u32);
        }
    } else {
        let mut w = arithmetic_config.get_float(total_weight);
        while w <= 1 {
            k -= 1;
            w *= 2;
        }
        k += 1;
    }
    k
}

/// Normalized Weighted Sampling
/// Returns the index of the element sampled according to the weights provided.
/// Uses optimized sampling if `optimize` set to true. Setting `optimize` to true
/// exacerbates timing channels.
/// ## Arguments
///   * `weights`: the set of weights to use for sampling; all weights must be positive,
///                zero-weight elements are not permitted.
///   * `arithmetic_config`: the arithmetic config specifying precision
///   * `rng`: source of randomness.
///   * `optimize`: whether to optimize sampling, introducing a timing channel and an error condition
///                 side channel.
/// ## Returns
/// Returns an index of an element sampled according to the weights provided. If the precision
/// of the provided ArithmeticConfig is insufficient for sampling, the method returns an error.
/// Note that errors are **not** returned on inexact arithmetic, and the caller is responsible
/// for calling `enter_exact_scope()` and  `exit_exact_scope()` to monitor inexact arithmetic.
///
///
/// ## Known Timing Channels
/// This method has known timing channels. They result from:
/// (1) Generating a random value in [0,2^k] and
/// (2) (In optimized sampling only) To determine the index corresponding to the random value,
/// the method iterates through cumulative weights
/// and terminates the loop when the index is found and
/// (3) (In optimized sampling only) Checking for zero weights
/// These can be exploited in several ways:
///   * **Rejection probability:** if the adversary can control the total weight of the utilities
///     such that the probability of rejection in the random index generation stage changes,
///     the time needed for sampling will vary between adjacent databases. The difference in time
///     will depend on the speed of random number generation. By default, ArithmeticConfig sets the
///     minimum retries to 1. To reduce the probability that this timing channel is accessible to an
///     adversary, the minimum number of retries can be increased via `ArithmeticConfig::set_retries`.
///   * **Optimized sampling:**
///     * **Ordering of weights:** if the adversary can change the ordering of the weights such
///       that the largest weights (most probable) weights are first under a certain condition,
///       and the largest weights are last if that condition doesn't hold, then the adversary
///       can use the time of normalized_sample to guess whether the condition holds.
///     * **Size of weights:** if the adversary can change the size of the weights such that if
///       a certain condition holds, the weight is more concentrated and if not the weight is less
///       concentrated, then the adversary can use the time taken by normalized_sample as a signal
///       for whether the condition holds.
///     * **Zero weight:** optimized sampling also rejects immediately if a zero weight is encountered.
///       If the adversary can inject a zero weight at a particular position in the weights depending on
///       a private condition, they can use the time it takes to return an error as a timing channel.
///
/// The timing channels for optimized sampling could be somewhat (but not completely) mitigated by
/// shuffling the weights prior to calling `normalized_sample`.
/// ### Exact Arithmetic
/// `normalized_sample` does not explicitly call `enter_exact_scope()` or
/// `exit_exact_scope()`, and therefore preserves any `mpfr::flags` that
/// are set before the function is called.

pub fn normalized_sample<R: ThreadRandGen>(
    weights: &Vec<Float>,
    arithmetic_config: &ArithmeticConfig,
    rng: &mut R,
) -> Fallible<usize> {
    // Compute the total weight
    let total_weight = arithmetic_config.get_float(Float::sum(weights.iter()));
    if total_weight == 0 {
        return fallible!(
            FailedFunction,
            "Total weight zero. Weights must be positive."
        );
    }
    let mut zero_weight: Option<()> = None;

    // Iterate through all weights to test to prevent timing channel,
    for w in weights.iter() {
        if w.is_zero() {
            zero_weight = Some(());
        }
    }

    if zero_weight.is_some() {
        return fallible!(FailedFunction, "All weights must be positive.");
    }
    // Determine smallest `k` such that `2^k >= total_weight`
    let k = get_power_bound(&total_weight, arithmetic_config);

    let mut t = arithmetic_config.get_float(&total_weight);
    let mut retries = 0;

    t += 1; // ensure that the initial `t` is larger than `total_weight`.
    while t >= total_weight || retries < arithmetic_config.retry_min {
        let mut s = arithmetic_config.get_rand_float(rng);
        // Multiply by 2^k to scale
        // Note: Float::i_exp(a,b) returns a*2^b
        let two_pow_k = arithmetic_config.get_float(Float::i_exp(1, k));
        s = s * two_pow_k;
        // Assign to t if in bounds
        if s < total_weight {
            t = arithmetic_config.get_float(&s);
        }
        retries += 1; // increment retries
    }
    if t >= total_weight {
        return fallible!(FailedFunction, "Failed to produce t");
    }
    let mut cumulative_weight = arithmetic_config.get_float(0);
    let mut index: Option<usize> = None;
    let mut prec_error: bool = false;

    // Iterate through the weights until the cumulative weight is greater than or equal to `t`
    for i in 0..weights.len() {
        let next_weight = arithmetic_config.get_float(&weights[i]);
        cumulative_weight += next_weight;
        if cumulative_weight > t {
            // This is the index to return
            if index.is_none() {
                // Check sufficient precision
                let mut next_highest = arithmetic_config.get_float(&t);
                next_highest.next_up();
                if i < weights.len() - 1 {
                    let next_weight = arithmetic_config.get_float(&weights[i + 1]);
                    let mut cumulative_next = arithmetic_config.get_float(&cumulative_weight);
                    cumulative_next = cumulative_next + next_weight;
                    if cumulative_next < next_highest {
                        prec_error = true;
                    }
                }
                index = Some(i);
            }
        }
    }

    if prec_error == true {
        return fallible!(FailedFunction, "Sampling precision insufficient");
    }
    if index.is_some() {
        return Ok(index.unwrap());
    }

    // Return an error if we are unable to sample
    // Caller can choose an index at random if needed
    fallible!(FailedFunction, "Unable to sample.")
}

/// The exact arithmetic configuration. Includes the precision of all
/// mechanism arithmetic and status bits indicating if any inexact
/// arithmetic has been performed.
/// The ArithmeticConfig implementation encapsulates all `unsafe` calls to
/// `mpfr`.
#[derive(Debug)]
pub struct ArithmeticConfig {
    /// The required precision (computed based on other parameters)
    pub precision: u32,
    /// Whether an inexact operation has been performed in the scope of
    /// this config
    pub inexact_arithmetic: bool,
    /// Whether the code is currently in an exact scope
    exact_scope: bool,
    /// The number of retries for timing channel prevention
    /// default is 1.
    retry_min: u32,
}

impl ArithmeticConfig {
    /// A basic arithmetic_config with default precision
    pub fn basic() -> Fallible<ArithmeticConfig> {
        let p; //= 53;
        unsafe {
            p = mpfr::get_default_prec() as u32;
        }
        let config = ArithmeticConfig {
            precision: p,
            inexact_arithmetic: false,
            exact_scope: false,
            retry_min: 1,
        };
        Ok(config)
    }

    /// Initialize an ArithmeticConfig for the base-2 exponential mechanism.
    /// This method determines the precision required to compute a linear
    /// combination of at most `max_outcomes` weights in the provided utility range.
    /// Note that the precision to create Floats in rug/mpfr is given as a `u32`, but the
    /// sizes (min, max, etc) of precision returned (e.g. `mpfr::PREC_MAX`) are `i64`.
    /// We handle this by explicitly checking that `mpfr::PREC_MAX` does not exceed the
    /// maximum value for a `u32` (this should never happen, but we check anyway).
    ///
    /// ## Arguments
    ///   * `eta`: the base-2 privacy parameter
    ///   * `utility_min`: the minimum utility permitted by the mechanism (highest possible weight)
    ///   * `utility_max`: the maximum utility permitted by the mechanism (lowest possible weight)
    ///   * `max_outcomes`: the maximum number of outcomes permitted by this instance of the exponential
    ///                     mechanism.
    ///
    /// ## Returns
    /// Returns an ArithmeticConfig with sufficient precision to carry out the operations for the
    /// exponential mechanism with the given parameters.
    ///
    /// ## Errors
    /// Returns an error if sufficient precision cannot be determined.
    pub fn for_exponential(
        eta: &Eta,
        _utility_min: u32,
        utility_max: u32,
        max_outcomes: u32,
        min_retries: u32,
    ) -> Fallible<ArithmeticConfig> {
        let mut p: u32;

        // Clear the flags
        unsafe {
            mpfr::clear_flags();
        }

        // Check that the maximum precision does not exceed the maximum value of a
        // u32. Precision for Float::with_val(precision: u32, val) requires a u32.
        let mut max_precision = u32::max_value();
        if mpfr::PREC_MAX < max_precision as i64 {
            max_precision = mpfr::PREC_MAX as u32;
        }
        p = eta.z * eta.y;
        let mut um = utility_max; //.abs() as u32;
        if um < 1 {
            um += 1;
        }
        p = p * um;
        p = p + 2 + max_outcomes;

        if p > max_precision {
            return fallible!(FailedFunction, "Maximum precision exceeded.");
        }

        let config = ArithmeticConfig {
            precision: p,
            inexact_arithmetic: false,
            exact_scope: false,
            retry_min: min_retries,
        };
        Ok(config)
    }

    /// Increase the precision by `increment`. Returns an error and leaves precision unchanged
    /// if this results in precision exceeding `mpfr::PREC_MAX`.
    pub fn increase_precision(&mut self, increment: u32) -> Fallible<()> {
        let new_precision = self.precision + increment;
        // Check that precision doesn't exceed maximum
        if new_precision as i64 > mpfr::PREC_MAX {
            return fallible!(FailedFunction, "Exceeds maximum precision");
        }
        self.precision = new_precision;
        Ok(())
    }

    /// Check the current state of the flags
    pub fn check_mpfr_flags() -> Fallible<()> {
        unsafe {
            let flags = mpfr::flags_save();
            if flags > 0 {
                if mpfr::inexflag_p() > 0 {
                    return fallible!(FailedFunction, "Inexact arithmetic.");
                } else {
                    return fallible!(
                        FailedFunction,
                        "Arithmetic error other than inexact (see mpfr::flags)"
                    );
                }
            }
        }
        Ok(())
    }

    /// Set the minimum number of retries for timing channel prevention.
    pub fn set_retries(&mut self, retry_min: u32) -> Fallible<()> {
        self.retry_min = retry_min;
        Ok(())
    }

    /// Invalidates the config
    pub fn invalidate(&mut self) {
        self.inexact_arithmetic = true;
    }

    /// Enter exact arithmetic scope.
    /// This method clears `mpfr` flags if not currently in an `exact_scope`.
    /// # Returns
    ///   * `OK(())` if the scope is successfully entered
    ///   * `Err` if the scope is alread invalid
    pub fn enter_exact_scope(&mut self) -> Fallible<()> {
        if self.inexact_arithmetic {
            // inexact arithmetic has already occurred
            return fallible!(FailedFunction, "ArithmeticConfiguration invalid.");
        }
        if !self.exact_scope {
            unsafe {
                mpfr::clear_flags();
            }
            // set the exact_scope flag
            self.exact_scope = true;
        }

        return ArithmeticConfig::check_mpfr_flags();
    }

    /// Exit the exact arithmetic scope.
    /// **Must be called after any arithmetic operations are performed which should be exact.**
    /// **Must be paired with `enter_exact_scope` to ensure that flags aren't misinterpreted.**
    /// This method checks the `mpfr` flag state, and returns whether
    /// the scope is still valid. Also sets the `inexact` property.
    /// This method does **not** reset the `mpfr` flags.
    ///
    /// ## Returns
    ///   * `OK(())` if the configuration reports than no inexact arithmetic was performed
    ///   * `Err` if the configuration is invalid (inexact arithmetic performed)
    pub fn exit_exact_scope(&mut self) -> Fallible<()> {
        if !self.exact_scope {
            return fallible!(FailedFunction, "Not in exact scope.");
        }

        if self.inexact_arithmetic {
            // Error has already occurred
            return fallible!(FailedFunction, "ArithmeticConfiguration invalid.");
        }

        let result = ArithmeticConfig::check_mpfr_flags();
        if result.is_err() {
            self.invalidate();
        }
        // set the exact_scope status to false
        self.exact_scope = false;
        return result;
    }

    /// Get a Float with value `T` and precision `self.precision`.
    /// This method avoid redudnant boilerplate code of the form
    /// `let x = Float::with_val(arithmetic_config.precision,val)`.
    ///
    /// ### Arguments
    ///   * `val`: the value to assign
    ///
    /// ### Returns
    /// A `Float` with precision `self.precision` and value `val`.
    pub fn get_float<T>(&self, val: T) -> Float
    where
        Float: Assign<T>,
    {
        Float::with_val(self.precision, val)
    }

    /// Get a Float with random bits from `rng` and precision `self.precision`.
    ///
    /// ### Arguments
    ///   * `rng`: Randomness source
    ///
    /// ### Returns
    /// A `Float` with precision `self.precision` and value of random bits
    /// provided by `rng`.
    pub fn get_rand_float<R: ThreadRandGen>(&self, rng: &mut R) -> Float {
        let mut f = Float::new(self.precision);
        let mut rand_state = ThreadRandState::new_custom(rng);
        f.assign(Float::random_bits(&mut rand_state));
        f
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::samplers::GeneratorOpenDP;
    use rug::ops::Pow;

    #[test]
    fn test_get_float() {
        let arithmetic_config = ArithmeticConfig::basic().unwrap();
        let f = arithmetic_config.get_float(1.5);
        assert_eq!(f, 1.5);
        assert_eq!(f.prec(), arithmetic_config.precision);
    }
    #[test]
    fn test_get_random_float() {
        let mut arithmetic_config = ArithmeticConfig::basic().unwrap();
        assert!(arithmetic_config.increase_precision(1600).is_ok());
        let mut rng = GeneratorOpenDP::default();
        let rho = arithmetic_config.get_rand_float(&mut rng);
        assert_eq!(rho.prec(), arithmetic_config.precision);
        assert!(rho.prec() > 1600);
    }

    #[test]
    fn test_all_zero_weight_sampling() {
        // Generate an arithmetic config
        let mut rng = GeneratorOpenDP::default();
        let mut arithmetic_config = ArithmeticConfig::basic().unwrap();
        arithmetic_config.precision = 2;

        let a = Float::with_val(arithmetic_config.precision * 4, 0.0);
        let b = Float::with_val(arithmetic_config.precision * 4, 0.0);
        let c = Float::with_val(arithmetic_config.precision * 4, 0.0);
        let d = Float::with_val(arithmetic_config.precision * 4, 0.0);
        let mut weights: Vec<Float> = Vec::new();
        weights.push(a);
        weights.push(b);
        weights.push(c);
        weights.push(d);

        // this example should fail due to zero total weight
        let mut exact = arithmetic_config.enter_exact_scope();
        assert!(exact.is_ok());
        if let Err(e) = normalized_sample(&weights, &arithmetic_config, &mut rng) {
            assert_eq!(
                e.message,
                Some("Total weight zero. Weights must be positive.".to_string())
            );
        }

        exact = arithmetic_config.exit_exact_scope();
        assert!(exact.is_ok());
    }

    #[test]
    fn test_zero_weight_sampling() {
        // Generate an arithmetic config
        let mut rng = GeneratorOpenDP::default();
        let mut arithmetic_config = ArithmeticConfig::basic().unwrap();
        arithmetic_config.precision = 2;

        let a = Float::with_val(arithmetic_config.precision * 4, 1.0);
        let b = Float::with_val(arithmetic_config.precision * 4, 1.0);
        let c = Float::with_val(arithmetic_config.precision * 4, 1.0);
        let d = Float::with_val(arithmetic_config.precision * 4, 0.0);
        let mut weights: Vec<Float> = Vec::new();
        weights.push(a);
        weights.push(b);
        weights.push(c);
        weights.push(d);

        // this example should fail due to a zero weight
        let mut exact = arithmetic_config.enter_exact_scope();
        assert!(exact.is_ok());
        if let Err(e) = normalized_sample(&weights, &arithmetic_config, &mut rng) {
            assert_eq!(e.message.unwrap(), "All weights must be positive.");
        }
        exact = arithmetic_config.exit_exact_scope();
        assert!(exact.is_ok());
    }

    /// Test normalized sampling in the case when precision
    /// is insufficient for correct sampling
    #[test]
    fn test_insufficient_sampling_precision() {
        // Generate an arithmetic config
        let mut rng = GeneratorOpenDP::default();
        let mut arithmetic_config = ArithmeticConfig::basic().unwrap();
        arithmetic_config.precision = 2;

        let a = Float::with_val(arithmetic_config.precision * 4, 1.0 / 64.0);
        let b = Float::with_val(arithmetic_config.precision * 4, 1.0 / 32.0);
        let c = Float::with_val(arithmetic_config.precision * 4, 1.0 / 16.0);
        let d = Float::with_val(arithmetic_config.precision * 4, 1.0 / 64.0);
        let mut weights: Vec<Float> = Vec::new();
        weights.push(a);
        weights.push(b);
        weights.push(c);
        weights.push(d);

        // this example should fail due to inexact arithmetic
        let mut exact = arithmetic_config.enter_exact_scope();
        assert!(exact.is_ok());
        let result = normalized_sample(&weights, &arithmetic_config, &mut rng);
        let _s = result.unwrap();
        exact = arithmetic_config.exit_exact_scope();
        assert!(exact.is_err());
    }

    #[test]
    fn test_power_bound() {
        // Generate an arithmetic config
        let eta = &Eta::new(1, 1, 1).unwrap();
        let utility_min = 0;
        let utility_max = 100;
        let max_outcomes = 10;
        let arithmetic_config_result = ArithmeticConfig::for_exponential(
            eta,
            utility_min,
            utility_max,
            max_outcomes,
            1,
        );
        assert!(arithmetic_config_result.is_ok());
        let mut arithmetic_config = arithmetic_config_result.unwrap();

        arithmetic_config.enter_exact_scope().unwrap();
        let mut x = Float::with_val(arithmetic_config.precision, 1.25);
        let mut r = get_power_bound(&x, &arithmetic_config);
        assert_eq!(r, 1);

        let y = Float::with_val(arithmetic_config.precision, 1);
        let s = get_power_bound(&y, &arithmetic_config);
        assert_eq!(s, 0);

        let y = Float::with_val(arithmetic_config.precision, 0.5);
        let s = get_power_bound(&y, &arithmetic_config);
        assert_eq!(s, -1);

        let y = Float::with_val(arithmetic_config.precision, 0.35);
        let s = get_power_bound(&y, &arithmetic_config);
        assert_eq!(s, -1);

        x = Float::with_val(arithmetic_config.precision, 0.75);
        r = get_power_bound(&x, &arithmetic_config);
        assert_eq!(r, 0);

        x = Float::with_val(arithmetic_config.precision, 5.75);
        r = get_power_bound(&x, &arithmetic_config);
        assert_eq!(r, 3);

        x = Float::with_val(arithmetic_config.precision, 0.0625); // 1/16 = 2^(-4)
        r = get_power_bound(&x, &arithmetic_config);
        assert_eq!(r, -4);

        x = Float::with_val(arithmetic_config.precision, 16);
        r = get_power_bound(&x, &arithmetic_config);
        assert_eq!(r, 4);

        // Test weights <= 0
        let y = Float::with_val(arithmetic_config.precision, 0);
        let s = get_power_bound(&y, &arithmetic_config);
        assert_eq!(s, 0);

        let y = Float::with_val(arithmetic_config.precision, -1);
        let s = get_power_bound(&y, &arithmetic_config);
        assert_eq!(s, 0);

        arithmetic_config.exit_exact_scope().unwrap();
    }

    /// Test flag behavior of mpfr.
    /// This is a canary test that tests some of the basic properties
    /// of the flags and expected behavior. Failure of this test should
    /// be considered critical, as critical assumptions may be broken.
    #[test]
    fn test_flags() {
        let precision = 53;
        // Use an unsafe block
        unsafe {
            // clear the flags
            mpfr::clear_flags();
            let mut flags = mpfr::flags_save();
            assert_eq!(flags, 0);

            // divide 1 by 3 to get an inexact result
            let x = Float::with_val(precision, 1.0);
            let y = Float::with_val(precision, 3.0);
            let _z = x / y;

            // Test the specific flag value
            flags = mpfr::flags_save();
            assert_eq!(flags, 8);

            // Test the inexflag directly
            assert_eq!((mpfr::inexflag_p() > 0), true);
            // Confirm other flags not set
            assert_eq!(
                (mpfr::underflow_p()
                    + mpfr::overflow_p()
                    + mpfr::divby0_p()
                    + mpfr::nanflag_p()
                    + mpfr::erangeflag_p())
                    > 0,
                false
            );

            // Do some exact arithmetic
            let a = Float::with_val(precision, 5.0);
            let b = Float::with_val(precision, 6.0);
            let c = a + b;
            // Test the specific flag value is preserved
            flags = mpfr::flags_save();
            assert_eq!(flags, 8);

            // Test the inexflag directly
            assert_eq!((mpfr::inexflag_p() > 0), true);
            // Confirm other flags not set
            assert_eq!(
                (mpfr::underflow_p()
                    + mpfr::overflow_p()
                    + mpfr::divby0_p()
                    + mpfr::nanflag_p()
                    + mpfr::erangeflag_p())
                    > 0,
                false
            );

            // Clear the flags and do some exact arithmetic
            mpfr::clear_flags();
            let d = Float::with_val(precision, 7.0);
            let _e = d * c;
            flags = mpfr::flags_save();
            assert_eq!(flags, 0);

            // Check that creating a value too large for the given precision
            // results in flags
            mpfr::clear_flags();
            let f = Float::with_val(precision, i64::max_value());
            // Confirm that precision isn't modified
            assert_eq!(f.prec(), precision);
            flags = mpfr::flags_save();
            assert!(flags > 0);
            assert!(mpfr::inexflag_p() > 0);
            let g = Float::with_val(precision, i64::max_value());
            // Clear the flags
            let h = f * g;
            assert_eq!(h.prec(), precision);
            flags = mpfr::flags_save();
            assert!(flags > 0);

            // Check overflow behavior
            mpfr::clear_flags();
            let max_u_precision = 3;
            let i = Float::with_val(max_u_precision, 16);
            assert_eq!(i.prec(), max_u_precision);
            let j = Float::with_val(max_u_precision, i + 2);
            assert_eq!(j.prec(), max_u_precision);
            assert!(j - 2 != 16);
            flags = mpfr::flags_save();
            assert!(flags > 0);
            assert!(mpfr::inexflag_p() > 0); // This sets the inexact flag rather than overflow

            // Check precision inheritance behavior
            // Addition results in a Float with precision of the first
            // element in the sum.
            mpfr::clear_flags();
            let k = Float::with_val(max_u_precision, 16);
            assert_eq!(k.prec(), max_u_precision);
            let l = Float::with_val(max_u_precision + 1, 20);
            assert_eq!(l.prec(), max_u_precision + 1);
            let m = k + l; // Switching the order of l and k will cause the test to fail.
            flags = mpfr::flags_save();
            assert!(flags > 0);
            assert_eq!(m.prec(), max_u_precision);
        }
    }

    #[test]
    fn test_high_precision_arithmetic_config_for_exponential() {
        let eta = &Eta::new(1, 2, 3).unwrap();
        let utility_min = 0;
        let utility_max = 2u32.pow(10);
        let max_outcomes = 2u32.pow(8);
        let arithmetic_config_result = ArithmeticConfig::for_exponential(
            eta,
            utility_min,
            utility_max,
            max_outcomes,
            1,
        );
        assert!(arithmetic_config_result.is_ok());
        let arithmetic_config = arithmetic_config_result.unwrap();
        assert!(arithmetic_config.precision >= 6000);

        let emp_arithmetic_config =
            ArithmeticConfig::for_exponential(eta, utility_min, utility_max, max_outcomes, 1)
                .unwrap();
        //assert_eq!(arithmetic_config.precision,6402);
        //assert_eq!(emp_arithmetic_config.precision,0);
        assert!(arithmetic_config.precision * 2 >= emp_arithmetic_config.precision);
    }

    #[test]
    fn test_high_precision_eta_arithmetic_config_for_exponential() {
        let eta = &Eta::new(31, 5, 18).unwrap();
        let utility_min = 0;
        let utility_max = 1;
        let max_outcomes = 1;
        let arithmetic_config_result = ArithmeticConfig::for_exponential(
            eta,
            utility_min,
            utility_max,
            max_outcomes,
            1,
        );
        assert!(arithmetic_config_result.is_ok());
        let arithmetic_config = arithmetic_config_result.unwrap();
        // assert_eq!(arithmetic_config.precision,0);
        assert!(arithmetic_config.precision >= 90);

        let emp_arithmetic_config =
            ArithmeticConfig::for_exponential(eta, utility_min, utility_max, max_outcomes, 1)
                .unwrap();

        assert!(arithmetic_config.precision * 2 >= emp_arithmetic_config.precision);
        assert!(53 <= emp_arithmetic_config.precision);
    }

    #[test]
    fn test_arithmetic_config_for_exponential() {
        let eta = &Eta::new(1, 1, 1).unwrap();
        let utility_min = 0;
        let utility_max = 100;
        let max_outcomes = 10;
        let arithmetic_config_result = ArithmeticConfig::for_exponential(
            eta,
            utility_min,
            utility_max,
            max_outcomes,
            1,
        );
        assert!(arithmetic_config_result.is_ok());
        let arithmetic_config = arithmetic_config_result.unwrap();
        assert!(arithmetic_config.precision >= 8);
    }

    #[test]
    fn test_exact_scope() {
        let eta = &Eta::new(1, 1, 1).unwrap();
        let utility_min = 0;
        let utility_max = 100;
        let max_outcomes = 10;
        let arithmetic_config_result = ArithmeticConfig::for_exponential(
            eta,
            utility_min,
            utility_max,
            max_outcomes,
            1,
        );
        assert!(arithmetic_config_result.is_ok());
        let mut arithmetic_config = arithmetic_config_result.unwrap();
        let working_precision = arithmetic_config.precision;

        // Test good behavior in exact scope
        // Enter exact scope
        let enter1 = arithmetic_config.enter_exact_scope();
        assert!(enter1.is_ok());

        // Do some arithmetic that should all be exact
        // Do some exact arithmetic
        let base_result = eta.get_base(working_precision);
        let base = &base_result.unwrap();
        let mut weight_sum = Float::with_val(working_precision, 0);
        for _i in 0..max_outcomes {
            let new_weight_sum =
                weight_sum + Float::with_val(working_precision, base.pow(utility_min));
            weight_sum = new_weight_sum;
        }
        assert_eq!(10, weight_sum);
        // Exit exact scope
        let exit1 = arithmetic_config.exit_exact_scope();
        assert!(exit1.is_ok());

        // Test bad behavior in exact scope
        // Enter exact scope
        let enter2 = arithmetic_config.enter_exact_scope();
        assert!(enter2.is_ok());

        // Do some arithmetic that should raise flags
        let x = Float::with_val(working_precision, 1.0);
        let y = Float::with_val(working_precision, 3.0);
        let _z = x / y;

        // Exit exact scope
        let exit2 = arithmetic_config.exit_exact_scope();
        assert!(exit2.is_err());
        assert!(arithmetic_config.inexact_arithmetic);

        // Try to enter after bad behavior
        // Enter exact scope
        let enter2 = arithmetic_config.enter_exact_scope();
        assert!(enter2.is_err());
    }
    #[test]
    fn test_optimized_normalized_sample() {
        // Generate an arithmetic config
        let eta = &Eta::new(1, 1, 1).unwrap();
        let utility_min = 0;
        let utility_max = 10;
        let max_outcomes = 10;
        let mut rng = GeneratorOpenDP::default();
        let arithmetic_config_result = ArithmeticConfig::for_exponential(
            eta,
            utility_min,
            utility_max,
            max_outcomes,
            1,
        );
        assert!(arithmetic_config_result.is_ok());
        let mut arithmetic_config = arithmetic_config_result.unwrap();

        arithmetic_config.enter_exact_scope().unwrap();
        let n = 1000;
        // Construct a vector of equal weights and test we are getting
        // approximately equal probabilities
        let a = Float::with_val(arithmetic_config.precision, 1);
        let b = Float::with_val(arithmetic_config.precision, 1);
        let c = Float::with_val(arithmetic_config.precision, 1);
        let mut weights: Vec<Float> = Vec::new();
        weights.push(a);
        weights.push(b);
        weights.push(c);
        let mut counts = [0; 3];
        for _i in 0..n {
            let j = normalized_sample(&weights, &arithmetic_config, &mut rng).unwrap();
            counts[j] += 1;
        }

        arithmetic_config.exit_exact_scope().unwrap();

        let mut probs = [0.0; 3];
        for i in 0..counts.len() {
            probs[i] = (counts[i] as f64) / (n as f64);
            assert!(probs[i] - 0.333 < 0.05);
        }
    }

    #[test]
    fn test_normalized_sample() {
        // Generate an arithmetic config
        let eta = &Eta::new(1, 1, 1).unwrap();
        let utility_min = 0;
        let utility_max = 10;
        let max_outcomes = 10;
        let mut rng = GeneratorOpenDP::default();
        let arithmetic_config_result = ArithmeticConfig::for_exponential(
            eta,
            utility_min,
            utility_max,
            max_outcomes,
            1,
        );
        assert!(arithmetic_config_result.is_ok());
        let mut arithmetic_config = arithmetic_config_result.unwrap();

        arithmetic_config.enter_exact_scope().unwrap();
        let n = 1000;
        // Construct a vector of equal weights and test we are getting
        // approximately equal probabilities
        let a = Float::with_val(arithmetic_config.precision, 1);
        let b = Float::with_val(arithmetic_config.precision, 1);
        let c = Float::with_val(arithmetic_config.precision, 1);
        let mut weights: Vec<Float> = Vec::new();
        weights.push(a);
        weights.push(b);
        weights.push(c);
        let mut counts = [0; 3];
        for _i in 0..n {
            let j = normalized_sample(&weights, &arithmetic_config, &mut rng).unwrap();
            counts[j] += 1;
        }

        let mut probs = [0.0; 3];
        for i in 0..counts.len() {
            probs[i] = (counts[i] as f64) / (n as f64);
            assert!(probs[i] - 0.333 < 0.05);
        }

        // Construct a vector with different weights, and confirm that
        // we still see low probability weights sometimes.
        weights.push(Float::with_val(arithmetic_config.precision, 0.0625));
        let mut new_counts = [0; 4];
        for _i in 0..n {
            let j = normalized_sample(&weights, &arithmetic_config, &mut rng).unwrap();
            new_counts[j] += 1;
        }

        let mut new_probs = [0.0; 4];
        let new_expected_probs = [0.327, 0.327, 0.327, 0.02];
        for i in 0..counts.len() {
            new_probs[i] = (new_counts[i] as f64) / (n as f64);
            assert!(new_probs[i] - new_expected_probs[i] < 0.05);
        }

        arithmetic_config.exit_exact_scope().unwrap();
    }

    #[test]
    fn test_randomized_round() {
        // Generate an arithmetic config
        let eta = &Eta::new(1, 1, 1).unwrap();
        let utility_min = 0;
        let utility_max = 100;
        let max_outcomes = 10;
        let mut rng = GeneratorOpenDP::default();
        let arithmetic_config_result = ArithmeticConfig::for_exponential(
            eta,
            utility_min,
            utility_max,
            max_outcomes,
            1,
        );
        assert!(arithmetic_config_result.is_ok());
        let mut arithmetic_config = arithmetic_config_result.unwrap();

        // Enter Exact scope
        arithmetic_config.enter_exact_scope().unwrap();
        let x = 1.25;
        let r = randomized_round(x, &arithmetic_config, &mut rng);
        assert!((x - (r as f64)).abs() < 1.0);

        // Exit exact scope
        arithmetic_config.exit_exact_scope().unwrap();
    }

    #[test]
    fn test_min_retries() {
        // Generate an arithmetic config
        let eta = &Eta::new(1, 1, 1).unwrap();
        let utility_min = 0;
        let utility_max = 10;
        let max_outcomes = 10;
        let mut rng = GeneratorOpenDP::default();
        let min_retries = 5;
        let arithmetic_config_result = ArithmeticConfig::for_exponential(
            eta,
            utility_min,
            utility_max,
            max_outcomes,
            min_retries,
        );
        assert!(arithmetic_config_result.is_ok());
        let mut arithmetic_config = arithmetic_config_result.unwrap();

        arithmetic_config.enter_exact_scope().unwrap();
        let n = 1000;
        // Construct a vector of equal weights and test we are getting
        // approximately equal probabilities
        let a = Float::with_val(arithmetic_config.precision, 1);
        let b = Float::with_val(arithmetic_config.precision, 1);
        let c = Float::with_val(arithmetic_config.precision, 1);
        let mut weights: Vec<Float> = Vec::new();
        weights.push(a);
        weights.push(b);
        weights.push(c);
        let mut counts = [0; 3];
        for _i in 0..n {
            let j = normalized_sample(&weights, &arithmetic_config, &mut rng).unwrap();
            counts[j] += 1;
        }

        arithmetic_config.exit_exact_scope().unwrap();

        let mut probs = [0.0; 3];
        for i in 0..counts.len() {
            probs[i] = (counts[i] as f64) / (n as f64);
            assert!(probs[i] - 0.333 < 0.05);
        }
    }
}
