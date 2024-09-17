use super::*;

#[test]
fn test_make_bounded_int_split_sum() -> Fallible<()> {
    let trans = make_bounded_int_split_sum((1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    let trans = make_bounded_int_split_sum((1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    // test saturation arithmetic
    let trans = make_bounded_int_split_sum((1i8, 127))?;
    let sum = trans.invoke(&vec![-128, 127, 127])?;
    // negative sum is -128, positive sum is 127; they cancel to -1
    assert_eq!(sum, -1);

    Ok(())
}

#[test]
fn test_make_sized_bounded_int_split_sum() -> Fallible<()> {
    let trans = make_sized_bounded_int_split_sum(4, (1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    let trans = make_sized_bounded_int_split_sum(4, (1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    Ok(())
}
