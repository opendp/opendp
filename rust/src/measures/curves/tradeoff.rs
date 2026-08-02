use crate::{
    error::Fallible,
    measures::curves::{TradeoffFn, check_beta, check_epsilon},
    traits::{A, C, D, Interval, IntervalArithmeticBackend},
    utilities::search::{SearchMode, optimize_to_precision_bracket},
};

pub fn delta_via_tradeoff(tradeoff: &TradeoffFn, symmetric: bool, epsilon: f64) -> Fallible<f64> {
    check_epsilon(epsilon)?;

    // Enclose r = exp(epsilon) with the soft backend once. The generic
    // tradeoff formula below can then be evaluated over A for search and C for
    // the final simple-arithmetic bound.
    let r = Interval::<D>::point(epsilon)?.exp()?;
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
    let optimum =
        optimize_to_precision_bracket(SearchMode::Maximize, 0.0, 1.0, None, |alpha: f64| {
            delta_tradeoff_term_upper_on::<A>(tradeoff, r_lo, r_hi, alpha, alpha, term)
                .unwrap_or_else(|_| SearchMode::Maximize.bad_value())
        });

    delta_tradeoff_term_upper_on::<C>(tradeoff, r_lo, r_hi, optimum.lo, optimum.hi, term)
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
    Bk: IntervalArithmeticBackend,
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
        DeltaTradeoffTerm::EpsAlphaBeta => r.mul(alpha)?.add(beta)?,
        DeltaTradeoffTerm::EpsBetaAlpha => r.mul(beta)?.add(alpha)?,
    };

    one.sub(subtrahend)?.clamp01()?.upper_f64()
}
