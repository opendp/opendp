use crate::domains::AtomDomain;
use crate::metrics::AbsoluteDistance;

use super::*;

#[test]
#[cfg(feature = "ffi")]
fn test_threading() -> Fallible<()> {
    use crate::{measurements::make_randomized_response_bool, transformations::make_split_lines};

    fn is_send_sync<T: Send + Sync>(_arg: &T) {}

    let meas = make_randomized_response_bool(0.75, false)?;
    is_send_sync(&meas);
    is_send_sync(&meas.into_any());

    let trans = make_split_lines()?;
    is_send_sync(&trans);
    is_send_sync(&trans.into_any());

    Ok(())
}

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
        output_domain,
        function,
        input_metric,
        output_metric,
        stability_map,
    )?;
    let arg = 99;
    let ret = identity.invoke(&arg)?;
    assert_eq!(ret, 99);
    Ok(())
}
