use std::ops::SubAssign;

use num::{One, Zero};

use crate::{
    error::Fallible,
    traits::{samplers::SampleBernoulli, Float, SaturatingAdd, WrappingAdd, WrappingSub, SaturatingSub, AlertingSub},
};

use super::Tail;

pub fn sample_geometric_linear_time<T, P>(
    mut shift: T,
    positive: bool,
    prob: P,
    tail: Tail<T>
) -> Fallible<T>
where
    T: Clone + Zero + One + PartialEq + SubAssign + AlertingSub + SaturatingAdd + SaturatingSub + WrappingAdd + WrappingSub,
    P: Float,
    bool: SampleBernoulli<P>,
{
    let mut success: bool = false;
    let _1 = T::one();
    let mut trials = if let Tail::Censored(Some((lower, upper))) = &tail {
        Some(upper.alerting_sub(lower)?.alerting_sub(&T::one())?)
    } else {
        None
    };

    // loop must increment at least once
    loop {
        // make steps on `shift` until there is a successful trial or have reached a boundary
        if !success {
            shift = match &tail {
                &Tail::Censored(_) => if positive {
                    shift.saturating_add(&_1)
                } else {
                    shift.saturating_sub(&_1)
                },
                &Tail::Modular => if positive {
                    shift.wrapping_add(&_1)
                } else {
                    shift.wrapping_sub(&_1)
                }
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
