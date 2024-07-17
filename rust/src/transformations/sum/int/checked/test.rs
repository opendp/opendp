use super::*;

#[test]
fn test_make_sized_bounded_int_checked_sum() -> Fallible<()> {
    let trans = make_sized_bounded_int_checked_sum(4, (1, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    // should error under these conditions
    assert!(make_sized_bounded_int_checked_sum::<u8>(2, (0, 255)).is_err());
    Ok(())
}
