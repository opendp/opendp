use super::*;
use crate::metrics::SymmetricDistance;

#[test]
fn test_find() -> Fallible<()> {
    let find = make_find(
        VectorDomain::default(),
        SymmetricDistance::default(),
        vec!["1", "3", "4"],
    )?;
    assert_eq!(
        find.invoke(&vec!["1", "2", "3", "4", "5"])?,
        vec![Some(0), None, Some(1), Some(2), None]
    );
    Ok(())
}

#[test]
fn test_bin() -> Fallible<()> {
    let bin = make_find_bin(
        Default::default(),
        SymmetricDistance::default(),
        vec![2, 3, 5],
    )?;
    assert_eq!(
        bin.invoke(&(1..10).collect())?,
        vec![0, 1, 2, 2, 3, 3, 3, 3, 3]
    );
    Ok(())
}

#[test]
fn test_index() -> Fallible<()> {
    let index = make_index(
        VectorDomain::default(),
        SymmetricDistance::default(),
        vec!["A", "B", "C"],
        "NA",
    )?;
    assert_eq!(
        index.invoke(&vec![0, 1, 3, 1, 5])?,
        vec!["A", "B", "NA", "B", "NA"]
    );
    Ok(())
}
