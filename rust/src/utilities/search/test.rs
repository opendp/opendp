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
fn test_binary_search_uses_search_error_variant() {
    let err = binary_search(|x: &i32| *x < 0, (0, 10)).unwrap_err();
    assert_eq!(err.variant, ErrorVariant::Search);
    assert_eq!(
        err.message.as_deref(),
        Some("the decision boundary of the predicate is outside the bounds")
    );
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
