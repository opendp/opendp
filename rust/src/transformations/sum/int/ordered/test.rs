use super::*;

#[test]
fn test_make_bounded_int_ordered_sum() -> Fallible<()> {
    let trans = make_bounded_int_ordered_sum((1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    let trans = make_bounded_int_ordered_sum((1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    // test saturation arithmetic
    let trans = make_bounded_int_ordered_sum((1i8, 127))?;
    let sum = trans.invoke(&vec![-128, -128, 127, 127, 127])?;
    assert_eq!(sum, 127);

    Ok(())
}

#[test]
fn test_make_sized_bounded_int_ordered_sum() -> Fallible<()> {
    let trans = make_sized_bounded_int_ordered_sum(4, (1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    let trans = make_sized_bounded_int_ordered_sum(4, (1i32, 10))?;
    let sum = trans.invoke(&vec![1, 2, 3, 4])?;
    assert_eq!(sum, 10);

    Ok(())
}
