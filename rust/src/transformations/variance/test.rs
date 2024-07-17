use crate::transformations::Pairwise;

use super::*;

#[test]
fn test_make_variance() -> Fallible<()> {
    let arg = vec![1., 2., 3., 4., 5.];

    let input_domain = VectorDomain::new(AtomDomain::new_closed((0., 10.))?).with_size(5);
    let input_metric = SymmetricDistance::default();
    let transformation_sample =
        make_variance::<Pairwise<_>>(input_domain.clone(), input_metric.clone(), 1)?;
    let ret = transformation_sample.invoke(&arg)?;
    let expected = 2.5;
    assert_eq!(ret, expected);
    assert!(transformation_sample.check(&1, &(100. / 5.))?);

    let transformation_pop = make_variance::<Pairwise<_>>(input_domain, input_metric, 0)?;
    let ret = transformation_pop.invoke(&arg)?;
    let expected = 2.0;
    assert_eq!(ret, expected);
    assert!(transformation_pop.check(&1, &(100. * 4. / 25.))?);

    Ok(())
}
