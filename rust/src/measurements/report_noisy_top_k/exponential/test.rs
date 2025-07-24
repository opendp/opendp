use crate::error::Fallible;

use super::*;

#[test]
fn test_permute_and_flip_max() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::new_non_nan());
    let input_metric = LInfDistance::default();
    let m_pf = make_report_noisy_top_k_exponential(input_domain, input_metric, 1, 1., false)?;
    let release = m_pf.invoke(&vec![1., 2., 3., 2., 1.])?;

    println!("{:?}", release);
    Ok(())
}

#[test]
fn test_permute_and_flip_min() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::new_non_nan());
    let input_metric = LInfDistance::default();
    let de = make_report_noisy_top_k_exponential(input_domain, input_metric, 1, 1., true)?;
    let release = de.invoke(&vec![1., 2., 3., 2., 0.])?;
    println!("{:?}", release);
    Ok(())
}
