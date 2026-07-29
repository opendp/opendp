use super::*;
use crate::error::ErrorVariant;

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
    assert_eq!(signed_binary_search(|x: &i32| *x >= 5, (0, 10))?, (5, 1));
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
fn test_fallible_binary_search_recovers_from_exception_boundary() -> Fallible<()> {
    let discovered = fallible_binary_search::<i32>(
        |x| {
            if *x <= 0 {
                return fallible!(FailedFunction, "x must be positive");
            }
            Ok(*x >= 5)
        },
        (),
    )?;

    assert_eq!(discovered, 5);
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
fn test_fallible_binary_search_by_increasing_and_decreasing() -> Fallible<()> {
    let increasing = |x: &i32| Ok(x.cmp(&5));
    let decreasing = |x: &i32| Ok(5.cmp(x));

    assert_eq!(fallible_binary_search_by(increasing, ())?, 5);
    assert_eq!(fallible_binary_search_by(decreasing, ())?, 5);
    assert_eq!(
        fallible_binary_search_by(|x: &i32| Ok(x.cmp(&5)), (0, 10))?,
        5
    );
    assert_eq!(
        fallible_binary_search_by(|x: &i32| Ok(5.cmp(x)), (0, 10))?,
        5
    );
    Ok(())
}

#[test]
fn test_fallible_binary_search_by_non_exact_boundary() -> Fallible<()> {
    let increasing = |x: &i32| Ok((*x as f64).partial_cmp(&5.5).unwrap());
    let decreasing = |x: &i32| Ok(5.5.partial_cmp(&(*x as f64)).unwrap());

    // The Less-side rule returns the lower bracket for increasing comparators
    // and the upper bracket for decreasing comparators.
    assert_eq!(fallible_binary_search_by(increasing, (0, 10))?, 5);
    assert_eq!(fallible_binary_search_by(decreasing, (0, 10))?, 6);
    Ok(())
}

#[test]
fn test_fallible_binary_search_by_explicit_and_inferred_bounds() -> Fallible<()> {
    assert_eq!(<f32 as Bands>::bands(0.0, 1).last(), Some(&f32::MAX));
    assert_eq!(<f64 as Bands>::bands(0.0, -1).last(), Some(&-f64::MAX));

    assert_eq!(
        fallible_binary_search_by(|x: &i32| Ok(x.cmp(&5)), Above(0))?,
        5
    );
    assert_eq!(
        fallible_binary_search_by(|x: &i32| Ok(x.cmp(&-5)), Below(0))?,
        -5
    );

    let positive_target = f32::MAX * 0.999;
    assert_eq!(
        fallible_binary_search_by(|x: &f32| Ok(x.partial_cmp(&positive_target).unwrap()), (),)?,
        positive_target
    );

    let negative_target = -f32::MAX * 0.999;
    assert_eq!(
        fallible_binary_search_by(|x: &f32| Ok(x.partial_cmp(&negative_target).unwrap()), (),)?,
        negative_target
    );
    Ok(())
}

#[test]
fn test_fallible_binary_search_by_range_errors_during_bound_discovery() -> Fallible<()> {
    let increasing = fallible_binary_search_by(
        |x: &i32| {
            if *x < 1 {
                fallible!(NumericRangeBelow, "value is below the representable range")
            } else {
                Ok(x.cmp(&5))
            }
        },
        (),
    )?;
    assert_eq!(increasing, 5);

    let decreasing = fallible_binary_search_by(
        |x: &i32| {
            if *x > 9 {
                fallible!(NumericRangeBelow, "value is below the target range")
            } else {
                Ok(5.cmp(x))
            }
        },
        (),
    )?;
    assert_eq!(decreasing, 5);
    Ok(())
}

#[test]
fn test_fallible_binary_search_by_propagates_bound_discovery_errors() {
    let error = fallible_binary_search_by::<i32>(
        |x| {
            if *x == 0 {
                fallible!(FailedFunction, "bound discovery failed")
            } else {
                Ok(x.cmp(&5))
            }
        },
        (),
    )
    .unwrap_err();

    assert_eq!(error.variant, ErrorVariant::FailedFunction);
    assert_eq!(error.message.as_deref(), Some("bound discovery failed"));
}

#[test]
fn test_fallible_binary_search_by_increasing_range_regions() -> Fallible<()> {
    let result = fallible_binary_search_by(
        |x: &i32| {
            if *x < 0 {
                fallible!(NumericRangeBelow, "final value is below target")
            } else if *x > 10 {
                fallible!(NumericRangeAbove, "final value is above target")
            } else {
                Ok(x.cmp(&5))
            }
        },
        (-10, 20),
    )?;
    assert_eq!(result, 5);
    Ok(())
}

#[test]
fn test_fallible_binary_search_by_decreasing_range_regions() -> Fallible<()> {
    let result = fallible_binary_search_by(
        |x: &i32| {
            if *x < 0 {
                fallible!(NumericRangeAbove, "final value is above target")
            } else if *x > 10 {
                fallible!(NumericRangeBelow, "final value is below target")
            } else {
                Ok(5.cmp(x))
            }
        },
        (-10, 20),
    )?;
    assert_eq!(result, 5);
    Ok(())
}

#[test]
fn test_fallible_binary_search_by_constant_range_errors() {
    let below =
        fallible_binary_search_by::<i32>(|_| fallible!(NumericRangeBelow, "always below"), ())
            .unwrap_err();
    assert_eq!(below.variant, ErrorVariant::Search);

    let above =
        fallible_binary_search_by::<i32>(|_| fallible!(NumericRangeAbove, "always above"), ())
            .unwrap_err();
    assert_eq!(above.variant, ErrorVariant::Search);
}

#[test]
fn test_fallible_binary_search_by_consumes_only_range_variants() -> Fallible<()> {
    macro_rules! assert_propagates {
        ($variant:ident) => {
            let error =
                fallible_binary_search_by(|_| fallible!($variant, "callback failure"), (0, 10))
                    .unwrap_err();
            assert_eq!(error.variant, ErrorVariant::$variant);
        };
    }
    assert_propagates!(NumericDomain);
    assert_propagates!(NumericIndeterminate);
    assert_propagates!(NumericBackend);
    Ok(())
}

#[test]
fn test_scalar_optimization() {
    let minimum = optimize_to_precision(SearchMode::Minimize, -10.0, 10.0, None, |x| {
        (x - 2.0).powi(2)
    });
    assert!((minimum.arg - 2.0).abs() <= 4.0 * f64::EPSILON);

    let maximum = optimize_to_precision(SearchMode::Maximize, -10.0, 10.0, Some(33), |x| {
        -(x + 3.0).powi(2)
    });
    assert!((maximum.arg + 3.0).abs() <= 8.0 * f64::EPSILON);
}

#[test]
fn test_fallible_scalar_optimization() -> Fallible<()> {
    let minimum = fallible_optimize_to_precision(SearchMode::Minimize, -10.0, 10.0, None, |x| {
        Ok((x - 2.0).powi(2))
    })?;
    assert!((minimum.arg - 2.0).abs() <= 4.0 * f64::EPSILON);

    let maximum =
        fallible_optimize_to_precision_bracket(SearchMode::Maximize, -10.0, 10.0, Some(33), |x| {
            Ok(-(x + 3.0).powi(2))
        })?;
    assert!((maximum.arg + 3.0).abs() <= 8.0 * f64::EPSILON);
    assert!(maximum.lo <= maximum.arg && maximum.arg <= maximum.hi);

    let log_minimum =
        fallible_optimize_log_domain_to_precision(SearchMode::Minimize, 1e-6, 1e6, None, |x| {
            Ok(x.ln().powi(2))
        })?;
    assert!((log_minimum.arg - 1.0).abs() <= 4.0 * f64::EPSILON);
    Ok(())
}

#[test]
fn test_fallible_scalar_optimization_returns_first_error_immediately() {
    use std::cell::Cell;

    let calls = Cell::new(0);
    let error = fallible_optimize_to_precision(SearchMode::Minimize, -1.0, 1.0, None, |_| {
        calls.set(calls.get() + 1);
        fallible!(FailedFunction, "ordinary objective failed")
    })
    .unwrap_err();
    assert_eq!(calls.get(), 1);
    assert_eq!(error.message.as_deref(), Some("ordinary objective failed"));

    let calls = Cell::new(0);
    let error =
        fallible_optimize_to_precision_bracket(SearchMode::Maximize, -1.0, 1.0, Some(17), |_| {
            calls.set(calls.get() + 1);
            fallible!(FailedFunction, "bracket objective failed")
        })
        .unwrap_err();
    assert_eq!(calls.get(), 1);
    assert_eq!(error.message.as_deref(), Some("bracket objective failed"));

    let calls = Cell::new(0);
    let error =
        fallible_optimize_log_domain_to_precision(SearchMode::Minimize, 1e-6, 1e6, None, |_| {
            calls.set(calls.get() + 1);
            fallible!(FailedFunction, "log objective failed")
        })
        .unwrap_err();
    assert_eq!(calls.get(), 1);
    assert_eq!(error.message.as_deref(), Some("log objective failed"));
}

#[test]
fn test_scalar_optimization_handles_boundaries_nonfinite_values_and_wide_ranges() {
    let boundary = optimize_to_precision(SearchMode::Minimize, -2.0, 3.0, None, |x| x + 2.0);
    assert_eq!(boundary.arg, -2.0);
    assert_eq!(boundary.value, 0.0);

    let with_nan = optimize_to_precision(SearchMode::Maximize, -2.0, 2.0, Some(17), |x| {
        if x < 0.0 { f64::NAN } else { -(x - 1.0).abs() }
    });
    assert!((with_nan.arg - 1.0).abs() <= 4.0 * f64::EPSILON);
    assert_eq!(with_nan.value, 0.0);

    let wide = optimize_to_precision(SearchMode::Minimize, -f64::MAX, f64::MAX, None, f64::abs);
    assert!(wide.arg.is_finite());
    assert!(wide.value < f64::MAX);
}

#[test]
fn test_scalar_optimization_refines_multiple_local_extrema() {
    let optimum = optimize_to_precision(SearchMode::Minimize, -4.0, 4.0, Some(65), |x| {
        ((x + 2.05).powi(2) + 1.0).min((x - 1.1).powi(2))
    });
    assert!((optimum.arg - 1.1).abs() <= 8.0 * f64::EPSILON);
    assert!(optimum.value <= f64::EPSILON);
}

#[test]
fn test_scalar_optimization_does_not_refine_flat_grid() {
    use std::cell::Cell;

    let calls = Cell::new(0);
    let optimum = optimize_to_precision(SearchMode::Maximize, -4.0, 4.0, Some(65), |_| {
        calls.set(calls.get() + 1);
        1.0
    });

    assert_eq!(optimum.value, 1.0);
    // One initial evaluation plus the grid, with no golden searches.
    assert_eq!(calls.get(), 66);
}

#[test]
fn test_log_domain_optimization_and_sampling() {
    let optimum =
        optimize_log_domain_to_precision(SearchMode::Minimize, 1e-6, 1e6, None, |x| x.ln().powi(2));
    assert!((optimum.arg - 1.0).abs() <= 4.0 * f64::EPSILON);

    let sampled = sample_log_domain(SearchMode::Maximize, 1e-3, 1e3, 7, |x| -x.ln().abs());
    assert!((sampled.arg - 1.0).abs() <= f64::EPSILON);
}

#[test]
fn test_log_domain_search_stays_within_extreme_bounds() {
    let lo = 1e-200;
    let hi = 1e-100;

    let optimum = optimize_log_domain_to_precision(SearchMode::Minimize, lo, hi, None, |x| x);
    assert!((lo..=hi).contains(&optimum.arg));
    assert_eq!(optimum.value, optimum.arg);

    let sampled = sample_log_domain(SearchMode::Minimize, lo, hi, 17, |x| x);
    assert!((lo..=hi).contains(&sampled.arg));
    assert_eq!(sampled.value, sampled.arg);
}

#[test]
fn test_bracketed_optimization_invariants() {
    let optimum = optimize_to_precision_bracket(SearchMode::Maximize, -10.0, 10.0, None, |x| {
        -(x - 0.25).powi(2)
    });
    assert!(optimum.lo <= optimum.arg);
    assert!(optimum.arg <= optimum.hi);
    assert_eq!(optimum.value, -(optimum.arg - 0.25).powi(2));

    let log_optimum =
        optimize_log_domain_to_precision_bracket(SearchMode::Minimize, 1e-200, 1e-100, None, |x| x);
    assert!(log_optimum.lo <= log_optimum.arg);
    assert!(log_optimum.arg <= log_optimum.hi);
    assert_eq!(log_optimum.value, log_optimum.arg);
}
