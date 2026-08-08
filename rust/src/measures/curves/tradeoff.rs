use crate::{
    error::Fallible,
    measures::curves::{TradeoffFn, check_beta, check_epsilon},
    traits::{A, C, Interval, IntervalBackend, SInterval, backend::Dashu},
    utilities::search::{SearchMode, fallible_optimize_to_precision_bracket},
};

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

    // Enclose r = exp(epsilon) with the soft backend once. The generic
    // tradeoff formula below can then be evaluated over A for search and C for
    // the final simple-arithmetic bound.
    let r = SInterval::<Dashu>::point(epsilon)?.exp()?;
    let r_lo = r.lower_f64()?;
    let r_hi = r.upper_f64()?;

    let c1 = delta_tradeoff_term(tradeoff, r_lo, r_hi, DeltaTradeoffTerm::EpsAlphaBeta)?;

    if symmetric {
        return Ok(c1.clamp(0.0, 1.0));
    }

    let c2 = delta_tradeoff_term(tradeoff, r_lo, r_hi, DeltaTradeoffTerm::EpsBetaAlpha)?;

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
fn delta_tradeoff_term(
    tradeoff: &TradeoffFn,
    r_lo: f64,
    r_hi: f64,
    term: DeltaTradeoffTerm,
) -> Fallible<f64> {
    // Use the approximate interval backend only to choose the candidate alpha
    // bracket. The final value is recomputed with C below on the returned
    // bracket, not just at the optimizer's point.
    let optimum = fallible_optimize_to_precision_bracket(
        SearchMode::Maximize,
        0.0,
        1.0,
        None,
        |alpha: f64| delta_tradeoff_term_upper_on::<A>(tradeoff, r_lo, r_hi, alpha, alpha, term),
    )?;

    delta_tradeoff_term_upper_on::<C>(tradeoff, r_lo, r_hi, optimum.lo, optimum.hi, term)
}

/// Find a finite epsilon whose directed exponential has finite f64 endpoints.
fn likelihood_ratio_epsilon_cap() -> Fallible<f64> {
    let mut epsilon = f64::MAX.ln().next_down();
    loop {
        let finite = SInterval::<Dashu>::point(epsilon)
            .and_then(|value| value.exp())
            .and_then(|value| value.lower_f64().and(value.upper_f64()))
            .is_ok();
        if finite {
            return Ok(epsilon);
        }

        let next = epsilon.next_down();
        if next == epsilon {
            return fallible!(FailedMap, "unable to find finite likelihood-ratio cap");
        }
        epsilon = next;
    }
}

fn tradeoff_delta_at_infinity(tradeoff: &TradeoffFn, symmetric: bool) -> Fallible<f64> {
    let beta_zero = tradeoff(0.0)?;
    check_beta(beta_zero)?;

    let one = SInterval::<Dashu>::point(1.0)?;
    let first_term = (one - SInterval::<Dashu>::point(beta_zero)?)?.upper_f64()?;

    if symmetric {
        return Ok(first_term.clamp(0.0, 1.0));
    }

    let alpha_zero = first_zero_alpha_lower_bound(tradeoff, beta_zero)?;
    let second_term =
        (SInterval::<Dashu>::point(1.0)? - SInterval::<Dashu>::point(alpha_zero)?)?.upper_f64()?;

    Ok(first_term.max(second_term).clamp(0.0, 1.0))
}

/// Return the lower endpoint of the first alpha at which beta reaches zero,
/// using the same left-sided generalized inverse convention as `PrivacyGuarantee::alpha`.
fn first_zero_alpha_lower_bound(tradeoff: &TradeoffFn, beta_zero: f64) -> Fallible<f64> {
    let beta_one = tradeoff(1.0)?;
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
                let beta = tradeoff(*alpha)?;
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
fn delta_tradeoff_term_upper_on<Bk>(
    tradeoff: &TradeoffFn,
    r_lo: f64,
    r_hi: f64,
    alpha_lo: f64,
    alpha_hi: f64,
    term: DeltaTradeoffTerm,
) -> Fallible<f64>
where
    Bk: IntervalBackend,
{
    debug_assert!(alpha_lo <= alpha_hi);

    let alpha_lo = alpha_lo.max(0.0);
    let alpha_hi = alpha_hi.min(1.0);

    let beta_lo = tradeoff(alpha_hi)?;
    check_beta(beta_lo)?;

    let one = Interval::<Bk>::point(1.0)?;
    let r = Interval::<Bk>::between(r_lo, r_hi)?;
    let alpha = Interval::<Bk>::between(alpha_lo, alpha_hi)?;

    // TradeoffFn is already conservative: beta_lo is a lower bound on beta at
    // alpha_hi, and monotonicity makes it a lower bound over [alpha_lo, alpha_hi].
    // Use a wide interval so subtraction can use the lower endpoint while the
    // formula remains backend-generic.
    let beta = Interval::<Bk>::between(beta_lo.max(0.0), 1.0)?;

    let subtrahend = match term {
        DeltaTradeoffTerm::EpsAlphaBeta => ((r * alpha)? + beta)?,
        DeltaTradeoffTerm::EpsBetaAlpha => ((r * beta)? + alpha)?,
    };

    (one - subtrahend)?.clamp01()?.upper_f64()
}
