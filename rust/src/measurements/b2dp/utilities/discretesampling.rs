//! Implements discrete sampling methods in base-2
//! including [`lazy_threshold`](./fn.lazy_threshold.html) and
//! [`sample_within_bounds`](./fn.sample_within_bounds.html).
//! Function names/signatures are not finalized and are
//! likely to be changed.

use crate::error::Fallible;
use rug::{float::Special, ops::Pow, rand::ThreadRandGen, Float};

use super::exactarithmetic::{normalized_sample, ArithmeticConfig};
use super::params::Eta;

/// Returns whether a is an integer multiple of b.
pub fn is_multiple_of(a: &Float, b: &Float) -> bool {
    if a.is_infinite() || b.is_infinite() {
        return false;
    }
    let c = Float::with_val(a.prec(), a).remainder(b);
    if c == 0 {
        let d = Float::with_val(a.prec(), a / b);
        return d.is_integer();
    }
    return false;
}

/// Samples from the Discrete Laplace mechanism at granularity
/// `gamma` within the provided bounds `wmin` and `wmax` where
/// the values `wmin` and `wmax` are sampled with probability
/// equal to the sum of the probabilities of all elements
/// greater than or less than the bounds, respectively.
/// ## Arguments
///  * `eta`: privacy parameter
///  * `gamma`: granularity parameter, must be reciprocal of an integer.
///  * `wmin`: the minimum bound, must be an integer multiple of `gamma`.
///  * `wmax`: the maximum bound, must be an integer multiple of `gamma`.
///  * `arithmeticconfig`: ArithmeticConfig with sufficient precision for
///      sampling.
///  * `rng`: randomness source
///  * `optimize`: whether to optimize sampling.
///
/// ## Returns
/// Returns an integer multiple of `gamma` within the provided bounds
/// sampled according to the discrete laplace mechanism or an error if
/// `eta` cannot be adjusted by `gamma`.
///
/// ## Privacy Budget Usage
/// Uses `eta` privacy budget.  
///
/// ## Exact Arithmetic
/// Does not enforce exact arithmetic, this is the caller's responsibility.
///
/// ## Timing Channels
/// * Method terminates early if the sample is outside of the bounds. This
///   may leak information if the magnitude of the noise should remain
///   secret.
///   TODO: Add an option to execute full sampling logic even if it is
///   not needed to mitigate this timing channel.
/// * Uses [`normalized_sample`](../exactarithmetic/fn.normalized_sample.html#known-timing-channels)
/// which has known timing channels.
/// * Uses [`get_sum`](../discretesampling/fn.get_sum.html), which has a
///    known timing channel.
///
/// ## Example Usage
/// ```
/// # use b2dp::{Eta,GeneratorOpenSSL,
/// #    utilities::exactarithmetic::ArithmeticConfig, sample_within_bounds};
/// # use rug::Float;
/// # use b2dp::errors::*;
/// # fn main() -> Result<()> {
/// // construct eta that can be adjusted for the desired value of gamma.
/// let eta = Eta::new(1,1,2)?;
/// let mut arithmeticconfig = ArithmeticConfig::basic()?;
/// let mut rng = GeneratorOpenDP::default();
/// let gamma = arithmeticconfig.get_float(0.5);
/// let wmin = arithmeticconfig.get_float(-5);
/// let wmax = arithmeticconfig.get_float(5);
/// arithmeticconfig.enter_exact_scope()?;
/// let s = sample_within_bounds(eta, &gamma, &wmin, &wmax,
///                              & mut arithmeticconfig, &mut rng,false)?;
/// let b = arithmeticconfig.exit_exact_scope();
/// assert!(b.is_ok()); // Must check that no inexact arithmetic was performed.
/// # Ok(())
/// # }
/// ```
pub fn sample_within_bounds<R: ThreadRandGen>(
    eta: Eta,
    gamma: &Float,
    wmin: &Float,
    wmax: &Float,
    arithmeticconfig: &mut ArithmeticConfig,
    rng: &mut R,
    optimize: bool,
) -> Fallible<Float> {
    // Check inputs
    if wmax <= wmin {
        return fallible!(FailedFunction, "`wmin` must be strictly less than `wmax`.");
    }
    if !is_multiple_of(&wmin, &gamma) {
        return fallible!(FailedFunction, "`wmin` is not integer multiple of `gamma`.");
    }
    if !is_multiple_of(&wmax, &gamma) {
        return fallible!(FailedFunction, "`wmax` is not integer multiple of `gamma`.");
    }
    let t_min = arithmeticconfig.get_float(wmin / gamma);
    let t_max = arithmeticconfig.get_float(wmax / gamma);

    // Adjust eta
    let gamma_inv = arithmeticconfig.get_float(1 / gamma);
    let eta_prime = adjust_eta(eta, &gamma_inv, arithmeticconfig)?;
    let base = eta_prime.get_base(arithmeticconfig.precision)?;
    let plus_infty = arithmeticconfig.get_float(Special::Infinity);
    let neg_infty = arithmeticconfig.get_float(Special::NegInfinity);

    // Get the weights for each region
    let p_l = get_sum(&base, arithmeticconfig, &neg_infty, &t_min)?;
    let p_u = get_sum(&base, arithmeticconfig, &t_max, &plus_infty)?;
    let p_t = get_sum(&base, arithmeticconfig, &neg_infty, &plus_infty)?;
    let p_m = arithmeticconfig.get_float(p_t - &p_u - &p_l);

    let region_weights: Vec<Float> = vec![p_l, p_u, p_m];

    // Sample which region rho lies in
    let r = normalized_sample(&region_weights, arithmeticconfig, rng, optimize)?;

    match r {
        // lower region
        0 => return Ok(arithmeticconfig.get_float(Special::NegInfinity)),
        // upper region
        1 => return Ok(arithmeticconfig.get_float(Special::Infinity)),
        _ => (), // Must sample from the middle region
    }
    // Sample from the middle region
    // Construct weights and outcome space
    let mut outcomes: Vec<Float> = Vec::new();
    let mut weights: Vec<Float> = Vec::new();
    let mut k = arithmeticconfig.get_float(wmin / gamma);
    k = k + 1; //
    let mut o = arithmeticconfig.get_float(&k * gamma);

    while o < *wmax {
        // record the outcome and the weight
        outcomes.push(o);
        let next_k = arithmeticconfig.get_float(&k + 1);
        let w = arithmeticconfig.get_float((&base).pow(k.abs()));
        weights.push(w);
        // increment k and update the outcome
        k = next_k;
        o = arithmeticconfig.get_float(&k * gamma);
    }

    // Sample
    let s = normalized_sample(&weights, arithmeticconfig, rng, optimize)?;
    return Ok(arithmeticconfig.get_float(&outcomes[s]));
}

/// Adjust the given eta to account for granularity 1/gamma_inv.
///
/// ## Arguments
/// * `eta`: privacy parameter
/// * `gamma_inv`: an integer-valued Float representing the inverse of `gamma`
/// * `arithmeticconfig`: an ArithmeticConfig with sufficient precision to
///       adjust `eta` if it can be adjusted.
///
/// ## Returns
/// A new `Eta` adjusted such that `2^eta_prime = (2^eta)^gamma` or an error
/// if `eta` cannot be adjusted.
///
/// ## Exact Arithmetic
/// Does not enforce exact arithmetic, this is the caller's responsibility.
pub fn adjust_eta(
    eta: Eta,
    gamma_inv: &Float,
    arithmeticconfig: &mut ArithmeticConfig,
) -> Fallible<Eta>
{
    // Check that gamma is valid for the given eta
    if !gamma_inv.is_integer() {
        return fallible!(FailedFunction, "`gamma_inv` must be an integer.");
    }
    let gamma = arithmeticconfig.get_float(1.0 / gamma_inv);
    // Check if eta.z is divisible by gamma_inv.
    let mut z_prime = eta.z;
    let mut x_prime = eta.x;
    let mut y_prime = eta.y;
    if arithmeticconfig.get_float(eta.z * &gamma).is_integer() {
        let rootz = arithmeticconfig.get_float(eta.z * &gamma);

        z_prime = rootz.to_integer().unwrap().to_u32().unwrap();
        // Leave x and y as is
    } else {
        // Leave z as is
        // Check if x and y meet the critera
        let fx = arithmeticconfig.get_float(eta.x);
        let fy = arithmeticconfig.get_float(eta.y);

        if !is_multiple_of(&fy, &gamma) {
            return fallible!(FailedFunction, "Unable to adjust for gamma (y).");
        }
        let rooty = arithmeticconfig.get_float(fy * &gamma);

        let rootx = arithmeticconfig.get_float(fx.pow(&gamma));
        if !rootx.is_integer() {
            return fallible!(FailedFunction, "Unable to adjust for gamma (x).");
        }

        // TODO: more elegant error handling
        x_prime = rootx.to_integer().unwrap().to_u32().unwrap();
        y_prime = rooty.to_integer().unwrap().to_u32().unwrap();
    }
    let eta_prime = Eta::new(x_prime, y_prime, z_prime)?;
    return Ok(eta_prime);
}

/// Determines whether the discretized Laplace exceeds the given
/// threshold conditioned on it exceeding the given conditional threshold.
/// ## Arguments
///   * `eta`: the privacy parameter
///   * `arithmeticconfig`: ArithmeticConfig with sufficient precision
///   * `gamma`: granularity parameter
///   * `threshold`: the threshold value
///   * `cond_threshold`: the conditional threshold value already exceeded
///      (Must be smaller than `threshold`)
///   * `rng`: randomness source
///   * `optimize`: whether to optimize sampling, exacerbates timing channels
///
/// ## Returns
/// Returns a `Float` with value `Special::Infinity` if greater than or equal
/// to the threshold, otherwise returns with value `Special::NegInfinity`.
/// Returns an error if `eta` cannot be appropriately adjusted or sum
/// computation fails.
///
/// ## Exact Arithmetic
/// Does not explicitly enforce exact arithmetic, this is the caller's
/// responsibility.
///
/// ## Privacy Budget
/// Uses `eta` privacy budget. Note that if multiple calls of conditional threshold are made
/// in a chain, i.e., cond_threshold(T_0,T_1); cond_threshold(T_1,T_2) then the
/// privacy budget may be shared among these calls. This accounting must be
/// used with caution.
///
/// ## Timing Channels
///  * Uses [`normalized_sample`](../exactarithmetic/fn.normalized_sample.html#known-timing-channels)
/// which has known timing channels if the total weight is not the same
/// between calls to `normalized_sample`. (For most invocations of sparse vector, this should not be the case.)
///  * Uses [`get_sum`](../discretesampling/fn.get_sum.html), which has a known timing channel.
///
/// ## Example Usage
/// ```
/// # use b2dp::{Eta,GeneratorOpenSSL,
/// #            utilities::exactarithmetic::ArithmeticConfig,
/// #            conditional_lazy_threshold};
/// # use rug::Float;
/// # use b2dp::errors::*;
/// # fn main() -> Result<()> {
/// // construct eta that can be adjusted for the desired value of gamma.
/// let eta = Eta::new(1,1,2)?;
/// let mut arithmeticconfig = ArithmeticConfig::basic()?;
/// let mut rng = GeneratorOpenDP::default();
/// let gamma_inv = arithmeticconfig.get_float(2);
/// let cond_threshold = arithmeticconfig.get_float(0);
/// let threshold = arithmeticconfig.get_float(1);
/// arithmeticconfig.enter_exact_scope()?;
/// let s = conditional_lazy_threshold(eta, & mut arithmeticconfig,
///                                     &gamma_inv, &threshold,
///                                     &cond_threshold, &mut rng, false)?;
/// assert!(!s.is_finite()); // returns plus or minus infinity
/// if s.is_sign_positive() { /* Greater than the threshold */ ;}
/// else { /* Less than the threshold. */ ;}
/// let b = arithmeticconfig.exit_exact_scope();
/// assert!(b.is_ok()); // Must check that no inexact arithmetic was performed.
/// # Ok(())
/// # }
/// ```
pub fn conditional_lazy_threshold<R: ThreadRandGen>(
    eta: Eta,
    arithmeticconfig: &mut ArithmeticConfig,
    gamma_inv: &Float,
    threshold: &Float,
    cond_threshold: &Float,
    rng: &mut R,
    optimize: bool,
) -> Fallible<Float> {
    // plus and minus infinity
    let plus_infty = arithmeticconfig.get_float(Special::Infinity);
    let neg_infty = arithmeticconfig.get_float(Special::NegInfinity);

    // Adjust eta to take gamma into account.
    let eta_prime = adjust_eta(eta, gamma_inv, arithmeticconfig)?;
    // get the base for the adjusted eta
    let base = eta_prime.get_base(arithmeticconfig.precision)?;

    // Check that gamma is valid (integer reciprocal)
    if !gamma_inv.is_integer() {
        return fallible!(FailedFunction, "`gamma_inv` must be an integer.");
    }
    let gamma = arithmeticconfig.get_float(1 / gamma_inv);

    // Check that cond_threshold is less than threshold
    if cond_threshold > threshold {
        return fallible!(FailedFunction, "conditional threshold must be smaller than threshold.");
    } else if cond_threshold == threshold {
        return Ok(plus_infty); // The value already equals the threshold
    }
    // Check that thresholds are integer multiples of gamma
    if !is_multiple_of(&threshold, &gamma) {
        return fallible!(FailedFunction, "`threshold` must be integer multiple of `gamma`.");
    }
    if !is_multiple_of(&cond_threshold, &gamma) {
        return fallible!(FailedFunction, "`cond_threshold` must be integer multiple of `gamma`.");
    }

    // Convert to integer multiples
    let t = arithmeticconfig.get_float(threshold * gamma_inv);
    let ct = arithmeticconfig.get_float(cond_threshold * gamma_inv);

    // Sum of weights above the threshold
    let p_top = get_sum(&base, arithmeticconfig, &t, &plus_infty)?;
    // Sum of weights between the conditional threshold and positive infinity
    let p_total = get_sum(&base, arithmeticconfig, &ct, &plus_infty)?;
    let p_bot = arithmeticconfig.get_float(&p_total - &p_top);
    let weights: Vec<Float> = vec![p_top, p_bot];
    let s = normalized_sample(&weights, arithmeticconfig, rng, optimize)?;

    if s == 0 {
        return Ok(plus_infty);
    } else {
        return Ok(neg_infty);
    }
}

/// Determines whether the discretized Laplace exceeds the given threshold.
/// ## Arguments
///   * `eta`: the privacy parameter
///   * `arithmeticconfig`: ArithmeticConfig with sufficient precision
///   * `gamma`: granularity parameter
///   * `threshold`: the threshold value
///   * `rng`: randomness source
///   * `optimize`: whether to optimize sampling, exacerbates timing channels
///
/// ## Returns
/// Returns a `Float` with value `Special::Infinity` if draw from the discrete
/// Laplace is greater than or equal to the threshold,
/// otherwise returns with value `Special::NegInfinity`.
///
/// Returns an error if `eta` cannot be appropriately adjusted or sum
/// computation fails.
///
/// ## Exact Arithmetic
/// Does not explicitly enforce exact arithmetic, this is the caller's
/// responsibility.
///
/// ## Privacy Budget
/// Uses `eta` privacy budget
///
/// ## Timing Channels
///  * Uses [`normalized_sample`](../exactarithmetic/fn.normalized_sample.html#known-timing-channels)
/// which has known timing channels if the total weight is not the same
/// between calls to `normalized_sample`. (For most invocations of sparse vector, this should not be the case.)
///  * Uses [`get_sum`](../discretesampling/fn.get_sum.html), which has a known timing channel.
///
/// ## Example Usage
/// ```
/// # use b2dp::{Eta,GeneratorOpenSSL,
/// # utilities::exactarithmetic::ArithmeticConfig, lazy_threshold};
/// # use rug::Float;
/// # use b2dp::errors::*;
/// # fn main() -> Result<()> {
/// // construct eta that can be adjusted for the desired value of gamma.
/// let eta = Eta::new(1,1,2)?;
/// let mut arithmeticconfig = ArithmeticConfig::basic()?;
/// let mut rng = GeneratorOpenDP::default();
/// let gamma_inv = arithmeticconfig.get_float(2);
/// let threshold = arithmeticconfig.get_float(0);
/// arithmeticconfig.enter_exact_scope()?;
/// let s = lazy_threshold(eta, & mut arithmeticconfig,
///                         &gamma_inv, &threshold, &mut rng, false)?;
/// assert!(!s.is_finite()); // returns plus or minus infinity
/// if s.is_sign_positive() { /* Greater than the threshold */ ;}
/// else { /* Less than the threshold. */ ;}
/// let b = arithmeticconfig.exit_exact_scope();
/// assert!(b.is_ok()); // Must check that no inexact arithmetic was performed.
/// # Ok(())
/// # }
/// ```
pub fn lazy_threshold<R: ThreadRandGen>(
    eta: Eta,
    arithmeticconfig: &mut ArithmeticConfig,
    gamma_inv: &Float,
    threshold: &Float,
    rng: &mut R,
    optimize: bool,
) -> Fallible<Float> {
    // plus and minus infinity
    let plus_infty = arithmeticconfig.get_float(Special::Infinity);
    let neg_infty = arithmeticconfig.get_float(Special::NegInfinity);

    // Adjust eta to take gamma into account.
    let eta_prime = adjust_eta(eta, &gamma_inv, arithmeticconfig)?;
    // get the base for the adjusted eta
    let base = eta_prime.get_base(arithmeticconfig.precision)?;

    // Check that gamma is valid (integer reciprocal)
    if !gamma_inv.is_integer() {
        return fallible!(FailedFunction, "`gamma_inv` must be an integer.");
    }
    let gamma = arithmeticconfig.get_float(1 / gamma_inv);

    // Check that threshold is integer multiple of gamma
    if !is_multiple_of(&threshold, &gamma) {
        return fallible!(FailedFunction, "`threshold` must be integer multiple of `gamma`.");
    }

    // Convert to integer multiple
    let t = arithmeticconfig.get_float(threshold * gamma_inv);

    let p_top = get_sum(&base, arithmeticconfig, &t, &plus_infty)?;
    let p_total = get_sum(&base, arithmeticconfig, &neg_infty, &plus_infty)?;
    let p_bot = arithmeticconfig.get_float(p_total - &p_top);
    let weights: Vec<Float> = vec![p_top, p_bot];
    let s = normalized_sample(&weights, arithmeticconfig, rng, optimize)?;

    if s == 0 {
        return Ok(plus_infty);
    } else {
        return Ok(neg_infty);
    }
}

/// Returns the sum:
/// (1-base)*\sum_{k=start}^{end}base^{|k|}
///
/// ## Arguments:
///   * `base`: a `Float` indicating the base for the sum
///   * `arithmeticconfig`: an ArithmeticConfig  
///   * `start`: an integer-valued or infinite `Float` indicating the starting
///              point of the sum
///   * `end`: an integer-valued or infinite `Float` indicating the ending
///            point of the sum
///
/// ## Returns
/// Either the sum or an error if parameters are mis-specified. This method
/// does not explicitly check for inexact arithmetic, it is the caller's
/// responsibility to do so.
///
/// ## Exact Arithmetic
/// This method does not enforce exact arithmetic, this is the caller's
/// responsibility.
///
/// ## Timing channels
/// **Known Timing Channel:** The recursive calls to `get_sum` introduce a
/// timing channel distinguishing whether the sum has an infinite start or
/// end point versus finite start and end points. In most settings, adjacent
/// databases should not result in finite vs infinite endpoint differences.
/// Slight timing variation due to logic between different types of infinite
/// endpoints may be noticeable (please see benchmarks.)
pub fn get_sum(
    base: &Float,
    arithmeticconfig: &ArithmeticConfig,
    start: &Float,
    end: &Float,
) -> Fallible<Float> {
    // Check ordering
    if start >= end {
        return fallible!(FailedFunction, "`start` must be strictly less than `end`.");
    }
    // Check integrality
    if (!start.is_integer() && start.is_finite()) || // infinite values are not 
        (!end.is_integer() && end.is_finite())
    {
        // considered integers
        return fallible!(FailedFunction, "`start` and `end` must be integer values or infinite.");
    }

    // Check base magnitude
    if *base >= 1 {
        return fallible!(FailedFunction, "`base` must be less than 1.");
    }

    // Sum components
    let abs_end = arithmeticconfig.get_float(end).abs();
    let abs_start = arithmeticconfig.get_float(start).abs();
    let end_plus_one = arithmeticconfig.get_float(&abs_end + 1);
    let start_plus_one = arithmeticconfig.get_float(&abs_start + 1);
    let pow_end_plus_one = arithmeticconfig.get_float(base).pow(end_plus_one);
    let pow_start_plus_one = arithmeticconfig.get_float(base).pow(start_plus_one);
    // base^(|end|)
    let pow_end = arithmeticconfig.get_float(base).pow(abs_end);
    // base^(|start|)
    let pow_start = arithmeticconfig.get_float(base).pow(abs_start);
    let base_plus_one = arithmeticconfig.get_float(1 + base);

    // Check for negative infinity case
    if start.is_infinite() && start.is_sign_negative() {
        if end.is_infinite() && end.is_sign_positive() {
            // Full infinite sum
            // = (1+base)
            return Ok(base_plus_one);
        }

        // Half-open infinite sum [-infinity, end]
        if *end < 0 {
            // = base^(|end|)
            let s = arithmeticconfig.get_float(pow_end);
            return Ok(s);
        } else {
            // = 1  + base - base^(|end| + 1)
            let s = arithmeticconfig.get_float(base_plus_one - pow_end_plus_one);
            return Ok(s);
        }
    }
    // Half-open positive infinite sum
    else if end.is_infinite() && end.is_sign_positive() {
        // Half-open infinite sum [start, +infinity]
        if *start > 0 {
            let s = arithmeticconfig.get_float(pow_start);
            return Ok(s);
        } else {
            let s = arithmeticconfig.get_float(base_plus_one - pow_start_plus_one);
            return Ok(s);
        }
    }

    // Otherwise, finite sum, recurse
    // = get_sum(-infinity,infinity) - get_sum(-infinity, start - 1)
    //   - get_sum(end + 1, infinity)
    let plus_infty = arithmeticconfig.get_float(Special::Infinity);
    let neg_infty = arithmeticconfig.get_float(Special::NegInfinity);

    let total_sum = get_sum(base, arithmeticconfig, &neg_infty, &plus_infty)?;
    let neg_sum = get_sum(
        base,
        arithmeticconfig,
        &neg_infty,
        &arithmeticconfig.get_float(start - 1),
    )?;
    let pos_sum = get_sum(
        base,
        arithmeticconfig,
        &arithmeticconfig.get_float(end + 1),
        &plus_infty,
    )?;

    let s = arithmeticconfig.get_float(total_sum - neg_sum - pos_sum);
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::samplers::GeneratorOpenDP;

    #[test]
    fn test_sample_within_bounds() {
        let eta = Eta::new(1, 1, 2).unwrap();
        let mut rng = GeneratorOpenDP::default();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let gamma = arithmeticconfig.get_float(0.5);
        let gamma_inv = arithmeticconfig.get_float(1 / &gamma);
        let eta_prime = adjust_eta(eta, &gamma_inv, &mut arithmeticconfig);
        assert!(eta_prime.is_ok());
        let wmin = arithmeticconfig.get_float(-5);
        let wmax = arithmeticconfig.get_float(5);
        let _a = arithmeticconfig.enter_exact_scope();
        let s = sample_within_bounds(eta, &gamma, &wmin, &wmax, &mut arithmeticconfig, &mut rng, false);

        assert!(s.is_ok());
        let b = arithmeticconfig.exit_exact_scope();
        assert!(b.is_ok());
    }
    #[test]
    fn test_sample_within_bounds_inf_likely() {
        let eta = Eta::new(1, 1, 2).unwrap();
        let mut rng = GeneratorOpenDP::default();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let gamma = arithmeticconfig.get_float(0.5);
        let gamma_inv = arithmeticconfig.get_float(1 / &gamma);
        let eta_prime = adjust_eta(eta, &gamma_inv, &mut arithmeticconfig);
        assert!(eta_prime.is_ok());
        let wmin = arithmeticconfig.get_float(-1);
        let wmax = arithmeticconfig.get_float(1);
        let _a = arithmeticconfig.enter_exact_scope();
        let s = sample_within_bounds(eta, &gamma, &wmin, &wmax, &mut arithmeticconfig, &mut rng, false);

        assert!(s.is_ok());
        let b = arithmeticconfig.exit_exact_scope();
        assert!(b.is_ok());
    }
    #[test]
    fn test_remainder() {
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let gamma = arithmeticconfig.get_float(0.25);
        let t = arithmeticconfig.get_float(0.75);
        let _a = arithmeticconfig.enter_exact_scope();
        let r = t.remainder(&gamma);
        println!("{:?}", &r);
        let b = arithmeticconfig.exit_exact_scope();
        assert!(b.is_ok());
    }

    #[test]
    fn test_cond_lazy_threshold() {
        // Equivalent threshold test
        let eta = Eta::new(1, 1, 2).unwrap();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let mut rng = GeneratorOpenDP::default();
        let gamma_inv = arithmeticconfig.get_float(2);
        let threshold = arithmeticconfig.get_float(0);
        let cond_threshold = arithmeticconfig.get_float(0);
        let _a = arithmeticconfig.enter_exact_scope();
        let s = conditional_lazy_threshold(
            eta,
            &mut arithmeticconfig,
            &gamma_inv,
            &threshold,
            &cond_threshold,
            &mut rng,
            false,
        )
        .unwrap();
        assert!(!s.is_finite()); // should get plus or minus infinity
        let b = arithmeticconfig.exit_exact_scope();
        assert!(b.is_ok());

        // Check fail on cond threshold that is not a multiple of gamma
        let eta = Eta::new(1, 1, 2).unwrap();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let mut rng = GeneratorOpenDP::default();
        let gamma_inv = arithmeticconfig.get_float(2);
        let cond_threshold = arithmeticconfig.get_float(0.3);
        let threshold = arithmeticconfig.get_float(1);
        let _a = arithmeticconfig.enter_exact_scope();
        let s = conditional_lazy_threshold(
            eta,
            &mut arithmeticconfig,
            &gamma_inv,
            &threshold,
            &cond_threshold,
            &mut rng,
            false,
        );
        assert!(s.is_err());
        let b = arithmeticconfig.exit_exact_scope();
        assert!(b.is_ok());

        // Check fail on threshold < cond_threshold
        let eta = Eta::new(1, 1, 2).unwrap();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let mut rng = GeneratorOpenDP::default();
        let gamma_inv = arithmeticconfig.get_float(2);
        let threshold = arithmeticconfig.get_float(0);
        let cond_threshold = arithmeticconfig.get_float(1);
        let _a = arithmeticconfig.enter_exact_scope();
        let s = conditional_lazy_threshold(
            eta,
            &mut arithmeticconfig,
            &gamma_inv,
            &threshold,
            &cond_threshold,
            &mut rng,
            false,
        );
        assert!(s.is_err());
        let _b = arithmeticconfig.exit_exact_scope();
    }

    #[test]
    fn test_lazy_threshold() {
        // Simple zero threshold test
        let eta = Eta::new(1, 1, 2).unwrap();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let mut rng = GeneratorOpenDP::default();
        let gamma_inv = arithmeticconfig.get_float(2);
        let threshold = arithmeticconfig.get_float(0);
        let _a = arithmeticconfig.enter_exact_scope();
        let s = lazy_threshold(
            eta,
            &mut arithmeticconfig,
            &gamma_inv,
            &threshold,
            &mut rng,
            false,
        )
        .unwrap();
        assert!(!s.is_finite()); // should get plus or minus infinity
        let b = arithmeticconfig.exit_exact_scope();
        assert!(b.is_ok());

        // Check fail on threshold that is not a multiple of gamma
        let eta = Eta::new(1, 1, 2).unwrap();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let mut rng = GeneratorOpenDP::default();
        let gamma_inv = arithmeticconfig.get_float(2);
        let threshold = arithmeticconfig.get_float(0.3);
        let _a = arithmeticconfig.enter_exact_scope();
        let s = lazy_threshold(
            eta,
            &mut arithmeticconfig,
            &gamma_inv,
            &threshold,
            &mut rng,
            false,
        );
        assert!(s.is_err());
        let b = arithmeticconfig.exit_exact_scope();
        assert!(b.is_ok());

        // Check fail on eta that cannot be adjusted
        let eta = Eta::new(1, 1, 1).unwrap();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let mut rng = GeneratorOpenDP::default();
        let gamma_inv = arithmeticconfig.get_float(2);
        let threshold = arithmeticconfig.get_float(0.3);
        let _a = arithmeticconfig.enter_exact_scope();
        let s = lazy_threshold(
            eta,
            &mut arithmeticconfig,
            &gamma_inv,
            &threshold,
            &mut rng,
            false,
        );
        assert!(s.is_err());
        let _b = arithmeticconfig.exit_exact_scope();
    }

    // Test eta adjustment
    #[test]
    fn test_eta_adjustment() {
        // Test case passes by modifying z
        let eta = Eta::new(1, 1, 2).unwrap();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let gamma_inv = arithmeticconfig.get_float(2);
        let eta_prime = adjust_eta(eta, &gamma_inv, &mut arithmeticconfig).unwrap();
        assert_eq!(eta_prime, Eta::new(1, 1, 1).unwrap());

        // Test case passes by modifying x and y
        let eta = Eta::new(1, 2, 1).unwrap();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let gamma_inv = arithmeticconfig.get_float(2);
        let eta_prime = adjust_eta(eta, &gamma_inv, &mut arithmeticconfig).unwrap();
        assert_eq!(eta_prime, Eta::new(1, 1, 1).unwrap());

        // Cannot be adjusted
        let eta = Eta::new(1, 1, 1).unwrap();
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let gamma_inv = arithmeticconfig.get_float(2);
        let eta_prime = adjust_eta(eta, &gamma_inv, &mut arithmeticconfig);
        assert!(eta_prime.is_err());
    }

    // Test sum parameter errors
    #[test]
    fn test_sum_parameters() {
        let arithmeticconfig = ArithmeticConfig::basic().unwrap();

        // base that is too large
        let base = arithmeticconfig.get_float(1.0);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, 0),
            &Float::with_val(32, 5),
        );
        assert!(s.is_err());

        // start > end
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, 5),
            &Float::with_val(32, 0),
        );
        assert!(s.is_err());

        // start = end
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, 5),
            &Float::with_val(32, 5),
        );
        assert!(s.is_err());

        // infinite and equal but different precision
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, Special::NegInfinity),
            &Float::with_val(16, Special::NegInfinity),
        );
        assert!(s.is_err());

        // non-integer
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, Special::NegInfinity),
            &Float::with_val(16, 1.75),
        );
        assert!(s.is_err());
    }

    // Test Sum computation
    #[test]
    fn test_sums() {
        let arithmeticconfig = ArithmeticConfig::basic().unwrap();
        // base = 0.5
        // Complete infinite sum
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, Special::NegInfinity),
            &Float::with_val(32, Special::Infinity),
        )
        .unwrap();
        assert_eq!(s, 1.5); // 1 + Base

        // [0,infinity]
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, 0),
            &Float::with_val(32, Special::Infinity),
        )
        .unwrap();
        assert_eq!(s, 1.0);

        // [-infinity, 0]
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, Special::NegInfinity),
            &Float::with_val(32, 0),
        )
        .unwrap();
        assert_eq!(s, 1.0);

        // [1,infinity]
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, 1),
            &Float::with_val(32, Special::Infinity),
        )
        .unwrap();
        assert_eq!(s, 0.5);

        // [-1,infinity]
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, -1),
            &Float::with_val(32, Special::Infinity),
        )
        .unwrap();
        assert_eq!(s, 1.25);

        // [-infinity,-1]
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, Special::NegInfinity),
            &Float::with_val(32, -1),
        )
        .unwrap();
        assert_eq!(s, 0.5);

        // [-infinity,1]
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, Special::NegInfinity),
            &Float::with_val(32, 1),
        )
        .unwrap();
        assert_eq!(s, 1.25);

        // [1,5]
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, 1),
            &Float::with_val(32, 5),
        )
        .unwrap();
        assert_eq!(s, 0.484375);

        // [-5,5]
        let base = arithmeticconfig.get_float(0.5);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, -5),
            &Float::with_val(32, 5),
        )
        .unwrap();
        assert_eq!(s, 1.46875);

        // base = 0.1
        // Complete infinite sum
        let base = arithmeticconfig.get_float(0.1);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, Special::NegInfinity),
            &Float::with_val(32, Special::Infinity),
        )
        .unwrap();
        assert_eq!(s, 1.1);

        // [0,infinity]
        let base = arithmeticconfig.get_float(0.1);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, 0),
            &Float::with_val(32, Special::Infinity),
        )
        .unwrap();
        assert_eq!(s, 1.0);

        // [-infinity, 0]
        let base = arithmeticconfig.get_float(0.1);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, Special::NegInfinity),
            &Float::with_val(32, 0),
        )
        .unwrap();
        assert_eq!(s, 1.0);

        // [1,infinity]
        let base = arithmeticconfig.get_float(0.1);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, 1),
            &Float::with_val(32, Special::Infinity),
        )
        .unwrap();
        assert_eq!(s, 0.1); // result not exact, test case only

        // [-1,infinity]
        let base = arithmeticconfig.get_float(0.1);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, -1),
            &Float::with_val(32, Special::Infinity),
        )
        .unwrap();
        assert_eq!(s, 1.09);

        // [-infinity,-1]
        let base = arithmeticconfig.get_float(0.1);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, Special::NegInfinity),
            &Float::with_val(32, -1),
        )
        .unwrap();
        assert_eq!(s, 0.1); // result not exact, test case only

        // [-infinity,1]
        let base = arithmeticconfig.get_float(0.1);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, Special::NegInfinity),
            &Float::with_val(32, 1),
        )
        .unwrap();
        assert_eq!(s, 1.09);

        // [1,5]
        let base = arithmeticconfig.get_float(0.1);
        let s = get_sum(
            &base,
            &arithmeticconfig,
            &Float::with_val(32, 1),
            &Float::with_val(32, 5),
        )
        .unwrap();
        assert!(s.to_f64() - 0.099999 < 0.001); // result not exact, test case only
    }

    // Basic test of special infinity values
    #[test]
    fn test_infinity() {
        let mut arithmeticconfig = ArithmeticConfig::basic().unwrap();
        let a = arithmeticconfig.enter_exact_scope();
        assert!(a.is_ok());
        let f = arithmeticconfig.get_float(Special::Infinity);
        assert!(f.is_infinite());
        assert!(f.is_sign_positive());
        let g = arithmeticconfig.get_float(Special::NegInfinity);
        assert!(g.is_infinite());
        assert!(g.is_sign_negative());
        let b = arithmeticconfig.exit_exact_scope();
        assert!(b.is_ok());
        assert!(!g.is_integer()); // infinite values are not considered integers
    }
}
