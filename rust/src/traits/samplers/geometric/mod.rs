use std::ops::{SubAssign, Sub, AddAssign};

use num::{Zero, One, clamp};

use crate::{error::Fallible, traits::{AlertingSub, InfExp, InfAdd, InfSub, InfDiv, FiniteBounds, TotalOrd}};

use super::{SampleBernoulli, SampleUniform, SampleStandardBernoulli};


pub trait SampleGeometric: Sized {

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
    /// use opendp::traits::samplers::SampleGeometric;
    /// let geom = u8::sample_geometric(0, true, 0.1, Some(20));
    /// # use opendp::error::ExplainUnwrap;
    /// # geom.unwrap_test();
    /// ```
    fn sample_geometric(shift: Self, positive: bool, prob: f64, trials: Option<Self>) -> Fallible<Self>;
}

impl<T: Clone + Zero + One + PartialEq + AddAssign + SubAssign + FiniteBounds> SampleGeometric for T {

    fn sample_geometric(mut shift: Self, positive: bool, prob: f64, mut trials: Option<Self>) -> Fallible<Self> {

        // ensure that prob is a valid probability
        if !(0.0..=1.0).contains(&prob) {return fallible!(FailedFunction, "probability is not within [0, 1]")}

        let bound = if positive { Self::MAX_FINITE } else { Self::MIN_FINITE };
        let mut success: bool = false;

        // loop must increment at least once
        loop {
            // make steps on `shift` until there is a successful trial or have reached the boundary
            if !success && shift != bound {
                if positive { shift += T::one() } else { shift -= T::one() }
            }

            // stopping criteria
            if let Some(trials) = trials.as_mut() {
                // in the constant-time regime, decrement trials until zero
                if trials.is_zero() { break }
                *trials -= T::one();
            } else if success {
                // otherwise break on first success
                break
            }

            // run a trial-- do we stop?
            success |= bool::sample_bernoulli(prob, trials.is_some())?;
        }
        Ok(shift)
    }
}

pub trait SampleTwoSidedGeometric: SampleGeometric {

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
    /// use opendp::traits::samplers::SampleTwoSidedGeometric;
    /// let geom = u8::sample_two_sided_geometric(0, 0.1, Some((20, 30)));
    /// # use opendp::error::ExplainUnwrap;
    /// # geom.unwrap_test();
    /// ```
    fn sample_two_sided_geometric(
        shift: Self, scale: f64, bounds: Option<(Self, Self)>
    ) -> Fallible<Self>;
}

impl<T: Clone + SampleGeometric + Sub<Output=T> + FiniteBounds + Zero + One + TotalOrd + AlertingSub> SampleTwoSidedGeometric for T {
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
    fn sample_two_sided_geometric(mut shift: T, scale: f64, bounds: Option<(Self, Self)>) -> Fallible<Self>  {
        if scale.is_zero() {return Ok(shift)}
        let trials: Option<T> = if let Some((lower, upper)) = bounds.clone() {
            // if the output interval is a point
            if lower == upper {return Ok(lower)}
            Some(upper.alerting_sub(&lower)?.alerting_sub(&T::one())?)
        } else {None};

        // make alpha conservatively larger
        let inf_alpha: f64 = (-scale.recip()).inf_exp()?;

        // It should be possible to drop the input clamp at a cost of `delta = 2^(-(upper - lower))`.
        // Thanks for the input @ctcovington (Christian Covington)
        if let Some((lower, upper)) = &bounds {
            shift = clamp(shift, lower.clone(), upper.clone());
        }

        // TODO: check MIR for reordering that moves these samples inside the conditional
        // TODO: benchmark execution time on different inputs
        let uniform = f64::sample_standard_uniform(bounds.is_some())?;
        let direction = bool::sample_standard_bernoulli()?;
        // make prob conservatively smaller, because a smaller probability means greater noise
        let geometric = T::sample_geometric(
            shift.clone(), direction, (1.).neg_inf_sub(&inf_alpha)?, trials)?;

        // add 0 noise with probability (1-alpha) / (1+alpha), otherwise use geometric sample
        // rounding should always make threshold smaller
        let threshold = (1.).neg_inf_sub(&inf_alpha)?.neg_inf_div(
            &(1.).inf_add(&inf_alpha)?)?;
        let noised = if uniform < threshold { shift } else { geometric };

        Ok(if let Some((lower, upper)) = bounds {
            clamp(noised, lower, upper)
        } else {
            noised
        })
    }
}



#[cfg(test)]
mod test {
    #[test]
    #[cfg(feature="test-plot")]
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
            .build()?.show().unwrap();
        Ok(())
    }
}