use crate::core::Domain;
use crate::domains::{
    ArrayDomain, AtomDomain, CategoricalDomain, DatetimeDomain, LazyFrameDomain, OptionDomain,
    SeriesDomain,
};
use crate::metrics::{FrameDistance, SymmetricDistance};
use crate::transformations::make_stable_lazyframe;

use super::*;

#[test]
fn test_make_expr_cast_string() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "String",
        AtomDomain::<String>::default(),
    )])?;
    let lf = df!(
        "String" => &["A".to_string(), "3.14".to_string(), "1e300".to_string(), "NaN".to_string()],
    )?
    .lazy();

    let casted = lf.clone().with_columns([
        col("String").cast(DataType::Float64).alias("String_f64"),
        col("String").cast(DataType::Int64).alias("String_i64"),
        // Polars doesn't support casting a string to a boolean
        // col("String").cast(DataType::Boolean).alias("String_bool"),
    ]);
    let t_casted = make_stable_lazyframe(lf_domain, FrameDistance(SymmetricDistance), casted)?;

    let actual = t_casted.invoke(&lf)?.collect()?;
    let expected = df!(
        // repeat inputs
        "String" => &["A".to_string(), "3.14".to_string(), "1e300".to_string(), "NaN".to_string()],

        // string -> int casts can go to null and nan
        "String_f64" => &[None, Some(3.14), Some(1e300), Some(f64::NAN)],
        // No value is exactly integer, so all are None
        "String_i64" => &[None::<i64>, None, None, None],
        // non-strict casting still fails, but fails in a data-independent way
        // "String_bool" => &[],
    )?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn test_make_expr_cast_f64() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("f64", AtomDomain::<f64>::default())])?;
    let lf = df!(
        "f64" => &[f64::NAN, 1., 1.0e300, -1.],
    )?
    .lazy();

    let casted = lf.clone().with_columns([
        col("f64").cast(DataType::String).alias("f64_String"),
        col("f64").cast(DataType::Int64).alias("f64_i64"),
        col("f64").cast(DataType::Boolean).alias("f64_bool"),
    ]);
    let t_casted = make_stable_lazyframe(lf_domain, FrameDistance(SymmetricDistance), casted)?;

    let actual = t_casted.invoke(&lf)?.collect()?;
    let expected = df!(
        // repeat inputs
        "f64" => &[f64::NAN, 1., 1e300, -1.],
        "f64_String" => &["NaN".to_string(), "1.0".to_string(), "1e300".to_string(), "-1.0".to_string()],
        "f64_i64" => &[None, Some(1), None, Some(-1)],
        // NaN is considered truth-y
        "f64_bool" => &[true, true, true, true],
    )?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn test_make_expr_cast_int() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("i64", AtomDomain::<i64>::default())])?;
    let lf = df!(
        "i64" => &[i64::MAX, i64::MIN, 0, 1],
    )?
    .lazy();

    let casted = lf.clone().with_columns([
        col("i64").cast(DataType::String).alias("i64_String"),
        col("i64").cast(DataType::Float64).alias("i64_f64"),
        col("i64").cast(DataType::Boolean).alias("i64_bool"),
    ]);
    let t_casted = make_stable_lazyframe(lf_domain, FrameDistance(SymmetricDistance), casted)?;

    let actual = t_casted.invoke(&lf)?.collect()?;
    let expected = df!(
        // repeat inputs
        "i64" => &[i64::MAX, i64::MIN, 0, 1],
        "i64_String" => &["9223372036854775807".to_string(), "-9223372036854775808".to_string(), "0".to_string(), "1".to_string()],
        "i64_f64" => &[i64::MAX as f64, i64::MIN as f64, 0.0, 1.0],
        "i64_bool" => &[true, true, false, true],
    )?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn test_make_expr_cast_bool() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "bool",
        AtomDomain::<bool>::default(),
    )])?;
    let lf = df!(
        "bool" => &[true, true, false, false]
    )?
    .lazy();

    let casted = lf.clone().with_columns([
        col("bool").cast(DataType::String).alias("bool_String"),
        col("bool").cast(DataType::Float64).alias("bool_f64"),
        col("bool").cast(DataType::Int64).alias("bool_i64"),
    ]);
    let t_casted = make_stable_lazyframe(lf_domain, FrameDistance(SymmetricDistance), casted)?;

    let actual = t_casted.invoke(&lf)?.collect()?;
    let expected = df!(
        // repeat inputs
        "bool" => &[true, true, false, false],
        "bool_String" => &["true".to_string(), "true".to_string(), "false".to_string(), "false".to_string()],
        "bool_f64" => &[1.0f64, 1.0, 0.0, 0.0],
        "bool_i64" => &[1i64, 1, 0, 0],
    )?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn test_make_expr_cast_output_dtype() -> Fallible<()> {
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "value",
        AtomDomain::<f64>::default(),
    )])?;

    let enum_categories = FrozenCategories::new(["a", "b"].into_iter()).unwrap();
    for dtype in [
        DataType::Boolean,
        DataType::UInt32,
        DataType::UInt64,
        DataType::Int8,
        DataType::Int16,
        DataType::Int32,
        DataType::Int64,
        DataType::Float32,
        DataType::Float64,
        DataType::String,
        DataType::Datetime(TimeUnit::Microseconds, None),
        DataType::Time,
        DataType::Enum(enum_categories.clone(), enum_categories.mapping().clone()),
        DataType::Array(Box::new(DataType::Float32), 2),
    ] {
        let cast = make_expr_cast(
            input_domain.clone().row_by_row(),
            FrameDistance(SymmetricDistance),
            col("value").cast(dtype.clone()),
        )?;

        assert_eq!(cast.output_domain.column.dtype(), dtype);
    }
    Ok(())
}

#[test]
fn test_make_expr_cast_categorical_to_integer_rejected() -> Fallible<()> {
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "category",
        CategoricalDomain::default(),
    )])?;

    for dtype in [
        DataType::UInt8,
        DataType::UInt16,
        DataType::UInt32,
        DataType::UInt64,
        DataType::Int8,
        DataType::Int16,
        DataType::Int32,
        DataType::Int64,
    ] {
        let result = make_expr_cast(
            input_domain.clone().row_by_row(),
            FrameDistance(SymmetricDistance),
            col("category").cast(dtype),
        );
        assert!(result.is_err());
    }

    // Logical string values and fixed Enum codes remain safe.
    let categories = FrozenCategories::new(["A", "B"])?;
    for dtype in [
        DataType::String,
        DataType::Enum(categories.clone(), categories.mapping().clone()),
    ] {
        make_expr_cast(
            input_domain.clone().row_by_row(),
            FrameDistance(SymmetricDistance),
            col("category").cast(dtype),
        )?;
    }

    Ok(())
}

#[test]
fn test_make_expr_cast_rejects_mutable_categorical_destinations() -> Fallible<()> {
    let input_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("name", AtomDomain::<String>::default()),
        SeriesDomain::new("code", AtomDomain::<u32>::default()),
    ])?;
    let categories = Categories::global();
    let dtype = DataType::Categorical(categories.clone(), categories.mapping());

    // Inserting strings into a mapping can raise a data-dependent error once
    // its finite capacity is reached. Casting integers decodes mutable codes.
    for column in ["name", "code"] {
        let result = make_expr_cast(
            input_domain.clone().row_by_row(),
            FrameDistance(SymmetricDistance),
            col(column).cast(dtype.clone()),
        );
        assert!(result.is_err(), "{column} -> Categorical must be rejected");
    }

    // A fixed Enum has public codes and cannot grow at execution time.
    let enum_categories = FrozenCategories::new(["A", "B"])?;
    let enum_dtype = DataType::Enum(
        enum_categories.clone(),
        enum_categories.mapping().clone(),
    );
    let enum_cast = make_expr_cast(
        input_domain.clone().row_by_row(),
        FrameDistance(SymmetricDistance),
        col("name").cast(enum_dtype.clone()),
    )?;
    assert_eq!(enum_cast.output_domain.column.dtype(), enum_dtype);
    assert!(enum_cast.output_domain.column.nullable);

    // A categorical no-op is safe, as are casts to fixed Enum and String.
    let cat_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "category",
        CategoricalDomain::default(),
    )])?;
    make_expr_cast(
        cat_domain.row_by_row(),
        FrameDistance(SymmetricDistance),
        col("category").cast(dtype),
    )?;

    Ok(())
}

#[test]
fn test_make_expr_cast_rejects_categorical_remapping() -> Fallible<()> {
    let cat_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "category",
        CategoricalDomain::default(),
    )])?;
    let categories = Categories::random("".into(), CategoricalPhysical::U32);
    let dtype = DataType::Categorical(categories.clone(), categories.mapping());
    assert!(
        make_expr_cast(
            cat_domain.row_by_row(),
            FrameDistance(SymmetricDistance),
            col("category").cast(dtype),
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn test_make_expr_cast_rejects_nested_categorical_casts() -> Fallible<()> {
    let categories = Categories::global();
    let categorical = DataType::Categorical(categories.clone(), categories.mapping());

    let cat_array = LazyFrameDomain::new(vec![SeriesDomain::new(
        "values",
        ArrayDomain::new(CategoricalDomain::default(), 2),
    )])?;
    let string_array = LazyFrameDomain::new(vec![SeriesDomain::new(
        "values",
        ArrayDomain::new(AtomDomain::<String>::default(), 2),
    )])?;

    // Polars recursively casts Array elements, so a top-level categorical
    // guard alone does not protect the physical codes of nested categoricals.
    assert!(
        make_expr_cast(
            cat_array.clone().row_by_row(),
            FrameDistance(SymmetricDistance),
            col("values").cast(DataType::Array(Box::new(DataType::UInt32), 2)),
        )
        .is_err()
    );
    assert!(
        make_expr_cast(
            string_array.row_by_row(),
            FrameDistance(SymmetricDistance),
            col("values").cast(DataType::Array(Box::new(categorical), 2)),
        )
        .is_err()
    );

    // Reading logical strings from a categorical Array is still safe.
    make_expr_cast(
        cat_array.row_by_row(),
        FrameDistance(SymmetricDistance),
        col("values").cast(DataType::Array(Box::new(DataType::String), 2)),
    )?;
    Ok(())
}

#[test]
fn test_make_expr_cast_rejects_non_strict_categorical_capacity_errors() -> Fallible<()> {
    let categories = Categories::random("".into(), CategoricalPhysical::U8);
    let dtype = DataType::Categorical(categories.clone(), categories.mapping());
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "value",
        AtomDomain::<String>::default(),
    )])?;

    // Non-strict casting does not prevent a failure when a mutable mapping
    // runs out of category IDs. The constructor must reject this cast.
    let values = (0..256).map(|i| i.to_string()).collect::<Vec<_>>();
    let series = Series::new("value".into(), values);
    assert!(series.cast_with_options(&dtype, CastOptions::NonStrict).is_err());
    assert!(
        make_expr_cast(
            input_domain.row_by_row(),
            FrameDistance(SymmetricDistance),
            col("value").cast(dtype),
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn test_make_expr_cast_rejects_nanosecond_timestamp_parsing() -> Fallible<()> {
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "value",
        AtomDomain::<String>::default(),
    )])?;

    // Polars 1.36.1 calls timestamp_nanos_opt().unwrap() when parsing an
    // otherwise valid date outside the nanosecond timestamp range.
    for timezone in [None, Some("UTC".into())] {
        let dtype = DataType::Datetime(TimeUnit::Nanoseconds, timezone);
        assert!(
            make_expr_cast(
                input_domain.clone().row_by_row(),
                FrameDistance(SymmetricDistance),
                col("value").cast(dtype),
            )
            .is_err()
        );
    }

    // Millisecond and microsecond timestamp parsing does not use that panic path.
    for unit in [TimeUnit::Milliseconds, TimeUnit::Microseconds] {
        make_expr_cast(
            input_domain.clone().row_by_row(),
            FrameDistance(SymmetricDistance),
            col("value").cast(DataType::Datetime(unit, None)),
        )?;
    }

    Ok(())
}

#[test]
fn test_make_expr_cast_rejects_numeric_to_date() -> Fallible<()> {
    let input_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("days", AtomDomain::<i32>::default()),
        SeriesDomain::new("other", AtomDomain::<i64>::default()),
        SeriesDomain::new("string", AtomDomain::<String>::default()),
    ])?;

    // Int32 -> Date is a physical reinterpretation with no Chrono range
    // check. Extreme i32 values produce non-null but invalid dates.
    for column in ["days", "other"] {
        assert!(
            make_expr_cast(
                input_domain.clone().row_by_row(),
                FrameDistance(SymmetricDistance),
                col(column).cast(DataType::Date),
            )
            .is_err()
        );
    }

    // Millisecond/microsecond Datetime domains admit times well outside the
    // calendar range Chrono can format, even when the resulting day count
    // fits into i32.
    for unit in [TimeUnit::Milliseconds, TimeUnit::Microseconds] {
        let datetime_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
            "timestamp",
            DatetimeDomain {
                time_unit: unit,
                time_zone: None,
            },
        )])?;
        assert!(
            make_expr_cast(
                datetime_domain.row_by_row(),
                FrameDistance(SymmetricDistance),
                col("timestamp").cast(DataType::Date),
            )
            .is_err()
        );
    }

    // Every i64 nanosecond timestamp lies within Chrono's calendar range.
    let ns_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "timestamp",
        DatetimeDomain {
            time_unit: TimeUnit::Nanoseconds,
            time_zone: None,
        },
    )])?;
    make_expr_cast(
        ns_domain.row_by_row(),
        FrameDistance(SymmetricDistance),
        col("timestamp").cast(DataType::Date),
    )?;

    // A parsed date is checked by Polars, with invalid strings becoming null.
    make_expr_cast(
        input_domain.row_by_row(),
        FrameDistance(SymmetricDistance),
        col("string").cast(DataType::Date),
    )?;
    Ok(())
}

#[test]
fn test_make_expr_cast_array_element_nullability() -> Fallible<()> {
    let string_array = LazyFrameDomain::new(vec![SeriesDomain::new(
        "values",
        ArrayDomain::new(AtomDomain::<String>::default(), 2),
    )])?;

    // "invalid" becomes a null *inside* the Array. Marking the containing
    // Series nullable does not cover this possible result.
    assert!(
        make_expr_cast(
            string_array.row_by_row(),
            FrameDistance(SymmetricDistance),
            col("values").cast(DataType::Array(Box::new(DataType::Int64), 2)),
        )
        .is_err()
    );

    let int_array = LazyFrameDomain::new(vec![SeriesDomain::new(
        "values",
        ArrayDomain::new(AtomDomain::<i32>::default(), 2),
    )])?;
    let widen = make_expr_cast(
        int_array.row_by_row(),
        FrameDistance(SymmetricDistance),
        col("values").cast(DataType::Array(Box::new(DataType::Int64), 2)),
    )?;
    assert!(!widen.output_domain.column.nullable);

    // Identity casts preserve domain constraints, including inner nullability.
    let nullable_inner = LazyFrameDomain::new(vec![SeriesDomain::new(
        "values",
        ArrayDomain::new(OptionDomain::new(AtomDomain::<i32>::default()), 2),
    )])?;
    let identity = make_expr_cast(
        nullable_inner.clone().row_by_row(),
        FrameDistance(SymmetricDistance),
        col("values").cast(DataType::Array(Box::new(DataType::Int32), 2)),
    )?;
    assert!(
        identity
            .output_domain
            .column
            .element_domain::<ArrayDomain>()?
            .has_nullable_elements()
    );

    // A non-identity cast would erase this property when resetting the dtype.
    assert!(
        make_expr_cast(
            nullable_inner.row_by_row(),
            FrameDistance(SymmetricDistance),
            col("values").cast(DataType::Array(Box::new(DataType::Int64), 2)),
        )
        .is_err()
    );

    Ok(())
}

#[test]
fn test_make_expr_cast_nullability() -> Fallible<()> {
    // Non-strict casts can introduce nulls into otherwise non-nullable columns.
    let lf = df!("x" => &["1", "not-an-integer"])?.lazy();
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "x",
        AtomDomain::<String>::default(),
    )])?;
    assert!(!input_domain.series_domains[0].nullable);

    // The constructor forces the default strict cast to be non-strict.
    let t_cast = make_expr_cast(
        input_domain.row_by_row(),
        FrameDistance(SymmetricDistance),
        col("x").cast(DataType::Int64),
    )?;
    assert!(t_cast.output_domain.column.nullable);

    let expr = t_cast.invoke(&lf.logical_plan)?.expr;
    let observed = lf.select([expr]).collect()?;
    assert_eq!(observed, df!("x" => &[Some(1_i64), None])?);
    assert!(
        t_cast
            .output_domain
            .column
            .member(observed.column("x")?.as_materialized_series())?
    );

    Ok(())
}

#[test]
fn test_make_expr_cast_widening_preserves_non_null() -> Fallible<()> {
    let lf = df!("x" => &[1i32, 2, 3])?.lazy();
    let input_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("x", AtomDomain::<i32>::default())])?;

    let t_cast = make_expr_cast(
        input_domain.row_by_row(),
        FrameDistance(SymmetricDistance),
        col("x").cast(DataType::Int64),
    )?;
    assert!(!t_cast.output_domain.column.nullable);

    let expr = t_cast.invoke(&lf.logical_plan)?.expr;
    let observed = lf.select([expr]).collect()?;
    assert_eq!(observed, df!("x" => &[1i64, 2, 3])?);
    assert!(
        t_cast
            .output_domain
            .column
            .member(observed.column("x")?.as_materialized_series())?
    );

    Ok(())
}

#[test]
fn test_make_expr_cast_other_null_preserving() -> Fallible<()> {
    let input_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("integer", AtomDomain::<i64>::default()),
        SeriesDomain::new("float", AtomDomain::<f32>::default()),
        SeriesDomain::new("bool", AtomDomain::<bool>::default()),
    ])?;

    for expr in [
        col("integer").cast(DataType::Float64),
        col("integer").cast(DataType::String),
        col("float").cast(DataType::Float64),
        col("bool").cast(DataType::String),
    ] {
        let t_cast = make_expr_cast(
            input_domain.clone().row_by_row(),
            FrameDistance(SymmetricDistance),
            expr,
        )?;
        assert!(!t_cast.output_domain.column.nullable);
    }

    Ok(())
}
