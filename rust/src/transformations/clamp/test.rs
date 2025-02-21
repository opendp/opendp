use crate::{error::Fallible, metrics::SymmetricDistance, transformations::then_clamp};

use super::*;

#[test]
fn test_make_clamp() -> Fallible<()> {
    let input_space = (
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
    );
    let transformation = (input_space >> then_clamp((0, 10)))?;
    let arg = vec![-10, -5, 0, 5, 10, 20];
    let ret = transformation.invoke(&arg)?;
    let expected = vec![0, 0, 0, 5, 10, 10];
    assert_eq!(ret, expected);
    Ok(())
}
