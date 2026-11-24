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
///
/// Let a_j be the exact sum of the absolute values of the summands in coordinate j.
/// The norm bounds apply to a, not to the vector of final sums.
///
/// # Arguments
/// * `max_depth` - Upper bound on summation tree depth.
/// * `l_inf_bound` - Upper bound on the L-infinity norm of a.
/// * `lp_bound` - Upper bound on the Lp norm of a (p >= 1).
#[proven]
pub(crate) fn sum_error<T: Float>(
    max_depth: usize,
    l_inf_bound: &RBig,
    lp_bound: &RBig,
) -> Fallible<RBig> {
    if l_inf_bound < &RBig::ZERO || lp_bound < &RBig::ZERO {
        return fallible!(InvalidDistance, "sum magnitude bounds must be nonnegative");
    }
    let l_inf_bound = l_inf_bound.min(lp_bound);
    if max_depth == 0 || l_inf_bound == &RBig::ZERO {
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
    if l_inf_bound * (RBig::ONE + &factor) > T::MAX_FINITE.into_rational()? {
        return fallible!(
            MakeTransformation,
            "potential intermediate floating-point sum overflow"
        );
    }

    Ok(factor * lp_bound)
}

#[cfg(any(feature = "ffi", test))]
/// Bound the Lp sensitivity of floating-point sums, including rounding error.
/// Assumes finite summands and round-to-nearest addition with gradual underflow.
///
/// Let a_j be the exact sum of the absolute values of the summands in coordinate j,
/// on either input. The norm bounds apply to a, not to the vector of final sums.
///
/// # Arguments
/// * `sensitivity` - Lp sensitivity in exact arithmetic (p >= 1).
/// * `max_depth` - Upper bound on summation tree depth.
/// * `l_inf_bound` - Upper bound on the L-infinity norm of a.
/// * `lp_bound` - Upper bound on the Lp norm of a.
/// # Generics
/// * `T` - Summation type (f32 or f64).
#[proven]
pub(crate) fn float_sum_sensitivity<T: Float>(
    sensitivity: f64,
    max_depth: usize,
    l_inf_bound: f64,
    lp_bound: f64,
) -> Fallible<f64> {
    use crate::traits::CastInternalRational;

    if !sensitivity.is_finite() || sensitivity < 0.0 {
        return fallible!(
            InvalidDistance,
            "sensitivity must be finite and nonnegative"
        );
    }
    let error = sum_error::<T>(
        max_depth,
        &l_inf_bound.into_rational()?,
        &lp_bound.into_rational()?,
    )?;
    round_up(sensitivity.into_rational()? + error * RBig::from(2))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_float_sum_sensitivity() -> Fallible<()> {
        use crate::traits::CastInternalRational;
        use dashu::rational::RBig;

        for (epsilon, actual) in [
            (
                f64::from(f32::EPSILON),
                float_sum_sensitivity::<f32>(1., 4, 16., 32.)?,
            ),
            (f64::EPSILON, float_sum_sensitivity::<f64>(1., 4, 16., 32.)?),
        ] {
            let hu = epsilon.into_rational()? * RBig::from(2);
            let expected = RBig::ONE + &hu / (RBig::ONE - &hu) * RBig::from(64);
            assert_eq!(actual, f64::inf_cast(expected)?);
        }
        Ok(())
    }
}
