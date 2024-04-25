use crate::error::Fallible;

use super::*;

#[test]
fn test_exponential() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = LInfDistance::default();
    let de = make_report_noisy_max_gumbel(input_domain, input_metric, 1., Optimize::Max)?;
    let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
    println!("{:?}", release);

    Ok(())
}
