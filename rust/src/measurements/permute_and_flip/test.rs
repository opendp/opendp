use crate::error::Fallible;

use super::*;

#[test]
fn test_shuffle() -> Fallible<()> {
    let out: Vec<usize> = exact_fisher_yates(10, None)?;
    assert_eq!(out.len(), 10);
    // check that out contains all elements from 0 to 9
    for i in 0..10 {
        assert!(out.contains(&i));
    }

    // check that the elements are shuffled
    let mut sorted = out.clone();
    sorted.sort();
    assert_ne!(out, sorted);
    
    Ok(())
}

#[test]
fn test_permute_and_flip_max() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = LInfDistance::default();
    let de = make_report_noisy_max_permute_and_flip(input_domain, input_metric, 1., Optimize::Max)?;
    let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
    println!("{:?}", release);
    Ok(())
}


#[test]
fn test_permute_and_flip_min() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = LInfDistance::default();
    let de = make_report_noisy_max_permute_and_flip(input_domain, input_metric, 1., Optimize::Min)?;
    let release = de.invoke(&vec![1., 2., 3., 2., 0.])?;
    println!("{:?}", release);
    
    Ok(())
}
