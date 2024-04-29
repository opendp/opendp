use super::*;
use crate::domains::AtomDomain;

#[test]
fn test() -> Fallible<()> {
    let (input_domain, input_metric) = (
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
    );
    let trans =
        make_resize::<_, SymmetricDistance, SymmetricDistance>(input_domain, input_metric, 3, "x")?;
    assert_eq!(trans.invoke(&vec!["A"; 2])?, vec!["A", "A", "x"]);
    assert_eq!(trans.invoke(&vec!["A"; 3])?, vec!["A"; 3]);
    assert_eq!(trans.invoke(&vec!["A"; 4])?, vec!["A", "A", "A"]);

    assert!(trans.check(&1, &2)?);
    assert!(!trans.check(&1, &1)?);
    Ok(())
}
