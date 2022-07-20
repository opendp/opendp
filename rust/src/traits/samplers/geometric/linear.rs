use std::ops::{AddAssign, SubAssign};

use num::{One, Zero};

use crate::{
    error::Fallible,
    traits::{samplers::SampleBernoulli, Float, OptionFiniteBounds},
};

pub fn sample_geometric_linear_time<T, P>(
    mut shift: T,
    positive: bool,
    prob: P,
    mut trials: Option<T>,
) -> Fallible<T>
where
    T: Clone + Zero + One + PartialEq + AddAssign + SubAssign + OptionFiniteBounds,
    P: Float,
    bool: SampleBernoulli<P>,
{
    let bound = if positive {
        T::OPTION_MAX_FINITE
    } else {
        T::OPTION_MIN_FINITE
    };
    let mut success: bool = false;

    // loop must increment at least once
    loop {
        // make steps on `shift` until there is a successful trial or have reached a boundary
        if !success && bound.as_ref().map(|b| b != &shift).unwrap_or(true) {
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
            *trials -= T::one();
        } else if success {
            // otherwise break on first success
            break;
        }

        // run a trial-- do we stop?
        success |= bool::sample_bernoulli(prob, trials.is_some())?;
    }
    Ok(shift)
}
