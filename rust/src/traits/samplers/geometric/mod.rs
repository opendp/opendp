use opendp_derive::proven;

use crate::{
    error::Fallible,
    traits::{ExactIntCast, InfDiv, InfExp, Integer},
};

use super::{
    BERNOULLI_U64_DENOMINATOR, bernoulli_u64_threshold, fill_bytes, sample_bernoulli_u64,
    sample_standard_bernoulli,
};

#[cfg(test)]
mod test;

/// # Proof Definition
/// Validate finite nonnegative scale and ordered bounds.
/// Return (0, 0) for zero scale or coincident bounds.
/// Otherwise return the interval width, representable in T and usize,
/// and a threshold in (0, 2^64) whose grid probability is at least
/// $\exp(-1/\texttt{scale})$, or return an error.
pub(crate) fn discrete_laplace_linear_parameters<T>(
    scale: f64,
    (lower, upper): (T, T),
) -> Fallible<(usize, u128)>
where
    T: Integer,
    usize: ExactIntCast<T>,
{
    if !scale.is_finite() || scale < 0.0 {
        return fallible!(FailedFunction, "scale must be nonnegative and finite");
    }
    if lower > upper {
        return fallible!(FailedFunction, "lower bound must not exceed upper bound");
    }
    if scale == 0.0 || lower == upper {
        return Ok((0, 0));
    }

    let width = usize::exact_int_cast(upper.alerting_sub(&lower)?)?;

    // Round the failure probability up to the u64 grid, so geometric noise
    // is never smaller than specified by scale.
    let inverse = 1.0f64.neg_inf_div(&scale)?;
    let tail_threshold = bernoulli_u64_threshold((-inverse).inf_exp()?).max(1);
    if tail_threshold == BERNOULLI_U64_DENOMINATOR {
        return fallible!(
            FailedFunction,
            "scale is too large for fixed-width geometric sampling"
        );
    }
    Ok((width, tail_threshold))
}

/// # Proof Definition
/// Width and threshold must be returned by `discrete_laplace_linear_parameters`
/// for the same bounds.
/// Clamp the input to the bounds. If the threshold is zero, return that value.
/// Otherwise add discrete Laplace noise with continuation probability
/// $\texttt{threshold}/2^{64}$ and clamp the result, or return an entropy error.
pub(crate) fn sample_bounded_discrete_laplace<T: Integer>(
    shift: T,
    (lower, upper): (T, T),
    width: usize,
    threshold: u128,
) -> Fallible<T> {
    let shift = shift.clamp(lower, upper);
    if threshold == 0 {
        return Ok(shift);
    }

    // Reject negative zero before executing the remaining trials.
    let (positive, mut continuing) = loop {
        let positive = sample_standard_bernoulli()?;
        let first = sample_bernoulli_u64(threshold)?;
        if positive | first {
            break (positive, first);
        }
    };

    let mut magnitude = usize::from(continuing);
    for _ in 1..width {
        continuing &= sample_bernoulli_u64(threshold)?;
        magnitude += usize::from(continuing);
    }

    // Both distances to the bounds and the magnitude are at most the validated width.
    let magnitude = T::exact_int_cast(magnitude)?;
    Ok(if positive {
        shift + magnitude.min(upper - shift)
    } else {
        shift - magnitude.min(shift - lower)
    })
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
            .filter(|(_, sample)| **sample > 0)
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
