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
    /// Exact real interpretation of a finite IEEE-754 binary64 number.
    /// We never use this value on NaNs or infinities.
    pub open spec fn pow2(exponent: nat) -> real
        decreases exponent
    {
        if exponent == 0 { 1real } else { 2real * pow2((exponent - 1) as nat) }
    }

    pub open spec fn finite(value: f64) -> bool {
        ((f64_to_bits(value) >> 52u64) & 2047u64) != 2047u64
    }

    /// Finite exponent bits cannot encode positive infinity.
    pub proof fn lemma_finite_bits_not_positive_infinity(bits: u64)
        requires ((bits >> 52u64) & 2047u64) != 2047u64,
        ensures bits != 0x7ff0_0000_0000_0000u64,
    {
        assert(bits != 0x7ff0_0000_0000_0000u64) by(bit_vector)
            requires ((bits >> 52u64) & 2047u64) != 2047u64;
    }

    pub closed spec fn float_value(value: f64) -> real {
        let bits = f64_to_bits(value);
        let exponent = ((bits >> 52u64) & 2047u64) as nat;
        let mantissa = (bits & 0x000f_ffff_ffff_ffffu64) as nat;
        let magnitude = if exponent == 0 {
            (mantissa as real) / pow2(1074nat)
        } else if exponent >= 1075 {
            ((4_503_599_627_370_496nat + mantissa) as real)
                * pow2((exponent - 1075nat) as nat)
        } else {
            ((4_503_599_627_370_496nat + mantissa) as real)
                / pow2((1075nat - exponent) as nat)
        };
        if bits == 0u64 { 0real }
        else if (bits >> 63u64) == 0u64 { magnitude }
        else { -magnitude }
    }

    /// The zero bit-pattern denotes exactly mathematical zero.
    pub proof fn lemma_positive_zero_real(value: f64)
        requires f64_to_bits(value) == 0u64,
        ensures float_value(value) == 0real,
    {
    }

    /// Clients of the binary64 interpretation need only bitwise congruence.
    /// Reveal the exact dyadic definition once, rather than in every proof.
    pub proof fn lemma_same_bits_same_value(a: f64, b: f64)
        requires f64_to_bits(a) == f64_to_bits(b),
        ensures float_value(a) == float_value(b),
    {
        reveal(float_value);
    }

    /// Mathematical real transcendental functions (not executable libm).
    pub uninterp spec fn real_ln(value: real) -> real;
    pub uninterp spec fn real_exp(value: real) -> real;

    /// Standard exponential identity for the real mathematical model.
    #[verifier::external_body]
    pub proof fn real_exp_zero()
        ensures real_exp(0real) == 1real,
    { }

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

/// The numerical primitives are trusted, not reimplemented for Verus.
/// Only successful results promise upward rounding; failures are unconstrained.
#[cfg_attr(verus_only, verus_verify(external_body))]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures f64_to_bits(result) == 0x7ff0_0000_0000_0000u64,
))]
pub fn positive_infinity_f64() -> f64 {
    #[cfg(not(verus_only))]
    { f64::INFINITY }
    #[cfg(verus_only)]
    { unimplemented!() }
}

/// The integer-to-binary64 cast is exact through 2^53.
#[cfg_attr(verus_only, verus_verify(external_body))]
#[cfg_attr(verus_only, verus_spec(result =>
    requires value <= 9_007_199_254_740_992u64,
    ensures finite(result), float_value(result) == value as real,
))]
pub fn exact_u53_cast(value: u64) -> f64 {
    #[cfg(not(verus_only))]
    { value as f64 }
    #[cfg(verus_only)]
    { unimplemented!() }
}

#[cfg_attr(verus_only, verus_verify(external_body))]
#[cfg_attr(verus_only, verus_spec(result =>
    requires
        finite(numerator),
        finite(denominator),
        float_value(numerator) > 0real,
        float_value(denominator) > 0real,
    ensures match result {
        Ok(q) => finite(q) && float_value(q) >= float_value(numerator) / float_value(denominator),
        Err(_) => true,
    },
))]
pub fn inf_div_up_f64(
    numerator: f64, denominator: f64,
) -> Result<f64, ArithmeticError> {
    #[cfg(not(verus_only))]
    { numerator.inf_div(&denominator) }
    #[cfg(verus_only)]
    { unimplemented!() }
}

#[cfg_attr(verus_only, verus_verify(external_body))]
#[cfg_attr(verus_only, verus_spec(result =>
    requires finite(value), float_value(value) > 0real,
    ensures match result {
        Ok(ln) => finite(ln) && float_value(ln) >= real_ln(float_value(value)),
        Err(_) => true,
    },
))]
pub fn inf_ln_up_f64(value: f64) -> Result<f64, ArithmeticError> {
    #[cfg(not(verus_only))]
    { value.inf_ln() }
    #[cfg(verus_only)]
    { unimplemented!() }
}

/// Exact native IEEE-754 bit reinterpretation. Verus does not yet
/// understand f64::to_bits as an executable intrinsic, so we trust only
/// this low-level correspondence, not a distribution or privacy property.
#[cfg_attr(verus_only, verus_verify(external_body))]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures result == f64_to_bits(value),
))]
pub fn f64_to_bits_exact(value: f64) -> u64 {
    #[cfg(not(verus_only))]
    { value.to_bits() }
    #[cfg(verus_only)]
    { unimplemented!() }
}
