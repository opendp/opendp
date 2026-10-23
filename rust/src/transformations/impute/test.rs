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
    assert!((2.0..=3.0).contains(&result[1]));
    assert!(imputer.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_impute_uniform_narrow_bounds_terminates() -> Fallible<()> {
    // #3015: the exclusive sampler spends excessive time adjusting narrow bounds.
    let cases = [
        (0.0, 1.0),
        (1.7e9, 1.7e9 + 1.0),
        (1.0, 1.0 + 1e-10),
        (1.0, f64::from_bits(1_f64.to_bits() + 1)),
    ];
    for (lower, upper) in cases {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = make_impute_uniform_float(
                VectorDomain::new(AtomDomain::default()),
                SymmetricDistance,
                (lower, upper),
            )
            .and_then(|imputer| imputer.invoke(&vec![f64::NAN; 10_000]));
            let _ = sender.send(result);
        });
        let result = receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap_or_else(|_| panic!("imputation on [{lower}, {upper}] did not finish"))?;
        assert_eq!(result.len(), 10_000);
        assert!(result.iter().all(|value| (lower..=upper).contains(value)));
    }
    Ok(())
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
