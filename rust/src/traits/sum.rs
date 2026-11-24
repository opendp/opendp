use super::{Float, InfCast};
use crate::error::Fallible;
use dashu::{integer::IBig, rational::RBig};
use opendp_derive::proven;

pub(crate) fn round_up<T: Float + InfCast<RBig>>(bound: RBig) -> Fallible<T> {
    let value = T::inf_cast(bound)?;
    if !value.is_finite() {
        return fallible!(Overflow, "floating-point sum bound is not finite");
    }
    Ok(value)
}

/// Bound the sum of absolute values of at most size summands in the given interval.
pub(crate) fn sum_abs_bound<T: Float>(size: usize, (lower, upper): (T, T)) -> Fallible<RBig> {
    if !lower.is_finite() || !upper.is_finite() || lower > upper {
        return fallible!(
            MakeTransformation,
            "floating-point sum requires finite ordered bounds"
        );
    }
    Ok(RBig::from(IBig::from(size)) * lower.abs().max(upper.abs()).into_rational()?)
}

/// Bound the Lp rounding error of a vector of sums.
#[proven]
pub(crate) fn sum_error<T: Float>(
    max_depth: usize,
    max_abs_sum: &RBig,
    max_norm: &RBig,
) -> Fallible<RBig> {
    if max_abs_sum < &RBig::ZERO || max_norm < &RBig::ZERO {
        return fallible!(InvalidDistance, "sum magnitude bounds must be nonnegative");
    }
    let max_abs_sum = max_abs_sum.min(max_norm);
    if max_depth == 0 || max_abs_sum == &RBig::ZERO {
        return Ok(RBig::ZERO);
    }

    // Higham (1993), §3: https://doi.org/10.1137/0914050.
    // With gradual underflow, subnormal results of floating-point addition are exact.
    let u = T::epsilon().into_rational()? / RBig::from(2);
    let hu = RBig::from(IBig::from(max_depth)) * u;
    if hu >= RBig::ONE {
        return fallible!(
            MakeTransformation,
            "summation error bound requires depth * unit_roundoff < 1"
        );
    }
    let factor = &hu / (RBig::ONE - &hu);

    // Bound intermediate sums, including rounding error.
    if max_abs_sum * (RBig::ONE + &factor) > T::MAX_FINITE.into_rational()? {
        return fallible!(
            MakeTransformation,
            "potential intermediate floating-point sum overflow"
        );
    }

    Ok(factor * max_norm)
}
