use super::*;
use crate::{measures::curves::test::batch, traits::InfCast};

#[test]
fn test_interval_certificate_at_adjacent_floats() -> Fallible<()> {
    for lo in [0.0, f64::from_bits(1), 0.1, 0.5, 1.0_f64.next_down()] {
        let hi = lo.next_up();
        for r in [1.0, 2.0, f64::MAX] {
            let beta = 0.25;
            for term in [
                DeltaTradeoffTerm::EpsAlphaBeta,
                DeltaTradeoffTerm::EpsBetaAlpha,
            ] {
                let actual = delta_tradeoff_term_upper_on::<N64<Certified>>(
                    &batch(move |_| Ok(beta)),
                    r,
                    lo,
                    hi,
                    term,
                )?;
                let r = RBig::try_from(r)?;
                let alpha = RBig::try_from(lo)?;
                let beta = RBig::try_from(beta)?;
                let objective = match term {
                    DeltaTradeoffTerm::EpsAlphaBeta => r * alpha + beta,
                    DeltaTradeoffTerm::EpsBetaAlpha => r * beta + alpha,
                };
                assert!(RBig::try_from(actual)? >= (RBig::from(1) - objective).max(RBig::from(0)));
            }
        }
    }
    Ok(())
}

#[test]
fn test_nonsymmetric_infinity_and_early_approximate_zeros() -> Fallible<()> {
    // Infinity does not use a convex optimizer; preserve early-zero and
    // nondyadic-crossing adversaries independently of the finite callback contract.
    for quantized in [false, true] {
        let callback = move |alpha| {
            let beta = (SInterval::point(0.875)?
                - (SInterval::point(2.0)? * SInterval::point(alpha)?)?)?
            .clamp01()?
            .lower_f64()?;
            Ok(if quantized {
                (beta * 8.0).floor() / 8.0
            } else {
                beta
            })
        };
        let actual = delta_via_tradeoff(&batch(callback), false, f64::INFINITY)?;
        assert!(actual >= 0.5625 && actual <= 0.625_f64.next_up());
    }
    let actual = delta_via_tradeoff(
        &batch(|alpha| {
            (SInterval::point(1.0)? - (SInterval::point(3.0)? * SInterval::point(alpha)?)?)?
                .clamp01()?
                .lower_f64()
        }),
        false,
        f64::INFINITY,
    )?;
    assert!(RBig::try_from(actual)? >= RBig::from(2) / RBig::from(3));
    Ok(())
}

#[test]
fn test_certification_propagates_callback_errors() {
    for variant in ["NumericError", "Overflow", "FailedFunction"] {
        let callback = move |alpha| {
            if alpha > 0.75 {
                return Err(crate::error::Error {
                    variant: variant.parse().unwrap(),
                    ..err!(FailedMap, "late callback failure")
                });
            }
            Ok(1.0 - alpha)
        };
        // The second direction searches toward alpha=1 and encounters the
        // failure. Errors in unevaluated regions need not be discovered.
        let error = delta_via_tradeoff(&batch(callback), false, 1.0).unwrap_err();
        assert_eq!(error.variant.to_string(), variant);
    }
}

#[test]
fn test_constant_objectives_against_exact_reference() -> Fallible<()> {
    for beta in [0.0, f64::from_bits(1), 0.25, 0.5, 1.0] {
        let callback = batch(move |_| Ok(beta));
        for epsilon in [
            0.0,
            0.1,
            1.0,
            20.0,
            likelihood_ratio_epsilon_cap()?,
            f64::MAX,
            f64::INFINITY,
        ] {
            for symmetric in [false, true] {
                let actual = delta_via_tradeoff(&callback, symmetric, epsilon)?;
                let reference = RBig::from(1) - RBig::try_from(beta)?;
                assert!(RBig::try_from(actual)? >= reference);
                assert!(actual <= f64::inf_cast(reference)? + 1e-14);
            }
        }
    }
    Ok(())
}

#[test]
fn test_kinks_flat_minima_and_narrow_optimizer_intervals() -> Fallible<()> {
    // A dyadic kink replaces the removed point-envelope fixture. The exact
    // objective exercises the existing bracket algorithm without a cache.
    for r in [1, 2, 4] {
        let kink = RBig::from(1) / RBig::from(2);
        let objective = |alpha: f64| -> Fallible<RBig> {
            let alpha = RBig::try_from(alpha)?;
            let beta = (RBig::from(1) - RBig::from(2) * &alpha).max(RBig::from(0));
            Ok(RBig::from(r) * alpha + beta)
        };
        let (lo, hi) = fallible_golden_section_bracket_by(
            SearchMode::Minimize,
            0.0,
            1.0,
            objective,
            |a: &RBig, b: &RBig| Ok(a.cmp(b)),
        )?;
        if r < 2 {
            assert!(RBig::try_from(lo)? <= kink && kink <= RBig::try_from(hi)?);
        }
        if r > 2 {
            assert_eq!(lo, 0.0);
        }
        let lo = 0.5_f64.next_down();
        let hi = 0.5;
        let (left, right) = fallible_golden_section_bracket_by(
            SearchMode::Minimize,
            lo,
            hi,
            objective,
            |a: &RBig, b: &RBig| Ok(a.cmp(b)),
        )?;
        assert_eq!((left, right), (lo, hi));
    }
    Ok(())
}
