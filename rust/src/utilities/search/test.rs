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
