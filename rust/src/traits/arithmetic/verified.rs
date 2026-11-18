//! Small, deliberately trusted numerical boundary for Verus.
//!
//! Native calls delegate directly to OpenDP's InfDiv and InfLn traits.
//! Only successful results are constrained. Dashu and panic handling are
//! outside the verification target; failures remain ordinary constructor
//! errors. Mathematical exponential/log axioms are documented separately.
#[cfg(verus_only)]
use vstd::prelude::*;
#[cfg(not(verus_only))]
use crate::traits::{InfDiv, InfLn};

#[cfg(verus_only)]
pub type ArithmeticError = ();
#[cfg(not(verus_only))]
pub type ArithmeticError = crate::error::Error;

#[cfg(verus_only)]
verus! {
    /// Mathematical interpretation of a binary64 value.
    /// The relationship with IEEE bit patterns is a primitive numeric axiom.
    pub uninterp spec fn float_value(value: f64) -> real;

    /// Mathematical real transcendental functions (not executable libm).
    pub uninterp spec fn real_ln(value: real) -> real;
    pub uninterp spec fn real_exp(value: real) -> real;

    /// Standard real-analysis facts, marked trusted, not smuggled into a
    /// purported verification of the arithmetic implementation.
    #[verifier::external_body]
    pub proof fn real_exp_monotonic(a: real, b: real)
        requires a <= b,
        ensures real_exp(a) <= real_exp(b),
    { }

    #[verifier::external_body]
    pub proof fn real_exp_ln_inverse(x: real)
        requires x > 0real,
        ensures real_exp(real_ln(x)) == x,
    { }
}

/// Exact conversion of nonnegative integers up to 2^53.
#[cfg_attr(verus_only, verus_verify(external_body))]
#[cfg_attr(verus_only, verus_spec(result =>
    requires value <= 9_007_199_254_740_992u64,
    ensures float_value(result) == value as real,
))]
pub fn exact_u53_cast(value: u64) -> f64 {
    value as f64
}

/// The native f64 InfDiv implementation; trust only the documented
/// upward-rounding property on successful results and positive operands.
#[cfg_attr(verus_only, verus_verify(external_body))]
#[cfg_attr(verus_only, verus_spec(result =>
    requires float_value(denominator) > 0real,
        float_value(numerator) > 0real,
    ensures match result {
        Ok(q) => float_value(q) >= float_value(numerator) / float_value(denominator),
        Err(_) => true,
    },
))]
pub fn inf_div_up_f64(
    numerator: f64,
    denominator: f64,
) -> Result<f64, ArithmeticError> {
    #[cfg(not(verus_only))]
    { numerator.inf_div(&denominator) }
    #[cfg(verus_only)]
    { unreachable!() }
}

/// The native f64 InfLn implementation; trust only upward rounding
/// on successful results and strictly positive inputs.
#[cfg_attr(verus_only, verus_verify(external_body))]
#[cfg_attr(verus_only, verus_spec(result =>
    requires float_value(value) > 0real,
    ensures match result {
        Ok(ln) => float_value(ln) >= real_ln(float_value(value)),
        Err(_) => true,
    },
))]
pub fn inf_ln_up_f64(value: f64) -> Result<f64, ArithmeticError> {
    #[cfg(not(verus_only))]
    { value.inf_ln() }
    #[cfg(verus_only)]
    { unreachable!() }
}

/// Production floating-point log odds, with exactly the same division and
/// logarithm used elsewhere in OpenDP. The Verus harness checks this body
/// using the trusted primitive contracts above.
#[cfg_attr(verus_only, verus_verify(dual_spec, open))]
#[cfg_attr(verus_only, verus_spec(result =>
    requires
        4_503_599_627_370_496u64 <= numerator,
        numerator < 9_007_199_254_740_992u64,
        complement == 9_007_199_254_740_992u64 - numerator,
    ensures match result {
        Ok(epsilon) => {
            &&& real_exp(float_value(epsilon)) >= 1real
            &&& (numerator as real) <= real_exp(float_value(epsilon)) * (complement as real)
        },
        Err(_) => true,
    },
))]
pub fn upper_log_odds_u53(
    numerator: u64,
    complement: u64,
) -> Result<f64, ArithmeticError> {
    let n = exact_u53_cast(numerator);
    let c = exact_u53_cast(complement);
    let quotient = inf_div_up_f64(n, c)?;
    let epsilon = inf_ln_up_f64(quotient)?;
    #[cfg(verus_only)]
    proof! {
        let a = numerator as real;
        let b = complement as real;
        let q = float_value(quotient);
        let e = float_value(epsilon);
        assert(b > 0real);
        assert(a > 0real);
        assert(q >= a / b);
        assert(q > 0real) by(nonlinear_arith) requires a > 0real, b > 0real, q >= a / b;
        real_exp_monotonic(real_ln(q), e);
        real_exp_ln_inverse(q);
        assert(real_exp(e) >= q);
        assert(real_exp(e) * b >= a) by(nonlinear_arith)
            requires b > 0real, q >= a / b, real_exp(e) >= q;
        assert(real_exp(e) >= 1real) by(nonlinear_arith)
            requires a >= b, b > 0real, real_exp(e) * b >= a;
    }
    Ok(epsilon)
}
