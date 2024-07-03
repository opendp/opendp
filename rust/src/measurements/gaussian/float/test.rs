use crate::measures::ZeroConcentratedDivergence;

use super::*;

#[test]
fn test_make_scalar_float_gaussian() -> Fallible<()> {
    let measurement = make_scalar_float_gaussian::<ZeroConcentratedDivergence, _>(
        AtomDomain::default(),
        AbsoluteDistance::default(),
        1.0f64,
        None,
    )?;
    let arg = 0.0;
    let _ret = measurement.invoke(&arg)?;

    assert!(measurement.check(&0.1, &0.0050000001)?);
    Ok(())
}

#[test]
fn test_make_vector_float_gaussian() -> Fallible<()> {
    let measurement = make_vector_float_gaussian::<ZeroConcentratedDivergence, _>(
        VectorDomain::new(AtomDomain::default()),
        L2Distance::default(),
        1.0f64,
        None,
    )?;
    let arg = vec![0.0, 1.0];
    let _ret = measurement.invoke(&arg)?;

    assert!(measurement.map(&0.1)? <= 0.0050000001);
    Ok(())
}
