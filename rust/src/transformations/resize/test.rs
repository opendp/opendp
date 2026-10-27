use super::*;
use crate::domains::AtomDomain;

#[test]
fn test() -> Fallible<()> {
    let (input_domain, input_metric) =
        (VectorDomain::new(AtomDomain::default()), SymmetricDistance);
    let trans =
        make_resize::<_, SymmetricDistance, SymmetricDistance>(input_domain, input_metric, 3, "x")?;
    assert_eq!(trans.invoke(&vec!["A"; 2])?, vec!["A", "A", "x"]);
    assert_eq!(trans.invoke(&vec!["A"; 3])?, vec!["A"; 3]);
    assert_eq!(trans.invoke(&vec!["A"; 4])?, vec!["A", "A", "A"]);

    assert!(trans.check(&1, &2)?);
    assert!(!trans.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_ordered_resize_preserves_order() -> Fallible<()> {
    let trans = make_resize::<_, InsertDeleteDistance, InsertDeleteDistance>(
        VectorDomain::new(AtomDomain::<i32>::default()),
        InsertDeleteDistance,
        3,
        0,
    )?;

    // Regression: resizing an already-sized ordered vector must not shuffle.
    for _ in 0..16 {
        assert_eq!(trans.invoke(&vec![1, 2])?, vec![1, 2, 0]);
        assert_eq!(trans.invoke(&vec![1, 2, 3])?, vec![1, 2, 3]);
        assert_eq!(trans.invoke(&vec![1, 2, 3, 4])?, vec![1, 2, 3]);
    }
    Ok(())
}
