use crate::domains::OptionDomain;

use super::*;

#[test]
fn test_series_new() -> Fallible<()> {
    let series_domain = SeriesDomain::new("A", AtomDomain::<bool>::default());
    assert!(series_domain == series_domain);

    let series = Series::new("A".into(), vec![true; 50]);
    assert!(series_domain.member(&series)?);
    Ok(())
}

#[test]
fn test_series_bounded() -> Fallible<()> {
    let series_domain = SeriesDomain::new("A", AtomDomain::new_closed((1, 3))?);

    let inside_bounds = Series::new("A".into(), vec![1; 50]);
    assert!(series_domain.member(&inside_bounds)?);

    let outside_bounds = Series::new("A".into(), vec![4; 50]);
    assert!(!series_domain.member(&outside_bounds)?);

    Ok(())
}

#[test]
fn test_series_non_nan() -> Fallible<()> {
    // option domain with non-nan type
    let series_domain = SeriesDomain::new("A", OptionDomain::new(AtomDomain::<bool>::default()));

    let series = Series::new("A".into(), vec![Some(true), Some(false), None]);
    assert!(series_domain.member(&series)?);

    Ok(())
}

#[test]
fn test_series_nan_without_option() -> Fallible<()> {
    // nan type without options
    let series_domain = SeriesDomain::new("A", AtomDomain::<f64>::default());

    let series_with_none = Series::new("A".into(), vec![Some(1.), Some(f64::NAN), None]);
    assert!(!series_domain.member(&series_with_none)?);

    // series made with Option::Some are ok
    let series_with_some = Series::new("A".into(), vec![Some(1.), Some(f64::NAN)]);
    assert!(series_domain.member(&series_with_some)?);

    // series made without options are ok
    let series_wo_some = Series::new("A".into(), vec![1., f64::NAN]);
    assert!(series_domain.member(&series_wo_some)?);

    Ok(())
}

#[test]
fn test_series_nan_with_option() -> Fallible<()> {
    // permit both kinds of nullity
    let series_domain = SeriesDomain::new("A", OptionDomain::new(AtomDomain::<f64>::default()));

    // None and NaN are both ok
    let series_with_none = Series::new("A".into(), vec![Some(1.), Some(f64::NAN), None]);
    assert!(series_domain.member(&series_with_none)?);

    // doesn't have to have NaN
    let series_wo_none = Series::new("A".into(), vec![1., 2.]);
    assert!(series_domain.member(&series_wo_none)?);

    Ok(())
}

#[test]
fn test_series_small_unsigned() -> Fallible<()> {
    for dtype in [DataType::UInt8, DataType::UInt16] {
        let mut domain = SeriesDomain::new_from_field(Field::new("A".into(), dtype.clone()))?;
        assert_eq!(domain.dtype(), dtype);
        assert!(domain.nullable);
        domain.drop_bounds()?;

        let series = Series::new("A".into(), [1u32, 2, 3]).cast(&dtype)?;
        assert!(domain.member(&series)?);
    }
    Ok(())
}

#[test]
fn test_series_nullable_ignores_null_payloads() -> Fallible<()> {
    let bounded = AtomDomain::<i64>::new_closed((10, 20))?;
    let nullable = SeriesDomain::new("A", OptionDomain::new(bounded.clone()));
    let values = Series::new("A".into(), [None::<i64>, Some(15)]);
    // Null-slot physical payloads must not be checked against the bounds.
    assert!(nullable.member(&values)?);
    assert!(!nullable.member(&Series::new("A".into(), [None, Some(9)]))?);
    assert!(!SeriesDomain::new("A", bounded).member(&values)?);
    Ok(())
}

#[test]
fn test_series_temporal_membership() -> Fallible<()> {
    let date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let time = NaiveTime::from_hms_opt(0, 0, 1).unwrap();
    let series = [
        Series::new("A".into(), [date]),
        Series::new("A".into(), [time]),
        Series::new("A".into(), [date.and_time(time)]),
    ];
    for value in &series {
        let domain = SeriesDomain::new_from_field(Field::new("A".into(), value.dtype().clone()))?;
        assert!(domain.member(value)?);
    }
    let date_domain = SeriesDomain::new_from_field(Field::new("A".into(), DataType::Date))?;
    assert!(!date_domain.member(&series[2])?);
    Ok(())
}

#[test]
fn test_temporal_conversion_boundaries() {
    assert_eq!(date_from_days(0), NaiveDate::from_ymd_opt(1970, 1, 1));
    for days in [i32::MIN, i32::MAX] {
        assert!(date_from_days(days).is_none());
    }
    assert_eq!(time_from_nanos(0), NaiveTime::from_hms_opt(0, 0, 0));
    assert_eq!(
        time_from_nanos(86_400_000_000_000 - 1),
        NaiveTime::from_hms_nano_opt(23, 59, 59, 999_999_999)
    );
    // Invalid physical times must not wrap into valid times.
    for nanos in [-1, 86_400_000_000_000, 4_294_967_296_000_000_000, i64::MAX] {
        assert!(time_from_nanos(nanos).is_none());
    }
}

#[test]
fn test_series_categorical_membership() -> Fallible<()> {
    let domain = SeriesDomain::new("A", CategoricalDomain);
    let value = Series::new("A".into(), ["a", "b"]).cast(&domain.dtype())?;
    assert!(domain.member(&value)?);
    Ok(())
}

#[test]
fn test_series_array_membership_checks_bounds_and_width() -> Fallible<()> {
    let array_domain =
        ArrayDomain::new(AtomDomain::<i32>::new_closed((1, 3))?, 2).with_nullable(false);
    assert!(!array_domain.member(&Series::new("".into(), [1i32]))?);

    let dtype = DataType::Array(Box::new(DataType::Int32), 2);
    let make = |row| -> Fallible<Series> { Ok(Series::new("A".into(), [row]).cast(&dtype)?) };
    let domain = SeriesDomain::new("A", array_domain);
    assert!(domain.member(&make(Series::new("".into(), [1i32, 2]))?)?);
    assert!(!domain.member(&make(Series::new("".into(), [1i32, 4]))?)?);
    Ok(())
}

#[test]
fn test_series_array_inner_outer_nullability() -> Fallible<()> {
    let dtype = DataType::Array(Box::new(DataType::Int32), 2);
    let array = ArrayDomain::new(AtomDomain::<i32>::new_closed((1, 3))?, 2);
    let make = |values: Vec<Option<Series>>| -> Fallible<Series> {
        Ok(Series::new("A".into(), values).cast(&dtype)?)
    };
    let row = Series::new("".into(), [1i32, 2]);
    let inner_null = make(vec![Some(Series::new("".into(), [Some(1i32), None]))])?;
    let outer_null = make(vec![Some(row), None])?;
    let both_null = make(vec![Some(Series::new("".into(), [Some(1i32), None])), None])?;

    let neither = SeriesDomain::new("A", array.clone().with_nullable(false));
    let inner_only = SeriesDomain::new("A", array.clone());
    let outer_only = SeriesDomain::new("A", OptionDomain::new(array.clone().with_nullable(false)));
    let both = SeriesDomain::new("A", OptionDomain::new(array));

    assert!(!neither.member(&inner_null)?);
    assert!(!neither.member(&outer_null)?);
    assert!(inner_only.member(&inner_null)?);
    assert!(!inner_only.member(&outer_null)?);
    assert!(outer_only.member(&outer_null)?);
    assert!(!outer_only.member(&inner_null)?);
    assert!(both.member(&both_null)?);

    // Schemas do not encode nullability at either level.
    let inferred = SeriesDomain::new_from_field(Field::new("A".into(), dtype))?;
    assert!(inferred.nullable);
    assert!(inferred.element_domain::<ArrayDomain>()?.nullable);
    assert!(inferred.member(&both_null)?);
    Ok(())
}

#[test]
fn test_nested_array_member_bounds() -> Fallible<()> {
    let inner = ArrayDomain::new(AtomDomain::<i32>::new_closed((1, 3))?, 2);
    let domain = ArrayDomain::new(inner, 2);
    let dtype = DataType::Array(Box::new(DataType::Int32), 2);
    let make = |last| -> Fallible<Series> {
        Ok(Series::new(
            "".into(),
            [Series::new("".into(), [1i32, 2]), Series::new("".into(), [2i32, last])],
        )
        .cast(&dtype)?)
    };
    assert!(domain.member(&make(3)?)?);
    assert!(!domain.member(&make(4)?)?);
    Ok(())
}

#[test]
fn test_series_nested_enum_membership() -> Fallible<()> {
    let enum_domain = EnumDomain::new(Series::new("".into(), ["a", "b"]))?;
    let domain = SeriesDomain::new("A", ArrayDomain::new(enum_domain, 2));
    let make = |categories: [&str; 2]| -> Fallible<Series> {
        // Independent mappings: category labels and their order must agree.
        let categories = EnumDomain::new(Series::new("".into(), categories))?;
        let dtype = SeriesElementDomain::dtype(&categories);
        let row = Series::new("".into(), ["a", "b"]).cast(&dtype)?;
        Ok(Series::new("A".into(), [row]).cast(&DataType::Array(Box::new(dtype), 2))?)
    };
    let matching = make(["a", "b"])?;
    let reordered = make(["b", "a"])?;
    assert!(domain.member(&matching)?);
    assert!(!domain.member(&reordered)?);

    // The dtype check must also recurse through multiple Array levels.
    let nested = ArrayDomain::new(domain.element_domain::<ArrayDomain>()?.clone(), 3);
    let wrap = |dtype| DataType::Array(Box::new(dtype), 3);
    assert!(SeriesElementDomain::matches_dtype(&nested, &wrap(matching.dtype().clone())));
    assert!(!SeriesElementDomain::matches_dtype(&nested, &wrap(reordered.dtype().clone())));
    Ok(())
}
