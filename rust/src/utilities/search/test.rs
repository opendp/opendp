use std::cmp::Ordering;

use dashu::{rational::RBig, rbig};

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
fn test_fallible_exponential_bounds_search_propagates_callback_errors() {
    let error = fallible_exponential_bounds_search::<i32>(&|x| {
        if *x == 16 {
            fallible!(FailedFunction, "boom happened")
        } else {
            Ok(false)
        }
    })
    .unwrap_err();

    assert_eq!(error.variant, ErrorVariant::FailedFunction);
    assert_eq!(error.message.as_deref(), Some("boom happened"));
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
    assert_eq!(
        fallible_binary_search_by(|x: &i32| Ok(x.cmp(&5)), Above(0))?,
        5
    );
    assert_eq!(
        fallible_binary_search_by(|x: &i32| Ok(x.cmp(&-5)), Below(0))?,
        -5
    );
    Ok(())
}

#[test]
fn test_fallible_binary_search_by_with_range_errors_during_bound_discovery() -> Fallible<()> {
    let increasing = fallible_binary_search_by_with_range_errors(
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

    let decreasing = fallible_binary_search_by_with_range_errors(
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
fn test_fallible_binary_search_by_propagates_operation_range_errors() {
    let error = fallible_binary_search_by(
        |x: &i32| {
            if *x == 5 {
                fallible!(NumericRangeBelow, "intermediate arithmetic overflowed")
            } else {
                Ok(x.cmp(&5))
            }
        },
        (0, 10),
    )
    .unwrap_err();

    assert_eq!(error.variant, ErrorVariant::NumericRangeBelow);
}

#[test]
fn test_fallible_binary_search_by_with_range_errors_increasing_range_regions() -> Fallible<()> {
    let result = fallible_binary_search_by_with_range_errors(
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
fn test_fallible_binary_search_by_with_range_errors_decreasing_range_regions() -> Fallible<()> {
    let result = fallible_binary_search_by_with_range_errors(
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
    assert_eq!(below.variant, ErrorVariant::NumericRangeBelow);

    let above =
        fallible_binary_search_by::<i32>(|_| fallible!(NumericRangeAbove, "always above"), ())
            .unwrap_err();
    assert_eq!(above.variant, ErrorVariant::NumericRangeAbove);
}

#[test]
fn test_fallible_binary_search_by_propagates_range_variants() -> Fallible<()> {
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
fn test_ordered_golden_search_preserves_a_rbig_minimizer() -> Fallible<()> {
    let minimizer: f64 = 0.37;
    let offset = rbig!(100000000000000000000);

    assert_eq!(
        1e20 + (0.1 - minimizer).powi(2),
        1e20 + (0.9 - minimizer).powi(2)
    );

    let (lo, hi) = fallible_golden_search_to_precision_ordered(
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
fn test_ordered_golden_search_propagates_errors() {
    let callback_error = fallible_golden_search_to_precision_ordered(
        SearchMode::Minimize,
        0.0,
        1.0,
        |_| fallible!(FailedFunction, "objective failed"),
        |_: &(), _: &()| Ok(Ordering::Equal),
    )
    .unwrap_err();
    assert_eq!(callback_error.variant, ErrorVariant::FailedFunction);

    let comparison_error = fallible_golden_search_to_precision_ordered(
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
    let minimum = fallible_optimize_to_precision(SearchMode::Minimize, -10.0, 10.0, |x| {
        Ok((x - 2.0).powi(2))
    })?;
    assert!((minimum.arg - 2.0).abs() <= 4.0 * f64::EPSILON);

    let maximum = fallible_optimize_to_precision(SearchMode::Maximize, -10.0, 10.0, |x| {
        Ok(-(x + 3.0).powi(2))
    })?;
    assert!((maximum.arg + 3.0).abs() <= 8.0 * f64::EPSILON);

    let boundary =
        fallible_optimize_to_precision(SearchMode::Minimize, -2.0, 3.0, |x| Ok(x + 2.0))?;
    assert_eq!(boundary.arg, -2.0);

    let degenerate = fallible_optimize_to_precision(SearchMode::Minimize, 3.0, 3.0, |x| Ok(x * x))?;
    assert_eq!(degenerate.arg, 3.0);
    assert_eq!(degenerate.value, 9.0);
    Ok(())
}

#[test]
fn test_scalar_optimization_rejects_malformed_bounds_and_nan() {
    for (lo, hi) in [(2.0, 1.0), (f64::NEG_INFINITY, 1.0), (1.0, f64::INFINITY)] {
        let error =
            fallible_optimize_to_precision(SearchMode::Minimize, lo, hi, |_| Ok(0.0)).unwrap_err();
        assert_eq!(error.variant, ErrorVariant::Search);
    }

    let error = fallible_optimize_to_precision(SearchMode::Minimize, -1.0, 1.0, |_| Ok(f64::NAN))
        .unwrap_err();
    assert_eq!(error.variant, ErrorVariant::Search);
}

#[test]
fn test_ordered_golden_search_rejects_malformed_bounds_and_supports_degenerate() -> Fallible<()> {
    for (lo, hi) in [(2.0, 1.0), (f64::NEG_INFINITY, 1.0), (1.0, f64::INFINITY)] {
        let error = fallible_golden_search_to_precision_ordered(
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
        fallible_golden_search_to_precision_ordered(
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
