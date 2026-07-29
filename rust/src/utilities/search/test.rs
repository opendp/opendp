use dashu::{rational::RBig, rbig};
use std::cmp::Ordering;

use super::*;
use crate::error::ErrorVariant;
use std::cell::Cell;

#[test]
fn test_binary_search() -> Fallible<()> {
    assert_eq!(binary_search::<i32>(|x| *x <= -5, ())?, -5);
    assert_eq!(binary_search::<i32>(|x| *x <= 5, ())?, 5);
    assert_eq!(binary_search::<i32>(|x| *x >= -5, ())?, -5);
    assert_eq!(binary_search::<i32>(|x| *x >= 5, ())?, 5);
    Ok(())
}

#[test]
fn test_binary_search_sorts_bounds() -> Fallible<()> {
    assert_eq!(binary_search(|x: &i32| *x > 5, (10, 0))?, 6);
    assert_eq!(binary_search(|x: &i32| *x < 5, (10, 0))?, 4);
    Ok(())
}

#[test]
fn test_bound_specs_resolve() {
    assert_eq!(BoundSpec::<i32>::resolve(()), (None, None));
    assert_eq!(BoundSpec::resolve((0, 10)), (Some(0), Some(10)));
    assert_eq!(BoundSpec::resolve(Above(0)), (Some(0), None));
    assert_eq!(BoundSpec::resolve(Below(10)), (None, Some(10)));
    assert_eq!(BoundSpec::resolve((Some(0), None)), (Some(0), None::<i32>));
    assert_eq!(
        BoundSpec::resolve((None, Some(10))),
        (None::<i32>, Some(10))
    );
    assert_eq!(BoundSpec::<i32>::resolve(None), (None, None));
    assert_eq!(BoundSpec::resolve(Some((0, 10))), (Some(0), Some(10)));
}

#[test]
fn test_binary_search_one_sided_bounds() -> Fallible<()> {
    let predicate = |x: &i32| *x <= -5;

    assert_eq!(binary_search(predicate, Above(-10))?, -5);
    assert_eq!(binary_search(predicate, Below(-1))?, -5);
    assert_eq!(binary_search(predicate, (Some(-10), None))?, -5);
    assert_eq!(binary_search(predicate, (None, Some(-1)))?, -5);

    let err = binary_search(predicate, Above(0)).unwrap_err();
    assert_eq!(
        err.message.as_deref(),
        Some(
            "the decision boundary is below the lower bound or the predicate does not change above it"
        )
    );

    let err = binary_search(predicate, Below(-10)).unwrap_err();
    assert_eq!(
        err.message.as_deref(),
        Some(
            "the decision boundary is above the upper bound or the predicate does not change below it"
        )
    );
    assert_eq!(binary_search::<i32>(|x| *x >= 5, None)?, 5);
    assert_eq!(binary_search(|x: &i32| *x >= 5, Some((0, 10)))?, 5);
    assert_eq!(binary_search(|x: &u32| *x >= 5, Below(10))?, 5);
    Ok(())
}

#[test]
fn test_signed_binary_search_reports_direction() -> Fallible<()> {
    let calls = Cell::new(0);
    assert_eq!(
        signed_binary_search(
            |x: &i32| {
                calls.set(calls.get() + 1);
                *x >= 5
            },
            (0, 10)
        )?,
        (5, 1)
    );
    assert_eq!(calls.get(), 6); // includes exactly one evaluation of each endpoint
    assert_eq!(signed_binary_search(|x: &i32| *x <= 5, (0, 10))?, (5, -1));
    Ok(())
}

#[test]
fn test_exponential_bounds_search_bands() {
    assert_eq!(exponential_bounds_search::<i32>(&|x| *x > 5), Some((1, 16)));
    assert_eq!(
        exponential_bounds_search::<f64>(&|x| *x > 5.0),
        Some((2.0, 16.0))
    );
}

#[test]
fn test_initial_overflow_anchor_discovery() -> Fallible<()> {
    for (sign, bounds) in [(1, (None, None)), (-1, (None, None))] {
        let predicate = |x: &i32| {
            if *x * sign <= 0 {
                return fallible!(Overflow, "origin unavailable");
            }
            Ok(*x * sign >= 5)
        };
        let found = fallible_binary_search(predicate, bounds)?;
        assert_eq!(found, 5 * sign);
        assert!(predicate(&found)?);
    }
    let calls = Cell::new(0);
    let error = fallible_binary_search(
        |x: &i32| {
            calls.set(calls.get() + 1);
            fallible!(Overflow, "initial overflow at {x}")
        },
        (),
    )
    .unwrap_err();
    assert_eq!(error.variant, ErrorVariant::Overflow);
    assert_eq!(error.message.as_deref(), Some("initial overflow at 0"));
    assert!(calls.get() < 40);
    assert_eq!(
        fallible_binary_search::<i32>(
            |x| {
                if *x >= 0 {
                    return fallible!(Overflow, "initial point unavailable");
                }
                Ok(*x <= -5)
            },
            ()
        )?,
        -5
    );
    Ok(())
}

#[test]
fn test_binary_search_handles_full_signed_ranges() -> Fallible<()> {
    assert_eq!(binary_search(|x: &i32| *x >= 0, (i32::MIN, i32::MAX))?, 0);
    assert_eq!(binary_search(|x: &i8| *x <= 0, (i8::MIN, i8::MAX))?, 0);
    Ok(())
}

#[test]
fn test_binary_search_one_sided_float_ranges() -> Fallible<()> {
    assert_eq!(binary_search(|x: &f32| *x <= 5.0, Above(0.0))?, 5.0);
    assert_eq!(binary_search(|x: &f64| *x <= 5.0, Above(1.0))?, 5.0);
    assert_eq!(binary_search(|x: &f64| *x >= -5.0, Below(-1.0))?, -5.0);
    assert_eq!(binary_search(|x: &f64| *x >= 5.0, Above(0.0))?, 5.0);
    Ok(())
}

#[test]
fn test_float_midpoint_handles_opposite_sign_extremes() {
    let midpoint = <f64 as BinarySearchable>::midpoint(&f64::MIN, &f64::MAX);
    assert!(midpoint.is_finite());
    assert!(!midpoint.is_nan());

    let midpoint = <f32 as BinarySearchable>::midpoint(&f32::MIN, &f32::MAX);
    assert!(midpoint.is_finite());
    assert!(!midpoint.is_nan());
}

#[test]
fn test_binary_search_uses_search_error_variant() {
    let err = binary_search(|x: &i32| *x < 0, (0, 10)).unwrap_err();
    assert_eq!(err.variant, ErrorVariant::Search);
    assert_eq!(
        err.message.as_deref(),
        Some("the decision boundary of the predicate is outside the bounds")
    );
}

#[test]
fn test_float_bands_and_full_range() -> Fallible<()> {
    macro_rules! check {
        ($ty:ty, $bits:expr, $thresholds:expr) => {
            for center in [0.0, -0.0, 1.0, <$ty>::MIN, <$ty>::MAX] {
                for sign in [-1.0, 1.0] {
                    let bands = <$ty>::bands(center, sign as i8);
                    assert_eq!(*bands.last().unwrap(), sign * <$ty>::MAX);
                    assert!(bands.iter().all(|x| x.is_finite()));
                    assert!(bands.windows(2).all(|w| sign * w[0] < sign * w[1]));
                }
            }
            for threshold in $thresholds {
                for sign in [-1.0, 1.0] {
                    let threshold = sign * threshold;
                    for bounds in [(None, None), (Some(<$ty>::MIN), Some(<$ty>::MAX))] {
                        let calls = Cell::new(0);
                        let predicate = |x: &$ty| {
                            calls.set(calls.get() + 1);
                            sign * *x >= sign * threshold
                        };
                        let (result, direction) = signed_binary_search(predicate, bounds)?;
                        eprintln!(
                            "{} {threshold:e} {bounds:?}: {} evaluations",
                            stringify!($ty),
                            calls.get()
                        );
                        // Two explicit endpoints, or finite-band discovery, plus
                        // at most one bisection evaluation per representation bit.
                        let overhead = if bounds.0.is_some() {
                            2
                        } else {
                            1 + 2 * (<$ty>::bands(0.0, 1).len() - 1)
                        };
                        assert!(calls.get() <= $bits + overhead);
                        assert!(predicate(&result));
                        assert_eq!((result, direction), (threshold, sign as i8));
                    }
                }
            }
        };
    }
    check!(f64, 64, [1e-300, 1e100, 1e300, f64::MAX]);
    check!(f32, 32, [1e-30, 1e30, f32::MAX]);
    Ok(())
}

#[test]
fn test_nonfinite_float_bounds_are_rejected() {
    macro_rules! check {
        ($ty:ty) => {
            for invalid in [<$ty>::NAN, <$ty>::INFINITY, <$ty>::NEG_INFINITY] {
                for bounds in [
                    (Some(invalid), Some(1.0)),
                    (Some(0.0), Some(invalid)),
                    (Some(invalid), None),
                    (None, Some(invalid)),
                ] {
                    let calls = Cell::new(0);
                    let error = fallible_binary_search(
                        |_: &$ty| {
                            calls.set(calls.get() + 1);
                            Ok(true)
                        },
                        bounds,
                    )
                    .unwrap_err();
                    assert_eq!(error.variant, ErrorVariant::Search);
                    assert_eq!(calls.get(), 0);
                }
            }
        };
    }
    check!(f32);
    check!(f64);
}

#[test]
fn test_range_error_bracket_recovery() -> Fallible<()> {
    for (threshold, sign, increasing, bounds) in [
        (5.0, 1.0, true, (None, None)),
        (5.0, -1.0, false, (None, None)),
        (5.0, 1.0, false, (Some(0.0), None)),
        (5.0, -1.0, true, (Some(-10.0), Some(0.0))),
        (5.0, 1.0, true, (Some(0.0), Some(10.0))),
        (1e-300, 1.0, true, (Some(0.0), None)),
        (1e300, 1.0, true, (Some(0.0), None)),
    ] {
        let calls = Cell::new(0);
        let predicate = |x: &f64| {
            calls.set(calls.get() + 1);
            let x = sign * x;
            if x > threshold * 1.1 {
                return fallible!(Overflow, "intermediate overflow");
            }
            Ok(if increasing {
                x >= threshold
            } else {
                x <= threshold
            })
        };
        let result = fallible_binary_search(predicate, bounds)?;
        assert!(predicate(&result)?);
        assert_eq!(result, sign * threshold);
        assert!(calls.get() < 2400);
    }
    Ok(())
}

#[test]
fn test_blocked_directions() -> Fallible<()> {
    let predicate = |x: &i32| {
        if *x > 0 {
            return fallible!(Overflow, "positive side blocked");
        }
        Ok(*x <= -5)
    };
    let result = fallible_binary_search(predicate, ())?;
    assert_eq!(result, -5);
    assert!(predicate(&result)?);
    for bounds in [(None, None), (Some(0), None)] {
        let error = fallible_binary_search(
            |x: &i32| {
                if *x >= 5 {
                    return fallible!(Overflow, "passing side unavailable");
                }
                Ok(false)
            },
            bounds,
        )
        .unwrap_err();
        assert_eq!(error.variant, ErrorVariant::Overflow);
    }
    Ok(())
}

#[test]
fn test_unexpected_numerical_errors_propagate() {
    for variant in ["NumericError", "FailedFunction"] {
        for failing_at in [16, 8] {
            let error = fallible_binary_search(
                |x: &i32| {
                    if *x == failing_at {
                        return Err(Error {
                            variant: variant.parse().expect("known error variant"),
                            ..err!(FFI, "unexpected error")
                        });
                    }
                    if *x > 10 {
                        return fallible!(Overflow, "recoverable overflow");
                    }
                    Ok(*x >= 5)
                },
                (),
            )
            .unwrap_err();
            assert_eq!(error.variant.to_string(), variant);
            assert_eq!(error.message.as_deref(), Some("unexpected error"));
        }
    }
}

#[test]
fn test_bisection_overflow_is_not_comparison_information() {
    let error = fallible_binary_search(
        |x: &i32| {
            if *x == 5 {
                return fallible!(Overflow, "interior point unavailable");
            }
            Ok(*x >= 5)
        },
        (0, 10),
    )
    .unwrap_err();
    assert_eq!(error.variant, ErrorVariant::Overflow);
    assert_eq!(error.message.as_deref(), Some("interior point unavailable"));
}

#[test]
fn test_signed_zero_and_subnormals() -> Fallible<()> {
    macro_rules! check {
        ($ty:ty, $bits:expr) => {
            let tiny = <$ty>::from_bits(1);
            let max = <$ty>::MAX;
            for (lower, upper) in [
                (-max, max),
                (-tiny, 0.0),
                (-0.0, tiny),
                (-tiny, tiny),
                (1.0, (1.0 as $ty).next_up()),
                (max.next_down(), max),
                (-max, (-max).next_up()),
            ] {
                for threshold in [
                    lower,
                    upper,
                    0.0,
                    -0.0,
                    tiny,
                    -tiny,
                    <$ty>::from_bits(2),
                    -<$ty>::from_bits(2),
                    <$ty>::MIN_POSITIVE,
                    -<$ty>::MIN_POSITIVE,
                    max.next_down(),
                    (-max).next_up(),
                ] {
                    if threshold < lower || threshold > upper {
                        continue;
                    }
                    for increasing in [false, true] {
                        if (increasing && threshold == lower) || (!increasing && threshold == upper)
                        {
                            continue;
                        }
                        let calls = Cell::new(0);
                        let predicate = |x: &$ty| {
                            calls.set(calls.get() + 1);
                            if increasing {
                                *x >= threshold
                            } else {
                                *x <= threshold
                            }
                        };
                        let (result, direction) = signed_binary_search(predicate, (lower, upper))?;
                        assert_eq!(result, threshold);
                        assert_eq!(direction, if increasing { 1 } else { -1 });
                        assert!(calls.get() <= $bits + 2);
                        let outside = if increasing {
                            result.next_down()
                        } else {
                            result.next_up()
                        };
                        assert!(!predicate(&outside));
                    }
                }
            }
            assert_eq!(<$ty as BinarySearchable>::midpoint(&-0.0, &0.0), 0.0);
        };
    }
    check!(f64, 64);
    check!(f32, 32);
    assert_eq!(binary_search(|x: &f64| *x > 0.0, ())?, f64::from_bits(1));
    assert_eq!(binary_search(|x: &f32| *x < 0.0, ())?, -f32::from_bits(1));
    Ok(())
}

#[test]
fn test_initial_invalid_segment_finds_boolean_bracket() -> Fallible<()> {
    for sign in [-1, 1] {
        for polarity in [false, true] {
            let predicate = |x: &i32| {
                let x = i64::from(*x) * sign;
                if !(3..=16).contains(&x) {
                    return fallible!(NumericError, "outside the evaluable interval");
                }
                Ok((x >= 5) == polarity)
            };
            let result = fallible_binary_search(predicate, ())?;
            assert_eq!(i64::from(result), sign * if polarity { 5 } else { 4 });
        }
    }
    Ok(())
}

#[test]
fn test_initial_recovery_tries_other_permitted_direction() -> Fallible<()> {
    let predicate = |x: &i32| {
        if *x == 0 {
            return fallible!(NumericError, "zero is unavailable");
        }
        Ok(*x <= -5)
    };
    assert_eq!(signed_fallible_binary_search(predicate, ())?, (-5, -1));
    assert_eq!(fallible_binary_search(predicate, Below(0))?, -5);
    let (lower, upper) = fallible_exponential_bounds_search(&predicate)?.unwrap();
    assert_ne!(predicate(&lower)?, predicate(&upper)?);
    let error = fallible_binary_search(
        |x: &i32| {
            assert!(*x >= 0, "crossed the one-sided lower bound");
            predicate(x)
        },
        Above(0),
    )
    .unwrap_err();
    assert_eq!(error.variant, ErrorVariant::Search);
    Ok(())
}

#[test]
fn test_one_sided_invalid_initial_endpoint() -> Fallible<()> {
    for sign in [-1, 1] {
        for polarity in [false, true] {
            let bounds = if sign > 0 {
                (Some(10), None)
            } else {
                (None, Some(-10))
            };
            let predicate = |x: &i32| {
                let x = i64::from(*x) * sign;
                assert!(x >= 10, "crossed the explicit bound");
                if x == 10 {
                    return fallible!(NumericError, "initial endpoint unavailable");
                }
                Ok((x >= 15) == polarity)
            };
            let result = fallible_binary_search(predicate, bounds)?;
            assert_eq!(i64::from(result), sign * if polarity { 15 } else { 14 });
        }
    }
    assert_eq!(
        fallible_binary_search(
            |x: &u32| {
                if *x == 0 {
                    return fallible!(NumericError, "positive size required");
                }
                Ok(*x >= 5)
            },
            Above(0)
        )?,
        5
    );
    Ok(())
}

#[test]
fn test_initial_discovery_failure_preserves_original_error() {
    for narrow in [false, true] {
        let error = fallible_binary_search(
            |x: &i32| {
                if narrow && *x == 7 {
                    return Ok(true);
                }
                if *x == 0 {
                    return fallible!(NumericError, "original error");
                }
                fallible!(FailedFunction, "later discovery error")
            },
            (),
        )
        .unwrap_err();
        assert_eq!(error.variant, ErrorVariant::NumericError);
        assert_eq!(error.message.as_deref(), Some("original error"));
    }
}

#[test]
fn test_initial_recovery_requires_opposing_boolean_observations() {
    let error = fallible_binary_search(
        |x: &i32| {
            if *x <= 0 {
                return fallible!(NumericError, "positive size required");
            }
            Ok(false)
        },
        (),
    )
    .unwrap_err();
    assert_eq!(error.variant, ErrorVariant::Search);
}

#[test]
fn test_ordered_golden_search_preserves_a_rbig_minimizer() -> Fallible<()> {
    let minimizer: f64 = 0.37;
    let offset = rbig!(100000000000000000000);

    assert_eq!(
        1e20 + (0.1 - minimizer).powi(2),
        1e20 + (0.9 - minimizer).powi(2)
    );

    let (lo, hi) = fallible_golden_section_bracket_by(
        SearchMode::Minimize,
        0.0,
        1.0,
        |x| {
            let distance = RBig::try_from(x)? - RBig::try_from(minimizer)?;
            Ok(offset.clone() + distance.clone() * distance)
        },
        |left: &RBig, right: &RBig| Ok(left.cmp(right)),
    )?;

    assert!(lo <= minimizer && minimizer <= hi);
    Ok(())
}

#[test]
fn test_golden_search_refreshes_rounded_coordinates_near_zero() -> Fallible<()> {
    let (lo, hi) = fallible_golden_section_bracket_by(
        SearchMode::Minimize,
        -1.0,
        1.0,
        |x| Ok(x.abs()),
        |left, right| Ok(left.total_cmp(right)),
    )?;
    assert!(lo <= 0.0 && 0.0 <= hi);
    assert!(lo.abs().max(hi.abs()) < 1e-300, "stopped at [{lo}, {hi}]");
    let candidate = fallible_optimize_unimodal(SearchMode::Minimize, -1.0, 1.0, |x| Ok(x.abs()))?;
    assert!(candidate.value < 1e-300);
    Ok(())
}

#[test]
fn test_golden_coordinates_collapse_before_exclusion() -> Fallible<()> {
    // In particular, a two-ULP interval has c == d strictly inside its bounds.
    // Even exact comparisons at that single point cannot select a side.
    for base in [0.0, 1.0, -1.0, f64::MIN_POSITIVE, -f64::MIN_POSITIVE] {
        for steps in 1..=8 {
            let mut hi = base;
            for _ in 0..steps {
                hi = hi.next_up();
            }
            let mut optimum = base;
            for _ in 0..=steps {
                for mode in [SearchMode::Minimize, SearchMode::Maximize] {
                    let (lo_out, hi_out) = fallible_golden_section_bracket_by(
                        mode,
                        base,
                        hi,
                        |x| {
                            let distance = RBig::try_from(x)? - RBig::try_from(optimum)?;
                            let square = distance.clone() * distance;
                            Ok(if mode == SearchMode::Minimize {
                                square
                            } else {
                                -square
                            })
                        },
                        |left: &RBig, right: &RBig| Ok(left.cmp(right)),
                    )?;
                    // Exact quadratics establish strict unimodality and
                    // reliable ordering, unlike arbitrary callbacks.
                    assert!(
                        lo_out <= optimum && optimum <= hi_out,
                        "lost optimum {optimum:?} from [{base:?}, {hi:?}] -> [{lo_out:?}, {hi_out:?}]"
                    );
                }
                optimum = optimum.next_up();
            }
        }
    }
    Ok(())
}

#[test]
fn test_golden_extreme_intervals_and_flat_candidates() -> Fallible<()> {
    for (lo, hi, optimum) in [
        (-f64::MAX, f64::MAX, 0.0),
        (-f64::MAX, 0.0, -f64::MAX),
        (0.0, f64::MAX, f64::MAX),
    ] {
        let (left, right) = fallible_golden_section_bracket_by(
            SearchMode::Minimize,
            lo,
            hi,
            |x| {
                let distance = RBig::try_from(x)? - RBig::try_from(optimum)?;
                Ok(distance.clone() * distance)
            },
            |left: &RBig, right: &RBig| Ok(left.cmp(right)),
        )?;
        assert!(left <= optimum && optimum <= right);
    }
    for (lo, hi) in [(-f64::MAX, f64::MAX), (1.0, 1.0_f64.next_up()), (0.0, 1.0)] {
        for mode in [SearchMode::Minimize, SearchMode::Maximize] {
            let candidate = fallible_optimize_unimodal(mode, lo, hi, |_| Ok(7.0))?;
            assert!(lo <= candidate.arg && candidate.arg <= hi);
            assert_eq!(candidate.value, 7.0);
        }
    }
    Ok(())
}

#[test]
fn test_ordered_golden_search_propagates_errors() {
    let callback_error = fallible_golden_section_bracket_by(
        SearchMode::Minimize,
        0.0,
        1.0,
        |_| fallible!(FailedFunction, "objective failed"),
        |_: &(), _: &()| Ok(Ordering::Equal),
    )
    .unwrap_err();
    assert_eq!(callback_error.variant, ErrorVariant::FailedFunction);

    let comparison_error = fallible_golden_section_bracket_by(
        SearchMode::Minimize,
        0.0,
        1.0,
        |_| Ok(()),
        |_: &(), _: &()| fallible!(FailedFunction, "comparison failed"),
    )
    .unwrap_err();
    assert_eq!(comparison_error.variant, ErrorVariant::FailedFunction);
}

#[test]
fn test_fallible_scalar_optimization() -> Fallible<()> {
    let minimum =
        fallible_optimize_unimodal(SearchMode::Minimize, -10.0, 10.0, |x| Ok((x - 2.0).powi(2)))?;
    assert!((minimum.arg - 2.0).abs() <= 4.0 * f64::EPSILON);

    let maximum =
        fallible_optimize_unimodal(
            SearchMode::Maximize,
            -10.0,
            10.0,
            |x| Ok(-(x + 3.0).powi(2)),
        )?;
    assert!((maximum.arg + 3.0).abs() <= 8.0 * f64::EPSILON);

    let boundary = fallible_optimize_unimodal(SearchMode::Minimize, -2.0, 3.0, |x| Ok(x + 2.0))?;
    assert_eq!(boundary.arg, -2.0);

    let degenerate = fallible_optimize_unimodal(SearchMode::Minimize, 3.0, 3.0, |x| Ok(x * x))?;
    assert_eq!(degenerate.arg, 3.0);
    assert_eq!(degenerate.value, 9.0);
    Ok(())
}

#[test]
fn test_scalar_optimization_rejects_malformed_bounds_and_nan() {
    for (lo, hi) in [(2.0, 1.0), (f64::NEG_INFINITY, 1.0), (1.0, f64::INFINITY)] {
        let error =
            fallible_optimize_unimodal(SearchMode::Minimize, lo, hi, |_| Ok(0.0)).unwrap_err();
        assert_eq!(error.variant, ErrorVariant::Search);
    }

    let error =
        fallible_optimize_unimodal(SearchMode::Minimize, -1.0, 1.0, |_| Ok(f64::NAN)).unwrap_err();
    assert_eq!(error.variant, ErrorVariant::NumericError);
}

#[test]
fn test_ordered_golden_search_rejects_malformed_bounds_and_supports_degenerate() -> Fallible<()> {
    for (lo, hi) in [(2.0, 1.0), (f64::NEG_INFINITY, 1.0), (1.0, f64::INFINITY)] {
        let error = fallible_golden_section_bracket_by(
            SearchMode::Minimize,
            lo,
            hi,
            |_| Ok(()),
            |_: &(), _: &()| Ok(Ordering::Equal),
        )
        .unwrap_err();
        assert_eq!(error.variant, ErrorVariant::Search);
    }

    assert_eq!(
        fallible_golden_section_bracket_by(
            SearchMode::Maximize,
            3.0,
            3.0,
            |x| Ok(x),
            |left, right| left.partial_cmp(right).ok_or_else(|| err!(Search, "NaN")),
        )?,
        (3.0, 3.0)
    );
    Ok(())
}
