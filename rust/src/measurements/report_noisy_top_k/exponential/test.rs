use crate::error::Fallible;

use super::*;

#[test]
fn test_permute_and_flip_max() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = LInfDistance::default();
    let de = make_permute_and_flip(input_domain, input_metric, 1, 1., false)?;
    let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
    println!("{:?}", release);
    Ok(())
}

#[test]
fn test_permute_and_flip_min() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = LInfDistance::default();
    let de = make_permute_and_flip(input_domain, input_metric, 1, 1., true)?;
    let release = de.invoke(&vec![1., 2., 3., 2., 0.])?;
    println!("{:?}", release);
    Ok(())
}
