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
