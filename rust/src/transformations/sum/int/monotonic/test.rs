use super::*;

#[test]
fn test_make_bounded_int_monotonic_sum() -> Fallible<()> {
    let trans = make_bounded_int_monotonic_sum((1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    let trans = make_bounded_int_monotonic_sum((1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    // should fail under these conditions
    assert!(make_bounded_int_monotonic_sum((-1i32, 1)).is_err());

    Ok(())
}

#[test]
fn test_make_sized_bounded_int_monotonic_sum() -> Fallible<()> {
    let trans = make_sized_bounded_int_monotonic_sum(4, (1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    let trans = make_sized_bounded_int_monotonic_sum(4, (1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    Ok(())
}
