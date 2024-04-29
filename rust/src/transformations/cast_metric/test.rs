use crate::domains::{AtomDomain, VectorDomain};
use crate::metrics::{ChangeOneDistance, InsertDeleteDistance, SymmetricDistance};

use super::*;

#[test]
fn test_ordering() -> Fallible<()> {
    let domain = VectorDomain::new(AtomDomain::default());
    let ord_trans = make_ordered_random(domain.clone(), SymmetricDistance::default())?;
    let data = vec![1i32, 2, 3];
    assert_eq!(ord_trans.invoke(&data)?.len(), 3);

    let ident_trans = (ord_trans >> make_unordered(domain, InsertDeleteDistance::default())?)?;
    assert_eq!(ident_trans.invoke(&data)?.len(), 3);
    Ok(())
}

#[test]
fn test_bounded() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default()).with_size(3);
    let bdd_trans = make_metric_bounded(input_domain.clone(), SymmetricDistance::default())?;
    let data = vec![1i32, 2, 3];
    assert_eq!(bdd_trans.invoke(&data)?.len(), 3);

    let ident_trans =
        (bdd_trans >> make_metric_unbounded(input_domain, ChangeOneDistance::default())?)?;
    assert_eq!(ident_trans.invoke(&data)?.len(), 3);
    Ok(())
}
