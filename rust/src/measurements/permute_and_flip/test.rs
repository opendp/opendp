use crate::error::Fallible;

use super::*;

#[test]
fn test_shuffle() -> Fallible<()> {
    let mut out: Vec<usize> = (0..10).collect();
    exact_fisher_yates(out.as_mut_slice())?;
    assert_eq!(out.len(), 10);
    // check that out contains all elements from 0 to 9
    for i in 0..10 {
        assert!(out.contains(&i));
    }

    Ok(())
}

#[test]
fn test_permute_and_flip_max() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = LInfDistance::default();
    let de = make_permute_and_flip(input_domain, input_metric, 1., Optimize::Max)?;
    let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
    println!("{:?}", release);
    Ok(())
}

#[test]
fn test_permute_and_flip_min() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = LInfDistance::default();
    let de = make_permute_and_flip(input_domain, input_metric, 1., Optimize::Min)?;
    let release = de.invoke(&vec![1., 2., 3., 2., 0.])?;
    println!("{:?}", release);
    Ok(())
}
