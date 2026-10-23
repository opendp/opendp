//! Numerical bounds for binary floating-point addition trees, independent of
//! contribution selection or the algorithm that constructs the tree.

use super::{Float, InfCast};
use crate::error::Fallible;
use dashu::{integer::IBig, rational::RBig};

// Round upward using the existing cast, but require a finite numerical bound.
pub(crate) fn round_up<T: Float + InfCast<RBig>>(bound: RBig) -> Fallible<T> {
    let value = T::inf_cast(bound)?;
    if !value.is_finite() {
        return fallible!(Overflow, "floating-point sum bound is not finite");
    }
    Ok(value)
}

pub(crate) fn sum_magnitude<T: Float>((lower, upper): (T, T)) -> Fallible<T> {
    if !lower.is_finite() || !upper.is_finite() || lower > upper {
        return fallible!(
            MakeTransformation,
            "floating-point sum requires finite ordered bounds"
        );
    }
    Ok(lower.abs().max(upper.abs()))
}

/// Exact rational error bound `E = gamma_h * n * M`, where `n = max_size`,
/// `M = max_magnitude`, `h = max_depth`, `u = epsilon / 2`, and
/// `gamma_h = h*u/(1-h*u)` requires `h*u < 1`.
///
/// Assume IEEE binary arithmetic, round-to-nearest addition, gradual underflow,
/// and no intermediate overflow. Expanding the addition tree multiplies each
/// leaf by at most `h` factors `1 + delta`, with `|delta| <= u`. The standard
/// product-error bound gives
/// `|computed_sum - exact_sum| <= gamma_h * sum_i |x_i| <= E`.
/// Sequential depth is `max(n-1, 0)`; pairwise depth is `ceil(log2(n))`, or zero
/// for `n <= 1`.
///
/// Gradual underflow is harmless for same-format addition: every finite operand
/// is an integer multiple of `q` (`2^-149` for binary32, `2^-1074` for binary64).
/// Its exact sum with another operand is also a multiple of `q`. If subnormal
/// or zero, that sum is exactly representable, so addition introduces zero error.
/// This extends Higham's no-underflow analysis: the relative-error model still
/// holds, without an extra absolute-error term for subnormal results.
///
/// See Nicholas J. Higham (1993), *The Accuracy of Floating Point Summation*,
/// SIAM J. Sci. Comput. 14(4), 783–799, §3, equation (3.6), p. 788
/// ([DOI](https://doi.org/10.1137/0914050),
/// [author's PDF](https://nhigham.com/wp-content/uploads/2023/10/high93s.pdf)).
pub(crate) fn sum_error<T: Float>(
    max_size: usize,
    max_depth: usize,
    max_magnitude: T,
) -> Fallible<RBig> {
    if !max_magnitude.is_finite() || max_magnitude < T::zero() {
        return fallible!(
            MakeTransformation,
            "summand magnitude must be finite and nonnegative"
        );
    }
    if max_size <= 1 || max_magnitude == T::zero() {
        return Ok(RBig::ZERO);
    }
    let u = T::epsilon().into_rational()? / RBig::from(2);
    let hu = RBig::from(IBig::from(max_depth)) * u;
    if hu >= RBig::ONE {
        return fallible!(
            MakeTransformation,
            "summation error bound requires depth * unit_roundoff < 1"
        );
    }
    Ok(&hu / (RBig::ONE - &hu)
        * RBig::from(IBig::from(max_size))
        * max_magnitude.into_rational()?)
}

/// Validate finite accumulation with `n*M + E <= MAX_FINITE` and return exact
/// relaxation `2E`. The depth bound covers the complete reduction, including
/// any combination of partial sums.
///
/// At a first overflowing addition, both child subtrees are overflow-free.
/// Their disjoint leaves and depth bounds give combined prior error at most E.
/// Thus the exact sum of the computed children has magnitude at most `n*M + E`,
/// which is at most `MAX_FINITE`: correctly rounded addition cannot overflow,
/// a contradiction.
///
/// Each neighboring computation has rounding error at most E, so the triangle
/// inequality gives `Delta(d) <= Delta_ideal(d) + 2E`. Keep this relaxation even
/// at symmetric distance zero: permutations can yield different floating sums.
/// E and 2E remain exact rationals; callers round the final stability bound up.
pub(crate) fn sum_relaxation<T: Float>(
    max_size: usize,
    max_depth: usize,
    max_magnitude: T,
) -> Fallible<RBig> {
    let error = sum_error(max_size, max_depth, max_magnitude)?;
    if RBig::from(IBig::from(max_size)) * max_magnitude.into_rational()? + &error
        > T::MAX_FINITE.into_rational()?
    {
        return fallible!(
            MakeTransformation,
            "potential intermediate floating-point sum overflow; tighten bounds or length"
        );
    }
    Ok(error * RBig::from(2))
}
