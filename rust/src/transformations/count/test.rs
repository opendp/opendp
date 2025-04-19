use crate::metrics::L2Distance;
use crate::transformations::count::make_count_by_categories;

use super::*;

#[test]
fn test_make_count() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = SymmetricDistance::default();
    let transformation = make_count::<_, i32>(input_domain, input_metric)?;
    let arg = vec![1, 2, 3, 4, 5];
    let ret = transformation.invoke(&arg)?;
    let expected = 5;
    assert_eq!(ret, expected);
    Ok(())
}

#[test]
fn test_make_count_distinct() -> Fallible<()> {
    let transformation = make_count_distinct::<_, i32>(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
    )?;
    let arg = vec![1, 1, 3, 4, 4];
    let ret = transformation.invoke(&arg)?;
    let expected = 3;
    assert_eq!(ret, expected);
    Ok(())
}

#[test]
fn test_make_count_by_categories() {
    let transformation = make_count_by_categories::<L2Distance<f64>, i64, i8>(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
        vec![2, 1, 3],
        true,
    )
    .unwrap_test();
    let arg = vec![1, 2, 3, 4, 5, 1, 1, 1, 2];
    let ret = transformation.invoke(&arg).unwrap_test();
    let expected = vec![2, 4, 1, 2];
    assert_eq!(ret, expected);

    assert!(!transformation.check(&5, &4.999).unwrap_test());
    assert!(transformation.check(&5, &5.0).unwrap_test());
}

#[test]
fn test_make_count_by() -> Fallible<()> {
    let arg = vec![
        true, true, true, false, true, false, false, false, true, true,
    ];
    let transformation = make_count_by::<L2Distance<f64>, bool, i8>(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
    )?;
    let ret = transformation.invoke(&arg)?;
    let mut expected = HashMap::new();
    expected.insert(true, 6);
    expected.insert(false, 4);
    assert_eq!(ret, expected);
    assert!(transformation.check(&6, &6.)?);
    Ok(())
}
