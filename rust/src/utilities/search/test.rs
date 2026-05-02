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
fn test_signed_binary_search_reports_direction() -> Fallible<()> {
    assert_eq!(signed_binary_search(|x: &i32| *x >= 5, (0, 10))?, (5, 1));
    assert_eq!(signed_binary_search(|x: &i32| *x <= 5, (0, 10))?, (5, -1));
    Ok(())
}

#[test]
fn test_exponential_bounds_search_bands() {
    assert_eq!(
        exponential_bounds_search::<i32>(&|x| *x > 5),
        Some((1, 65_536))
    );
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
