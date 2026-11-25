use polars::prelude::DataType;

use crate::error::*;

// Restrict private casts to reviewed dtype pairs. Unsupported casts must be
// rejected rather than relying on Polars' non-strict behavior to be infallible.
// Return whether a non-null input may produce null.
pub(crate) fn check_private_cast(from: &DataType, to: &DataType) -> Fallible<bool> {
    use DataType::*;

    let allowed = match (from, to) {
        // Identity is safe only for dtypes whose representation is supported.
        (left, right) if left == right => matches!(
            left,
            Boolean
                | UInt8
                | UInt16
                | UInt32
                | UInt64
                | Int8
                | Int16
                | Int32
                | Int64
                | Float32
                | Float64
                | String
                | Date
                | Datetime(_, _)
                | Time
                | Categorical(_, _)
                | Enum(_, _)
        ),
        (from, to) if is_numeric_coercion(from, to) => true,
        (from, String) if from.is_primitive_numeric() || matches!(from, Boolean) => true,
        (String, to) if to.is_primitive_numeric() => true,
        (Categorical(_, _), String | Enum(_, _)) | (Enum(_, _), String) | (String, Enum(_, _)) => {
            true
        }
        // In particular, no cross-temporal, physical categorical, or nested
        // conversions have an established data-independent failure guarantee.
        _ => false,
    };

    if !allowed {
        return fallible!(
            MakeTransformation,
            "casting from {} to {} is not supported for private data",
            from,
            to
        );
    }

    Ok(!is_null_preserving_cast(from, to))
}

// The only mixed-dtype implicit promotions currently certified for binary
// expressions and fill_null are between primitive numeric/boolean columns.
pub(crate) fn is_numeric_coercion(from: &DataType, to: &DataType) -> bool {
    let numeric =
        |dtype: &DataType| dtype.is_primitive_numeric() || matches!(dtype, DataType::Boolean);
    numeric(from) && numeric(to)
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
fn is_null_preserving_cast(from: &DataType, to: &DataType) -> bool {
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
    use polars::prelude::{Field, TimeUnit};

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
    fn test_private_cast_safety() -> Fallible<()> {
        use DataType::*;

        for (from, to, may_introduce_null) in [
            (Int64, Float64, false),
            (UInt64, Float64, false),
            (Int64, String, false),
            (Float64, String, false),
            (Boolean, String, false),
            (Float32, Float64, false),
            (Boolean, Boolean, false),
            (Int32, Int64, false),
            (Int64, Int32, true),
            (Float64, Int64, true),
            (String, Int64, true),
            (UInt8, Float64, true),
            (Boolean, Int64, true),
        ] {
            assert_eq!(
                check_private_cast(&from, &to)?,
                may_introduce_null,
                "{from:?} -> {to:?}"
            );
        }

        for (from, to) in [
            (String, Datetime(TimeUnit::Nanoseconds, None)),
            (Datetime(TimeUnit::Microseconds, None), String),
            (Date, Datetime(TimeUnit::Nanoseconds, None)),
            (Datetime(TimeUnit::Nanoseconds, None), Date),
            (Int32, Date),
            (Float64, Time),
            (Float64, Datetime(TimeUnit::Microseconds, None)),
            (Array(Box::new(Int32), 2), Array(Box::new(Int32), 2)),
            (List(Box::new(Int32)), List(Box::new(Int32))),
        ] {
            assert!(
                check_private_cast(&from, &to).is_err(),
                "{from:?} -> {to:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn test_private_cast_allowlist_matrix() -> Fallible<()> {
        use DataType::*;

        let numeric = [
            Boolean, UInt8, UInt16, UInt32, UInt64, Int8, Int16, Int32, Int64, Float32, Float64,
        ];
        for from in &numeric {
            for to in &numeric {
                assert!(is_numeric_coercion(from, to));
                check_private_cast(from, to)?;
            }
            check_private_cast(from, &String)?;
        }
        for to in numeric.iter().filter(|to| to.is_primitive_numeric()) {
            check_private_cast(&String, to)?;
        }
        assert!(!is_numeric_coercion(&String, &Int64));
        assert!(!is_numeric_coercion(&Date, &Int32));

        // Unsupported families must stay rejected even for identity casts.
        for dtype in [
            Null,
            Binary,
            List(Box::new(Int32)),
            Array(Box::new(Int32), 2),
            Struct(vec![Field::new("x".into(), Int32)]),
        ] {
            assert!(check_private_cast(&dtype, &dtype).is_err(), "{dtype:?}");
        }
        Ok(())
    }
}
