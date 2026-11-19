use num::{One, Zero};

use crate::{
    error::Fallible,
    traits::{ExactIntCast, FiniteBounds, Float, Integer},
};

use super::{bernoulli_u64_threshold, sample_bernoulli_u64, sample_standard_bernoulli};

#[cfg(test)]
mod test;

/// Sample from the censored two-sided geometric distribution.
///
/// # Proof Definition
/// The lower bound must not be greater than the upper bound.
///
/// Return a sample from the censored two-sided geometric distribution with parameter `scale`.
/// The support is truncated to:
/// ```text
///     [Self::MIN, Self::MAX] ∩ {shift ± {1, 2, 3, ..., trials}}
/// ```
///
/// Tail probabilities accumulate at the extrema of the support.
///
/// # Arguments
/// * `shift` - Parameter to shift the output by
/// * `scale` - Parameter to scale the output by
/// * `bounds` - Run the algorithm with both inputs and outputs clamped to this value.
///
/// # Return
/// A draw from the two-sided censored geometric distribution defined above.
///
/// # Example
/// ```
/// use opendp::traits::samplers::sample_discrete_laplace_linear;
/// let geom = sample_discrete_laplace_linear(0, 0.1, (20, 30));
/// # geom.unwrap();
/// ```
///
/// Samples are taken from the censored two-sided geometric distribution,
///     where the tail probabilities are accumulated in the +/- (upper - lower)th bucket from taking (upper - lower - 1) bernoulli trials.
///     This special bucket may at most appear at the clamping bound of the output distribution-
///     Should the shift be outside the bounds, this irregular bucket and its zero-neighbor bucket would both be present in the output.
///     There is no multiplicative bound on the difference in probabilities between the output probabilities for neighboring datasets.
///     Therefore the input must be clamped. In addition, the noised output must be clamped as well--
///         if the greatest magnitude noise GMN = (upper - lower), then should (upper + GMN) be released,
///             the analyst can deduce that the input was greater than or equal to upper
pub fn sample_discrete_laplace_linear<T, P>(
    mut shift: T,
    scale: P,
    (lower, upper): (T, T),
) -> Fallible<T>
where
    T: Integer,
    P: Float,
    usize: ExactIntCast<T>,
{
    if !scale.is_finite() || scale < P::zero() {
        return fallible!(FailedFunction, "scale must be nonnegative and finite");
    }
    if scale.is_zero() {
        return shift.total_clamp(lower, upper);
    }
    // if the output interval is a point
    if lower == upper {
        return Ok(lower);
    }

    let trials = usize::exact_int_cast(upper.alerting_sub(&lower)?.alerting_sub(&T::one())?)?;

    // Round the failure probability up to the u64 grid, so geometric noise
    // is never smaller than specified by scale.
    let inverse = P::one().neg_inf_div(&scale)?;
    let tail_prob = (-inverse).inf_exp()?;
    let tail_threshold = bernoulli_u64_threshold(
        tail_prob
            .to_f64()
            .ok_or_else(|| err!(FailedCast, "failed to cast tail probability"))?,
    );
    if tail_threshold == u64::MAX {
        return fallible!(
            FailedFunction,
            "scale is too large for fixed-width geometric sampling"
        );
    }

    // It should be possible to drop the input clamp at a cost of `delta = 2^(-(upper - lower))`.
    // Thanks for the input @ctcovington (Christian Covington)
    shift = shift.total_clamp(lower, upper)?;

    let noised = loop {
        let direction = sample_standard_bernoulli()?;
        let bound = if direction {
            T::MAX_FINITE
        } else {
            T::MIN_FINITE
        };
        let mut sample = shift.clone();
        let mut success = false;
        for _ in 0..=trials {
            success |= !sample_bernoulli_u64(tail_threshold)?;
            if !success && sample != bound {
                if direction {
                    sample += T::one();
                } else {
                    sample -= T::one();
                }
            }
        }
        // Reject exactly one zero-magnitude proposal. At the type minimum,
        // negative draws also saturate to `shift`, so reject positive zero
        // instead to preserve that tail. Switching at other lower bounds is
        // distribution-equivalent because both signed zeros are distinguishable.
        if direction != (shift == lower) || sample != shift {
            break sample;
        }
    };

    noised.total_clamp(lower, upper)
}
