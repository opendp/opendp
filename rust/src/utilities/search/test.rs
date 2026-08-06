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
        ($ty:ty, $thresholds:expr, $limit:expr) => {
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
                        assert!(predicate(&result));
                        assert_eq!((result, direction), (threshold, sign as i8));
                        assert!(calls.get() < $limit);
                    }
                }
            }
        };
    }
    check!(f64, [1e-300, 1e100, 1e300, f64::MAX], 2300);
    check!(f32, [1e-30, 1e30, f32::MAX], 350);
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
                            variant: variant.parse()?,
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
    for threshold in [0.0, -0.0, f64::from_bits(1), -f64::from_bits(1)] {
        for increasing in [false, true] {
            let predicate = |x: &f64| {
                if increasing {
                    *x >= threshold
                } else {
                    *x <= threshold
                }
            };
            let result = binary_search(predicate, (-1.0, 1.0))?;
            assert!(predicate(&result));
            assert_eq!(result, threshold);
        }
    }
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
