use super::*;

#[test]
fn test_error_variant_from_str() {
    assert_eq!("FFI".parse(), Ok(ErrorVariant::FFI));
    assert_eq!("Overflow".parse(), Ok(ErrorVariant::Overflow));

    assert_eq!("Unknown".parse::<ErrorVariant>(), Err(()));
}

#[test]
fn test_error_from_conversion_error() {
    let e: Error = ConversionError::OutOfBounds.into();
    assert_eq!(e.variant, ErrorVariant::FailedCast);
}

#[cfg(feature = "polars")]
#[test]
fn test_error_from_polars_error() {
    let e: Error = PolarsError::ColumnNotFound("A".into()).into();
    assert_eq!(e.variant, ErrorVariant::FailedFunction);
    assert_eq!(
        e.message,
        Some("ColumnNotFound(ErrString(\"A\"))".to_string())
    );
}
