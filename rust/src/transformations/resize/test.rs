use super::*;
use crate::domains::AtomDomain;
use crate::error::ErrorVariant;

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

#[test]
fn test_excessive_size_is_fallible() -> Fallible<()> {
    // Regression for #3006: an excessive target size with a small dataset
    // must return a FailedFunction error rather than panic or abort.
    let trans = make_resize::<_, SymmetricDistance, SymmetricDistance>(
        VectorDomain::new(AtomDomain::<i32>::default()),
        SymmetricDistance,
        usize::MAX,
        0,
    )?;
    let error = trans.invoke(&vec![1, 2]).unwrap_err();
    assert_eq!(error.variant, ErrorVariant::FailedFunction);
    Ok(())
}

#[test]
fn test_resize_truncates_via_reservoir_sampling() -> Fallible<()> {
    let trans = make_resize::<_, SymmetricDistance, SymmetricDistance>(
        VectorDomain::new(AtomDomain::<i32>::default()),
        SymmetricDistance,
        3,
        0,
    )?;
    // Regression: unordered truncation samples `size` distinct elements of the input.
    for _ in 0..16 {
        let out = trans.invoke(&vec![1, 2, 3, 4, 5])?;
        assert_eq!(out.len(), 3);
        assert!(out.iter().all(|v| (1..=5).contains(v)));
        let mut sorted = out;
        sorted.sort_unstable();
        assert!(sorted.windows(2).all(|w| w[0] < w[1]));
    }
    Ok(())
}
