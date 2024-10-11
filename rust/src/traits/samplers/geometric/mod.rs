use std::ops::{AddAssign, SubAssign};

use num::{One, Zero};
use opendp_derive::proven;

use crate::{
    error::Fallible,
    traits::{ExactIntCast, FiniteBounds, Float, Integer},
};

use super::{fill_bytes, sample_bernoulli_float, sample_standard_bernoulli};

#[cfg(test)]
mod test;

/// Sample from the censored geometric distribution.
///
/// # Proof Definition
/// Sample from the censored geometric distribution with parameter `prob`.
/// If `trials` is None, there are no timing protections, and the support is:
/// ```text
///     [Self::MIN, Self::MAX]
/// ```
///
/// If `trials` is Some, execution runs in constant time, and the support is
/// ```text
///     [Self::MIN, Self::MAX] ∩ {shift ± {0, 1, 2, ..., trials}}
/// ```
///
/// Tail probabilities of the uncensored geometric accumulate at the extreme value of the support.
///
/// # Arguments
/// * `shift` - Parameter to shift the output by
/// * `positive` - If true, positive noise is added, else negative
/// * `prob` - Parameter for the geometric distribution, the probability of success on any given trial.
/// * `trials` - If Some, run the algorithm in constant time with exactly this many trials.
///
/// # Return
/// A draw from the censored geometric distribution defined above.
///
/// # Example
/// ```
/// use opendp::traits::samplers::sample_geometric_linear;
/// let geom_res = sample_geometric_linear(0, true, 0.1, Some(20));
/// # geom_res.unwrap();
/// ```
pub fn sample_geometric_linear<T, P>(
    mut shift: T,
    positive: bool,
    prob: P,
    mut trials: Option<usize>,
) -> Fallible<T>
where
    T: Clone + Zero + One + PartialEq + AddAssign + SubAssign + FiniteBounds,
    P: Float,
    usize: ExactIntCast<P::Bits>,
    P::Bits: ExactIntCast<usize>,
{
    // ensure that prob is a valid probability
    if !(P::zero()..=P::one()).contains(&prob) {
        return fallible!(FailedFunction, "probability is not within [0, 1]");
    }

    let bound = if positive {
        T::MAX_FINITE
    } else {
        T::MIN_FINITE
    };
    let mut success: bool = false;

    loop {
        // run a trial-- is this our last step?
        success |= sample_bernoulli_float(prob, trials.is_some())?;

        // make steps on `shift` until there is a successful trial or have reached the boundary
        if !success && shift != bound {
            if positive {
                shift += T::one()
            } else {
                shift -= T::one()
            }
        }

        // stopping criteria
        if let Some(trials) = trials.as_mut() {
            // in the constant-time regime, decrement trials until zero
            if trials.is_zero() {
                break;
            }
            *trials -= 1;
        } else if success {
            // otherwise break on first success
            break;
        }
    }
    Ok(shift)
}

/// Sample from the censored two-sided geometric distribution.
///
/// # Proof Definition
/// Sample from the censored two-sided geometric distribution with parameter `scale`.
/// If `bounds` is None, there are no timing protections, and the support is:
/// ```text
///     [Self::MIN, Self::MAX]
/// ```
///
/// If `bounds` is Some, execution runs in constant time, and the support is
/// ```text
///     [Self::MIN, Self::MAX] ∩ {shift ± {1, 2, 3, ..., trials}}
/// ```
///
/// Tail probabilities accumulate at the extrema of the support.
///
/// # Arguments
/// * `shift` - Parameter to shift the output by
/// * `scale` - Parameter to scale the output by
/// * `bounds` - If Some, run the algorithm in constant time with both inputs and outputs clamped to this value.
///
/// # Return
/// A draw from the two-sided censored geometric distribution defined above.
///
/// # Example
/// ```
/// use opendp::traits::samplers::sample_discrete_laplace_linear;
/// let geom = sample_discrete_laplace_linear(0, 0.1, Some((20, 30)));
/// # geom.unwrap();
/// ```
///
/// When no bounds are given, there are no protections against timing attacks.
///     The bounds are effectively T::MIN and T::MAX and up to T::MAX - T::MIN trials are taken.
///     The output of this mechanism is as if samples were taken from the
///         uncensored two-sided geometric distribution and saturated at the bounds of T.
///
/// When bounds are given, samples are taken from the censored two-sided geometric distribution,
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
    bounds: Option<(T, T)>,
) -> Fallible<T>
where
    T: Integer,
    P: Float,
    usize: ExactIntCast<P::Bits> + ExactIntCast<T>,
    P::Bits: ExactIntCast<usize>,
{
    if scale.is_zero() {
        return Ok(shift);
    }
    let trials: Option<usize> = if let Some((lower, upper)) = bounds {
        // if the output interval is a point
        if lower == upper {
            return Ok(lower);
        }
        Some(usize::exact_int_cast(
            upper.alerting_sub(&lower)?.alerting_sub(&T::one())?,
        )?)
    } else {
        None
    };

    // make prob conservatively smaller, because a smaller probability means greater noise
    let prob = P::one().neg_inf_sub(&(-scale.recip()).inf_exp()?)?;

    // It should be possible to drop the input clamp at a cost of `delta = 2^(-(upper - lower))`.
    // Thanks for the input @ctcovington (Christian Covington)
    if let Some((lower, upper)) = bounds {
        shift = shift.total_clamp(lower, upper)?;
    }

    let noised = loop {
        let direction = sample_standard_bernoulli()?;
        let sample = sample_geometric_linear(shift, direction, prob, trials)?;
        // reject sampling no noise in the negative direction, to avoid double-drawing zero
        if direction || sample != shift {
            break sample;
        }
    };

    if let Some((lower, upper)) = bounds {
        noised.total_clamp(lower, upper)
    } else {
        Ok(noised)
    }
}

#[proven]
/// Sample from a specific discrete/geometric distribution.
///
/// # Proof Definition
/// For any setting of the input arguments, return
/// `Err(e)` if there is insufficient system entropy, or
/// `Ok(sample)` where `sample` is drawn from a discrete distribution.
///
/// `sample` is either
/// `None` with probability $2^{-buffer\_len * 8}$, or
/// `Some(geo)` where `geo` is a sample from the Geometric(p=0.5) distribution.
///
/// # Notes
/// The algorithm generates B * 8 bits at random and returns
/// - Some(index of the first set bit)
/// - None (if all bits are 0)
pub(super) fn sample_geometric_buffer(
    buffer_len: usize,
    constant_time: bool,
) -> Fallible<Option<usize>> {
    Ok(if constant_time {
        let mut buffer = vec![0_u8; buffer_len];
        fill_bytes(&mut buffer)?;
        (buffer.iter())
            .enumerate()
            // ignore samples that contain no events
            .filter(|(_, &sample)| sample > 0)
            // compute the index of the smallest event in the batch
            .map(|(i, sample)| 8 * i + sample.leading_zeros() as usize)
            // retrieve the smallest index
            .min()
    } else {
        // retrieve up to B bytes, each containing 8 trials
        let mut buffer = vec![0_u8; 1];
        for i in 0..buffer_len {
            fill_bytes(&mut buffer)?;

            if buffer[0] > 0 {
                return Ok(Some(i * 8 + buffer[0].leading_zeros() as usize));
            }
        }
        None
    })
}
