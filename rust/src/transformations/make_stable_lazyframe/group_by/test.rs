use super::*;

#[test]
fn test_check_infallible() -> Fallible<()> {
    // failure to cast causes a data-dependent error
    assert!(check_infallible(&lit("a").strict_cast(DataType::Int32), Resize::Allow).is_err());

    Ok(())
}

#[test]
fn test_check_infallible_resize() -> Fallible<()> {
    // col doesn't resize, so passes the ban
    assert!(check_infallible(&col("A"), Resize::Ban).is_ok());
    // sum results in a broadcastable scalar, so it passes the ban
    assert!(check_infallible(&col("A").sum(), Resize::Ban).is_ok());
    // unique resizes, so fails the ban
    assert!(check_infallible(&col("A").unique(), Resize::Ban).is_err());
    // resizing behind an aggregation is allowed, though, because it can broadcast
    assert!(check_infallible(&col("A").unique().sum(), Resize::Ban).is_ok());
    // while resizing is generally allowed, binary ops ban resizing
    assert!(check_infallible(&(col("A").unique() + col("B")), Resize::Allow).is_err());
    // the sum results in a broadcastable scalar, so it passes the binary op resize ban
    assert!(check_infallible(&(col("A").unique().sum() + col("B")), Resize::Ban).is_ok());

    Ok(())
}
