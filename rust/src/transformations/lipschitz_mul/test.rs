use super::*;

#[test]
fn test_lipschitz_mul() -> Fallible<()> {
    let extension = make_lipschitz_float_mul(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        2.,
        (0., 10.),
    )?;
    assert_eq!(extension.invoke(&1.3)?, 2.6);
    println!("{:?}", extension.invoke(&1.3));
    Ok(())
}
