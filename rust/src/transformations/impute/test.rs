use crate::metrics::SymmetricDistance;

use super::*;

#[test]
fn test_impute_uniform() -> Fallible<()> {
    let imputer = make_impute_uniform_float(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance,
        (2.0, 3.0),
    )?;
    assert!(!imputer.output_domain.element_domain.nan());

    let result = imputer.invoke(&vec![1.0, f64::NAN])?;

    assert_eq!(result[0], 1.);
    assert!((2.0..3.0).contains(&result[1]));
    assert!(imputer.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_impute_uniform_infinite_width() {
    fn assert_rejected<TA: Float + SampleUniform>(bounds: (TA, TA)) {
        let message = make_impute_uniform_float(
            VectorDomain::new(AtomDomain::default()),
            SymmetricDistance,
            bounds,
        )
        .unwrap_err()
        .message
        .unwrap_or_default();
        assert!(message.contains("must be finite"), "{bounds:?}: {message}");
    }

    assert_rejected((0.0, f64::INFINITY));
    assert_rejected((f64::NEG_INFINITY, 0.0));
    assert_rejected((f64::NEG_INFINITY, f64::INFINITY));
    // finite bounds, but the width overflows
    assert_rejected((f64::MIN, f64::MAX));
    assert_rejected((f32::MIN, f32::MAX));
}

#[test]
fn test_impute_constant_option() -> Fallible<()> {
    let imputer = make_impute_constant(
        VectorDomain::new(OptionDomain::new(AtomDomain::default())),
        SymmetricDistance,
        "IMPUTED".to_string(),
    )?;
    assert!(!imputer.output_domain.element_domain.nan());

    let result = imputer.invoke(&vec![Some("A".to_string()), None])?;

    assert_eq!(result, vec!["A".to_string(), "IMPUTED".to_string()]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_impute_constant_inherent() -> Fallible<()> {
    let imputer = make_impute_constant(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance,
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
        SymmetricDistance,
    )?;

    assert!(!imputer.output_domain.element_domain.nan());

    let result = imputer.invoke(&vec![Some(f64::NAN), Some(23.), None])?;

    assert_eq!(result, vec![23.]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}
#[test]
fn test_impute_drop_inherent() -> Fallible<()> {
    let imputer = make_drop_null(VectorDomain::new(AtomDomain::default()), SymmetricDistance)?;
    assert!(!imputer.output_domain.element_domain.nan());

    let result = imputer.invoke(&vec![f64::NAN, 23.])?;

    assert_eq!(result, vec![23.]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}
