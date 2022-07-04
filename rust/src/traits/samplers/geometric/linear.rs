use std::ops::{AddAssign, SubAssign};

use num::{Zero, One, Bounded};

use crate::{error::Fallible, traits::samplers::{SampleExponent, SampleBernoulli}};


pub fn sample_geometric_linear_time<T, S>(
    mut shift: T,
    positive: bool,
    prob: S,
    mut trials: Option<T>,
) -> Fallible<T>
where
    T: Clone + Zero + One + PartialEq + AddAssign + Bounded + SubAssign,
    S: num::Float + Copy + One + Zero + PartialOrd + SampleExponent,
    S::Bits: PartialOrd,
{
    let bound = if positive {
        T::MAX_FINITE
    } else {
        T::MIN_FINITE
    };
    let mut success: bool = false;

    // loop must increment at least once
    loop {
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
