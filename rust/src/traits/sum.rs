use super::{Float, InfCast};
use crate::error::Fallible;
use dashu::{integer::IBig, rational::RBig};

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

pub(crate) fn sum_error<T: Float>(
    max_size: usize,
    max_depth: usize,
    max_magnitude: T,
) -> Fallible<RBig> {
    // Higham (1993), §3: https://doi.org/10.1137/0914050.
    // With gradual underflow, subnormal results of floating-point addition are exact.
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

pub(crate) fn sum_relaxation<T: Float>(
    max_size: usize,
    max_depth: usize,
    max_magnitude: T,
) -> Fallible<RBig> {
    let error = sum_error(max_size, max_depth, max_magnitude)?;
    // At the first overflowing addition, the child sums have total error at most E.
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
