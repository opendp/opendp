use polars::prelude::{DataType, TimeUnit};

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
            (DataType::Categorical(_, _) | DataType::Enum(_, _), DataType::String)
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
    matches!(dtype, DataType::Date | DataType::Datetime(_, _) | DataType::Time)
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
