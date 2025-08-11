use crate::domains::AtomDomain;
use crate::metrics::AbsoluteDistance;

use super::*;

#[test]
fn test_identity() -> Fallible<()> {
    let input_domain = AtomDomain::<i32>::default();
    let output_domain = AtomDomain::<i32>::default();
    let function = Function::new(|arg: &i32| arg.clone());
    let input_metric = AbsoluteDistance::<i32>::default();
    let output_metric = AbsoluteDistance::<i32>::default();
    let stability_map = StabilityMap::new_from_constant(1);
    let identity = Transformation::new(
        input_domain,
        input_metric,
        output_domain,
        output_metric,
        function,
        stability_map,
    )?;
    let arg = 99;
    let ret = identity.invoke(&arg)?;
    assert_eq!(ret, 99);
    Ok(())
}
