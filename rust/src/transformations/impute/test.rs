use crate::metrics::SymmetricDistance;

use super::*;

#[test]
fn test_impute_uniform() -> Fallible<()> {
    let imputer = make_impute_uniform_float(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
        (2.0, 3.0),
    )?;

    let result = imputer.invoke(&vec![1.0, f64::NAN])?;

    assert_eq!(result[0], 1.);
    assert!((2.0..3.0).contains(&result[1]));
    assert!(imputer.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_impute_constant_option() -> Fallible<()> {
    let imputer = make_impute_constant(
        VectorDomain::new(OptionDomain::new(AtomDomain::default())),
        SymmetricDistance::default(),
        "IMPUTED".to_string(),
    )?;

    let result = imputer.invoke(&vec![Some("A".to_string()), None])?;

    assert_eq!(result, vec!["A".to_string(), "IMPUTED".to_string()]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_impute_constant_inherent() -> Fallible<()> {
    let imputer = make_impute_constant(
        VectorDomain::new(AtomDomain::new_nullable()),
        SymmetricDistance::default(),
        12.,
    )?;

    let result = imputer.invoke(&vec![f64::NAN, 23.])?;

    assert_eq!(result, vec![12., 23.]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_impute_drop_option() -> Fallible<()> {
    let imputer = make_drop_null(
        VectorDomain::new(OptionDomain::default()),
        SymmetricDistance::default(),
    )?;

    let result = imputer.invoke(&vec![Some(f64::NAN), Some(23.), None])?;

    assert_eq!(result, vec![23.]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}
#[test]
fn test_impute_drop_inherent() -> Fallible<()> {
    let imputer = make_drop_null(
        VectorDomain::new(AtomDomain::new_nullable()),
        SymmetricDistance::default(),
    )?;

    let result = imputer.invoke(&vec![f64::NAN, 23.])?;

    assert_eq!(result, vec![23.]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}
