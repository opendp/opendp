use super::*;
use crate::domains::{AtomDomain, OptionDomain};

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_identity() {
    let identity =
        make_identity(VectorDomain::new(AtomDomain::default()), SymmetricDistance).unwrap_test();
    let arg = vec![99];
    let ret = identity.invoke(&arg).unwrap_test();
    assert_eq!(ret, arg);
}

#[test]
fn test_is_equal() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = SymmetricDistance;
    let is_equal = make_is_equal(input_domain, input_metric, "alpha".to_string())?;
    let arg = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
    let ret = is_equal.invoke(&arg)?;

    assert_eq!(ret, vec![true, false, false]);
    assert!(is_equal.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_is_null_inherent() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::new_nullable());
    let input_metric = SymmetricDistance;
    let is_equal = make_is_null(input_domain, input_metric)?;
    let arg = vec![1., 2., f64::NAN];
    let ret = is_equal.invoke(&arg)?;

    assert_eq!(ret, vec![false, false, true]);
    assert!(is_equal.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_is_null_option() -> Fallible<()> {
    let input_domain = VectorDomain::new(OptionDomain::new(AtomDomain::new_nullable()));
    let input_metric = SymmetricDistance;
    let is_equal = make_is_null(input_domain, input_metric)?;
    let arg = vec![Some(1.), None, Some(f64::NAN)];
    let ret = is_equal.invoke(&arg)?;

    assert_eq!(ret, vec![false, true, true]);
    assert!(is_equal.check(&1, &1)?);
    Ok(())
}
