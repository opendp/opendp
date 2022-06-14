use super::*;

#[test]
fn test_binary_search_matches_python_r_int_behavior() -> Fallible<()> {
    assert_eq!(binary_search::<i32>(|x| *x <= -5, None)?, -5);
    assert_eq!(binary_search::<i32>(|x| *x <= 5, None)?, 5);
    assert_eq!(binary_search::<i32>(|x| *x >= -5, None)?, -5);
    assert_eq!(binary_search::<i32>(|x| *x >= 5, None)?, 5);
    assert_eq!(binary_search::<u32>(|x| *x > 5, None)?, 6);
    Ok(())
}

#[test]
fn test_binary_search_sorts_bounds() -> Fallible<()> {
    assert_eq!(binary_search(|x: &i32| *x > 5, Some((10, 0)))?, 6);
    assert_eq!(binary_search(|x: &i32| *x < 5, Some((10, 0)))?, 4);
    Ok(())
}

#[test]
fn test_signed_binary_search_reports_direction() -> Fallible<()> {
    assert_eq!(
        signed_binary_search(|x: &i32| *x >= 5, Some((0, 10)))?,
        (5, 1)
    );
    assert_eq!(
        signed_binary_search(|x: &i32| *x <= 5, Some((0, 10)))?,
        (5, -1)
    );
    Ok(())
}

#[test]
fn test_exponential_bounds_search_matches_python_r_bands() {
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
        None,
    )?;

    assert_eq!(discovered, 5);
    Ok(())
}

#[test]
fn test_binary_search_handles_full_signed_ranges() -> Fallible<()> {
    assert_eq!(
        binary_search(|x: &i32| *x >= 0, Some((i32::MIN, i32::MAX)))?,
        0
    );
    assert_eq!(
        binary_search(|x: &i8| *x <= 0, Some((i8::MIN, i8::MAX)))?,
        0
    );
    Ok(())
}
