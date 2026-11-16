use polars::chunked_array::cast::CastOptions;
use polars::prelude::{DataType, TimeUnit};

use crate::error::*;

// What a cast can do to private values, assuming the source domain is valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CastEffect {
    NullPreserving,
    MayIntroduceNull,
}

// Reject casts whose failures or mutable encodings can depend on private data.
// Strict casts must be null-preserving, whereas non-strict casts may add nulls.
pub(super) fn classify_private_cast(
    from: &DataType,
    to: &DataType,
    options: CastOptions,
) -> Fallible<CastEffect> {
    if matches!(from, DataType::Array(_, _)) || matches!(to, DataType::Array(_, _)) {
        return fallible!(MakeTransformation, "Array casts are not supported");
    }
    if !is_safe_categorical_cast(from, to) {
        return fallible!(
            MakeTransformation,
            "Casting to or from mutable Categorical encodings is unsafe. Cast to String or fixed Enum instead."
        );
    }
    if has_unsafe_temporal_cast(from, to) {
        return fallible!(
            MakeTransformation,
            "This temporal cast can produce data-dependent failures on private values."
        );
    }

    if is_null_preserving_cast(from, to) {
        return Ok(CastEffect::NullPreserving);
    }
    if matches!(options, CastOptions::Strict) {
        return fallible!(
            MakeTransformation,
            "strict cast from {} to {} may fail on private values",
            from,
            to
        );
    }
    Ok(CastEffect::MayIntroduceNull)
}

pub(super) fn is_numeric_or_boolean(dtype: &DataType) -> bool {
    dtype.is_primitive_numeric() || matches!(dtype, DataType::Boolean)
}


// Integer casts that preserve every value and do not increase Lp distances.
pub(super) fn is_integer_widening_cast(from: &DataType, to: &DataType) -> bool {
    let (Some((in_min, in_max)), Some((out_min, out_max))) =
        (integer_bounds(from), integer_bounds(to))
    else {
        return false;
    };

    out_min <= in_min && in_max <= out_max
}

// A non-null input must never become null; numeric precision need not be preserved.
pub(super) fn is_null_preserving_cast(from: &DataType, to: &DataType) -> bool {
    use DataType::*;

    from == to
        || is_integer_widening_cast(from, to)
        || matches!(
            (from, to),
            (
                DataType::Categorical(_, _) | DataType::Enum(_, _),
                DataType::String
            )
        )
        || matches!(
            (from, to),
            (Float32, Float64)
                | (Int8 | Int16 | Int32 | Int64 | UInt32 | UInt64, Float64)
                | (Int8 | Int16 | Int32 | Int64 | UInt32 | UInt64, String)
                | (Float32 | Float64 | Boolean, String)
        )
}

// A temporal operand can trigger Polars' implicit type-coercion rules.
pub(super) fn is_temporal_dtype(dtype: &DataType) -> bool {
    matches!(
        dtype,
        DataType::Date | DataType::Datetime(_, _) | DataType::Time
    )
}

/// Mutable categorical mappings must not be created or decoded as physical
/// codes. Logical String and fixed Enum conversions are safe.
pub(super) fn is_safe_categorical_cast(from: &DataType, to: &DataType) -> bool {
    match (from, to) {
        (DataType::Categorical(..), DataType::String | DataType::Enum(..)) => true,
        (DataType::Categorical(..), _) | (_, DataType::Categorical(..)) => from == to,
        _ => true,
    }
}

pub(super) fn has_unsafe_temporal_cast(from: &DataType, to: &DataType) -> bool {
    use DataType::*;

    match (from, to) {
        (String, Datetime(TimeUnit::Nanoseconds, _)) => true,
        (Datetime(TimeUnit::Milliseconds | TimeUnit::Microseconds, _), String) => true,
        (from, Date) if from.is_primitive_numeric() => true,
        // Millisecond/microsecond timestamps can produce invalid Date values.
        (Datetime(unit, _), Date) => !matches!(unit, TimeUnit::Nanoseconds),
        _ => false,
    }
}

fn integer_bounds(dtype: &DataType) -> Option<(i128, i128)> {
    use DataType::*;

    Some(match dtype {
        UInt32 => (0, u32::MAX.into()),
        UInt64 => (0, u64::MAX.into()),
        Int8 => (i8::MIN.into(), i8::MAX.into()),
        Int16 => (i16::MIN.into(), i16::MAX.into()),
        Int32 => (i32::MIN.into(), i32::MAX.into()),
        Int64 => (i64::MIN.into(), i64::MAX.into()),
        _ => return None,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_integer_widening_cast() {
        use DataType::*;

        for (from, to, expected) in [
            (Int32, Int64, true),
            (UInt32, Int64, true),
            (UInt32, UInt64, true),
            (UInt64, UInt64, true),
            (UInt64, Int64, false),
            (Int64, UInt64, false),
            (Int64, Int32, false),
            (Float64, Int64, false),
            (String, Int64, false),
        ] {
            assert_eq!(
                is_integer_widening_cast(&from, &to),
                expected,
                "{from:?} -> {to:?}"
            );
        }
    }

    #[test]
    fn test_private_cast_classification() -> Fallible<()> {
        use CastEffect::*;

        for (from, to, expected) in [
            (DataType::Int32, DataType::Int64, NullPreserving),
            (DataType::Int64, DataType::Float64, NullPreserving),
            (DataType::String, DataType::Int64, MayIntroduceNull),
            (
                DataType::Date,
                DataType::Datetime(TimeUnit::Nanoseconds, None),
                MayIntroduceNull,
            ),
        ] {
            assert_eq!(
                classify_private_cast(&from, &to, CastOptions::NonStrict)?,
                expected
            );
        }

        assert!(classify_private_cast(
            &DataType::String,
            &DataType::Int64,
            CastOptions::Strict,
        )
        .is_err());
        for (from, to) in [
            (
                DataType::String,
                DataType::Datetime(TimeUnit::Nanoseconds, None),
            ),
            (
                DataType::Datetime(TimeUnit::Microseconds, None),
                DataType::String,
            ),
            (DataType::Int32, DataType::Date),
            (
                DataType::Array(Box::new(DataType::Int32), 2),
                DataType::Array(Box::new(DataType::Int32), 2),
            ),
        ] {
            assert!(classify_private_cast(&from, &to, CastOptions::NonStrict).is_err());
        }
        Ok(())
    }

    #[test]
    fn test_null_preserving_cast() {
        use DataType::*;

        for (from, to, expected) in [
            (Int64, Float64, true),
            (UInt64, Float64, true),
            (Int64, String, true),
            (Float64, String, true),
            (Boolean, String, true),
            (Float32, Float64, true),
            (Boolean, Boolean, true),
            (Int32, Int64, true),
            (Int64, Int32, false),
            (Float64, Int64, false),
            (String, Int64, false),
        ] {
            assert_eq!(
                is_null_preserving_cast(&from, &to),
                expected,
                "{from:?} -> {to:?}"
            );
        }
    }
}
