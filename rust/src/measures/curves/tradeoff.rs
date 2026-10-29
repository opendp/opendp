use dashu::rational::RBig;

use crate::{
    error::{ErrorVariant, Fallible},
    measures::curves::{TradeoffFn, check_beta, check_epsilon, eval_tradeoff},
    traits::{Certified, DirectedScalar, Direction, N64, SInterval},
    utilities::search::{SearchMode, fallible_golden_section_bracket_by},
};

#[cfg(test)]
mod test;

pub fn delta_via_tradeoff(tradeoff: &TradeoffFn, symmetric: bool, epsilon: f64) -> Fallible<f64> {
    check_epsilon(epsilon)?;

    if epsilon == f64::INFINITY {
        return tradeoff_delta_at_infinity(tradeoff, symmetric);
    }

    // The finite likelihood-ratio calculation cannot represent an outward
    // exponential endpoint above f64::MAX. Evaluate at a finite cap instead:
    // delta is nonincreasing in epsilon, so delta(cap) is conservative for
    // every larger finite epsilon.
    let epsilon = epsilon.min(likelihood_ratio_epsilon_cap()?);

    // Only a lower bound on r = exp(epsilon) is needed to upper-bound delta.
    let r_lo = SInterval::point(epsilon)?.exp()?.lower_f64()?;
    let c1 = delta_tradeoff_term(tradeoff, r_lo, DeltaTradeoffTerm::EpsAlphaBeta)?;

    if symmetric {
        return Ok(c1.clamp(0.0, 1.0));
    }

    let c2 = delta_tradeoff_term(tradeoff, r_lo, DeltaTradeoffTerm::EpsBetaAlpha)?;

    Ok(c1.max(c2).clamp(0.0, 1.0))
}

#[derive(Clone, Copy, Debug)]
enum DeltaTradeoffTerm {
    // 1 - exp(epsilon) * alpha - beta(alpha)
    EpsAlphaBeta,

    // 1 - exp(epsilon) * beta(alpha) - alpha
    EpsBetaAlpha,
}

#[inline]
fn delta_tradeoff_term(tradeoff: &TradeoffFn, r_lo: f64, term: DeltaTradeoffTerm) -> Fallible<f64> {
    // Proof obligations for excluding regions:
    // 1. b is a continuous convex nonincreasing real-domain lower bound.
    //    User callbacks attest this INCLUDING numerical error. Internally
    //    derived rounded bounds are not routed through this optimizer.
    // 2. Both r_lo*alpha+b(alpha) and alpha+r_lo*b(alpha) are convex (r_lo>0).
    //    RBig evaluates these objectives exactly, preserving their ordering.
    // 3. For lo<c<d<hi, q(c)<=q(d) retains a minimizer in [lo,d]; otherwise
    //    one is retained in [c,hi]. Equality is safe even for flat minima:
    //    convexity guarantees a minimizer in [c,d]. The search checks coordinate
    //    degeneracy BEFORE updating the interval, preserving this invariant.
    // 4. The final interval-wide lower objective bound below uses alpha>=lo
    //    and true B(alpha)>=b(hi)>=the conservative f64 callback(hi).
    let r = RBig::try_from(r_lo)?;
    let objective = |alpha| -> Fallible<RBig> {
        let exact_alpha = RBig::try_from(alpha)?;
        let beta = RBig::try_from(eval_beta(tradeoff, alpha)?)?;
        Ok(match term {
            DeltaTradeoffTerm::EpsAlphaBeta => &r * exact_alpha + beta,
            DeltaTradeoffTerm::EpsBetaAlpha => &r * beta + exact_alpha,
        })
    };
    // For a convex function, equality at both endpoints and an interior point
    // proves it is constant on the whole interval. Certify the midpoint as a
    // minimizer directly, rather than spending ~1500 golden steps approaching
    // subnormal zero on a flat pure-DP objective. This is an exact equality
    // proof, not a tolerance, refinement budget, or grid certificate.
    let at_zero = objective(0.0)?;
    let at_mid = objective(0.5)?;
    let (lo, hi) = if at_zero == at_mid && objective(1.0)? == at_mid {
        (0.5, 0.5)
    } else {
        fallible_golden_section_bracket_by(
            SearchMode::Minimize,
            0.0,
            1.0,
            objective,
            |left: &RBig, right: &RBig| Ok(left.cmp(right)),
        )?
    };
    delta_tradeoff_term_upper_on::<N64<Certified>>(tradeoff, r_lo, lo, hi, term)
}

/// Find a finite epsilon whose directed exponential has finite f64 endpoints.
fn likelihood_ratio_epsilon_cap() -> Fallible<f64> {
    let mut epsilon = f64::MAX.ln().next_down();
    loop {
        match SInterval::point(epsilon)
            .and_then(|value| value.exp())
            .and_then(|value| value.lower_f64().and(value.upper_f64()))
        {
            Ok(_) => return Ok(epsilon),
            Err(error) if error.variant == ErrorVariant::Overflow => (),
            Err(error) => return Err(error),
        }

        let next = epsilon.next_down();
        if next == epsilon {
            return fallible!(FailedMap, "unable to find finite likelihood-ratio cap");
        }
        epsilon = next;
    }
}

fn tradeoff_delta_at_infinity(tradeoff: &TradeoffFn, symmetric: bool) -> Fallible<f64> {
    let beta_zero = eval_beta(tradeoff, 0.0)?;
    check_beta(beta_zero)?;

    let one = SInterval::point(1.0)?;
    let first_term = (one - SInterval::point(beta_zero)?)?.upper_f64()?;

    if symmetric {
        return Ok(first_term.clamp(0.0, 1.0));
    }

    let alpha_zero = first_zero_alpha_lower_bound(tradeoff, beta_zero)?;
    let second_term = (SInterval::point(1.0)? - SInterval::point(alpha_zero)?)?.upper_f64()?;

    Ok(first_term.max(second_term).clamp(0.0, 1.0))
}

/// Return the lower endpoint of the first alpha at which beta reaches zero,
/// using the same left-sided generalized inverse convention as `PrivacyGuarantee::alpha`.
fn first_zero_alpha_lower_bound(tradeoff: &TradeoffFn, beta_zero: f64) -> Fallible<f64> {
    let beta_one = eval_beta(tradeoff, 1.0)?;
    check_beta(beta_one)?;

    if beta_zero == 0.0 {
        return Ok(0.0);
    }
    if beta_one > 0.0 {
        return Ok(1.0);
    }

    let crossing = crate::utilities::search::fallible_binary_search(
        |alpha| {
            let beta = if *alpha == 0.0 {
                beta_zero
            } else if *alpha == 1.0 {
                beta_one
            } else {
                let beta = eval_beta(tradeoff, *alpha)?;
                check_beta(beta)?;
                beta
            };

            Ok(beta == 0.0)
        },
        (0.0, 1.0),
    )?;

    Ok(crossing.next_down().clamp(0.0, 1.0))
}

#[inline]
fn delta_tradeoff_term_upper_on<S>(
    tradeoff: &TradeoffFn,
    r_lo: f64,
    alpha_lo: f64,
    alpha_hi: f64,
    term: DeltaTradeoffTerm,
) -> Fallible<f64>
where
    S: DirectedScalar,
{
    debug_assert!(alpha_lo <= alpha_hi);

    let alpha_lo = alpha_lo.max(0.0);
    let alpha_hi = alpha_hi.min(1.0);

    let beta_lo = eval_beta(tradeoff, alpha_hi)?;
    check_beta(beta_lo)?;

    let r = S::exact(r_lo)?;
    let alpha = S::exact(alpha_lo)?;
    let beta = S::exact(beta_lo)?;

    // For every point in this cell, the TRUE monotone curve B satisfies
    // B(alpha) >= B(alpha_hi) >= callback(alpha_hi), even if numerical lower
    // approximations are not convex. These bounds certify both objectives on
    // the whole cell. Use only the required rounding direction, avoiding
    // overflow of irrelevant upper endpoints of r * alpha + beta.
    let subtrahend = match term {
        DeltaTradeoffTerm::EpsAlphaBeta => r
            .mul_round(alpha, Direction::Down)?
            .add_round(beta, Direction::Down)?,
        DeltaTradeoffTerm::EpsBetaAlpha => r
            .mul_round(beta, Direction::Down)?
            .add_round(alpha, Direction::Down)?,
    };
    Ok(S::exact(1.0)?
        .sub_round(subtrahend, Direction::Up)?
        .to_f64(Direction::Up)?
        .clamp(0.0, 1.0))
}

// Numerical searches query the single vector callback on singleton batches.
fn eval_beta(tradeoff: &TradeoffFn, alpha: f64) -> Fallible<f64> {
    Ok(eval_tradeoff(tradeoff, &[alpha])?[0])
}
