use std::{
    convert::TryFrom,
    ops::{Sub, SubAssign},
};

use num::{clamp, One, Zero};

use crate::{
    error::Fallible,
    traits::{
        AlertingSub, ExactIntCast, Float, SaturatingAdd, SaturatingSub,
        TotalOrd, WrappingAdd, WrappingSub, SaturatingCast, WrappingCast,
    },
};

use super::{fill_bytes, SampleBernoulli, SampleStandardBernoulli};

mod linear;
use self::linear::sample_geometric_linear_time;

#[cfg(feature="use-mpfr")]
mod logarithmic;
#[cfg(feature="use-mpfr")]
use self::logarithmic::sample_geometric_log_time;

pub trait SampleGeometric<P>: Sized {
    /// Sample from the censored geometric distribution with parameter `prob`.
    /// If `trials` is None, there are no timing protections, and the support is:
    ///     [Self::MIN, Self::MAX]
    /// If `trials` is Some, execution runs in constant time, and the support is
    ///     [Self::MIN, Self::MAX] ∩ {shift ±= {1, 2, 3, ..., `trials`}}
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
    /// use opendp::traits::samplers::{SampleGeometric, Tail};
    /// let geom = u8::sample_geometric(0, true, 0.1, Tail::Censored(Some((0, 20))));
    /// # use opendp::error::ExplainUnwrap;
    /// # geom.unwrap_test();
    /// ```
    fn sample_geometric(
        shift: Self,
        positive: bool,
        prob: P,
        tail: Tail<Self>,
    ) -> Fallible<Self>;
}

#[cfg(feature="use-mpfr")]
impl<T, P> SampleGeometric<P> for T
where
    T: 'static
        + Clone
        + Zero
        + One
        + PartialEq
        + SubAssign
        + SaturatingAdd
        + SaturatingSub
        + WrappingAdd
        + WrappingSub
        + AlertingSub
        + TryFrom<rug::Integer>
        + SaturatingCast<rug::Integer>
        + WrappingCast<rug::Integer>,
    P: 'static + Float,
    rug::Integer: From<T> + TryFrom<T>,
    rug::Rational: TryFrom<P>,
    bool: SampleBernoulli<P>,
{
    fn sample_geometric(
        shift: Self,
        positive: bool,
        prob: P,
        tail: Tail<Self>,
    ) -> Fallible<Self> {
        // ensure that prob is a valid probability
        if !(P::zero()..=P::one()).contains(&prob) {
            return fallible!(FailedFunction, "probability is not within [0, 1]");
        }

        if matches!(tail, Tail::Censored(Some(_))) || prob.recip() < P::exact_int_cast(3)? {
            sample_geometric_linear_time(shift, positive, prob, tail)
        } else {
            sample_geometric_log_time(shift, positive, prob, tail)
        }
    }
}

#[cfg(not(feature="use-mpfr"))]
impl<T, P> SampleGeometric<P> for T
where
    T: Clone + Zero + One + PartialEq + SubAssign + AlertingSub + SaturatingAdd + SaturatingSub + WrappingAdd + WrappingSub,
    P: Float,
    bool: SampleBernoulli<P>,
{
    fn sample_geometric(
        shift: Self,
        positive: bool,
        prob: P,
        tail: Tail<Self>,
    ) -> Fallible<Self> {
        // ensure that prob is a valid probability
        if !(P::zero()..=P::one()).contains(&prob) {
            return fallible!(FailedFunction, "probability is not within [0, 1]");
        }

        sample_geometric_linear_time(shift, positive, prob, tail)
    }
}

#[derive(Clone, Copy)]
pub enum Tail<T> {
    Censored(Option<(T, T)>),
    Modular,
}

pub trait SampleTwoSidedGeometric<P>: SampleGeometric<P> {
    /// Sample from the censored two-sided geometric distribution with parameter `prob`.
    /// If `bounds` is None, there are no timing protections, and the support is:
    ///     [Self::MIN, Self::MAX]
    /// If `bounds` is Some, execution runs in constant time, and the support is
    ///     [Self::MIN, Self::MAX] ∩ {shift ±= {1, 2, 3, ..., `trials`}}
    ///
    /// Tail probabilities of the uncensored two-sided geometric accumulate at the extrema of the support.
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
    /// use opendp::traits::samplers::{Tail, SampleTwoSidedGeometric};
    /// let geom = u8::sample_two_sided_geometric(0, 0.1, Tail::Censored(Some((20, 30))));
    /// # use opendp::error::ExplainUnwrap;
    /// # geom.unwrap_test();
    /// ```
    fn sample_two_sided_geometric(
        shift: Self,
        scale: P,
        tail: Tail<Self>,
    ) -> Fallible<Self>;
}

impl<T, P> SampleTwoSidedGeometric<P> for T
where
    T: Clone + SampleGeometric<P> + Sub<Output = T> + Zero + One + TotalOrd + AlertingSub,
    P: Float,
    P::Bits: PartialOrd + ExactIntCast<usize>,
    usize: ExactIntCast<P::Bits>
{
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
    fn sample_two_sided_geometric(
        mut shift: T,
        scale: P,
        tail: Tail<Self>,
    ) -> Fallible<Self> {
        if scale.is_zero() {
            return Ok(shift);
        }
        let bounds: Option<(T, T)> = if let Tail::Censored(Some((lower, upper))) = tail.clone() {
            // if the output interval is a point
            if lower == upper {
                return Ok(lower);
            }
            Some((lower, upper))
        } else {
            None
        };

        // make alpha conservatively larger
        let inf_alpha: P = (-scale.recip()).inf_exp()?;
        if inf_alpha.is_one() {
            return fallible!(FailedFunction, "scale is too large");
        }

        // It should be possible to drop the input clamp at a cost of `delta = 2^(-(upper - lower))`.
        // Thanks for the input @ctcovington (Christian Covington)
        if let Some((lower, upper)) = &bounds {
            shift = clamp(shift, lower.clone(), upper.clone());
        }

        // add 0 noise with probability (1-alpha) / (1+alpha), otherwise use geometric sample
        // rounding should always make threshold smaller
        let threshold = P::one()
            .neg_inf_sub(&inf_alpha)?
            .neg_inf_div(&P::one().inf_add(&inf_alpha)?)?;
        let noiseless = bool::sample_bernoulli(threshold, bounds.is_some())?;

        let direction = bool::sample_standard_bernoulli()?;
        // make prob conservatively smaller, because a smaller probability means greater noise
        let success_prob = P::one().neg_inf_sub(&inf_alpha)?;

        // sample from the geometric
        let geometric = T::sample_geometric(
            shift.clone(),
            direction,
            success_prob,
            tail,
        )?;

        let noised = if noiseless { shift } else { geometric };

        Ok(if let Some((lower, upper)) = bounds {
            clamp(noised, lower, upper)
        } else {
            noised
        })
    }
}

/// Internal function. Returns a sample from the Geometric(p=0.5) distribution.
///
/// The algorithm generates B * 8 bits at random and returns
/// - Some(index of the first set bit)
/// - None (if all bits are 0)
///
/// This is a lower-level version of the SampleGeometric
pub(super) fn sample_geometric_buffer(
    buffer_len: usize,
    constant_time: bool,
) -> Fallible<Option<usize>> {
    Ok(if constant_time {
        let mut buffer = vec![0_u8; buffer_len];
        fill_bytes(&mut buffer)?;
        buffer
            .iter()
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_sample_2sgeo() -> Fallible<()> {
        let dgeo = i8::sample_two_sided_geometric(0, 1e15, Tail::Censored(None))?;
        println!("final: {:?}", dgeo);

        let dgeo = i8::sample_two_sided_geometric(0, 1e8, Tail::Censored(None))?;
        println!("{:?}", dgeo);
        Ok(())
    }
    #[test]
    #[cfg(feature = "test-plot")]
    fn plot_geometric() -> Fallible<()> {
        let shift = 0;
        let scale = 5.;

        let title = format!("Geometric(shift={}, scale={}) distribution", shift, scale);
        let data = (0..10_000)
            .map(|_| i8::sample_two_sided_geometric(0, 1., None))
            .collect::<Fallible<Vec<i8>>>()?;

        use vega_lite_4::*;
        VegaliteBuilder::default()
            .title(title)
            .data(&data)
            .mark(Mark::Bar)
            .encoding(
                EdEncodingBuilder::default()
                    .x(XClassBuilder::default()
                        .field("data")
                        .position_def_type(Type::Nominal)
                        .build()?)
                    .y(YClassBuilder::default()
                        .field("data")
                        .position_def_type(Type::Quantitative)
                        .aggregate(NonArgAggregateOp::Count)
                        .build()?)
                    .build()?,
            )
            .build()?
            .show()
            .unwrap();
        Ok(())
    }
}
