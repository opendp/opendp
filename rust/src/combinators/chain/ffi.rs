use opendp_derive::bootstrap;

use crate::core::FfiResult;

use crate::error::Fallible;
use crate::ffi::any::{AnyFunction, AnyMeasurement, AnyTransformation};

#[bootstrap(
    features("contrib"),
    arguments(
        measurement1(rust_type = b"null"),
        transformation0(rust_type = b"null")
    ),
    dependencies(
        "$get_dependencies(measurement1)",
        "$get_dependencies(transformation0)"
    )
)]
/// Construct the functional composition (`measurement1` ○ `transformation0`).
/// Returns a Measurement that when invoked, computes `measurement1(transformation0(x))`.
///
/// # Arguments
/// * `measurement1` - outer mechanism
/// * `transformation0` - inner transformation
fn make_chain_mt(
    measurement1: &AnyMeasurement,
    transformation0: &AnyTransformation,
) -> Fallible<AnyMeasurement> {
    super::make_chain_mt(measurement1, transformation0)
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_chain_mt(
    measurement1: *const AnyMeasurement,
    transformation0: *const AnyTransformation,
) -> FfiResult<*mut AnyMeasurement> {
    let transformation0 = try_as_ref!(transformation0);
    let measurement1 = try_as_ref!(measurement1);
    make_chain_mt(measurement1, transformation0).into()
}

#[bootstrap(
    features("contrib"),
    arguments(
        transformation1(rust_type = b"null"),
        transformation0(rust_type = b"null")
    ),
    dependencies(
        "$get_dependencies(transformation1)",
        "$get_dependencies(transformation0)"
    )
)]
/// Construct the functional composition (`transformation1` ○ `transformation0`).
/// Returns a Transformation that when invoked, computes `transformation1(transformation0(x))`.
///
/// # Arguments
/// * `transformation1` - outer transformation
/// * `transformation0` - inner transformation
fn make_chain_tt(
    transformation1: &AnyTransformation,
    transformation0: &AnyTransformation,
) -> Fallible<AnyTransformation> {
    super::make_chain_tt(transformation1, transformation0)
}
#[no_mangle]
pub extern "C" fn opendp_combinators__make_chain_tt(
    transformation1: *const AnyTransformation,
    transformation0: *const AnyTransformation,
) -> FfiResult<*mut AnyTransformation> {
    let transformation0 = try_as_ref!(transformation0);
    let transformation1 = try_as_ref!(transformation1);
    make_chain_tt(transformation1, transformation0).into()
}

#[bootstrap(
    features("contrib"),
    arguments(postprocess1(rust_type = b"null"), measurement0(rust_type = b"null")),
    dependencies("$get_dependencies(postprocess1)", "$get_dependencies(measurement0)")
)]
/// Construct the functional composition (`postprocess1` ○ `measurement0`).
/// Returns a Measurement that when invoked, computes `postprocess1(measurement0(x))`.
/// Used to represent non-interactive postprocessing.
///
/// # Arguments
/// * `postprocess1` - outer postprocessor
/// * `measurement0` - inner measurement/mechanism
fn make_chain_pm(
    postprocess1: &AnyFunction,
    measurement0: &AnyMeasurement,
) -> Fallible<AnyMeasurement> {
    super::make_chain_pm(postprocess1, measurement0)
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_chain_pm(
    postprocess1: *const AnyFunction,
    measurement0: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    let postprocess1 = try_as_ref!(postprocess1);
    let measurement0 = try_as_ref!(measurement0);
    make_chain_pm(postprocess1, measurement0).into()
}

#[cfg(test)]
mod tests {
    use crate::combinators::test::{make_test_measurement, make_test_transformation};
    use crate::core::{self, Function};
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;

    use super::*;

    #[test]
    fn test_make_chain_mt_ffi() -> Fallible<()> {
        let transformation0 = util::into_raw(make_test_transformation::<i32>()?.into_any());
        let measurement1 = util::into_raw(make_test_measurement::<i32>()?.into_any());
        let chain = Result::from(opendp_combinators__make_chain_mt(
            measurement1,
            transformation0,
        ))?;
        let arg = AnyObject::new_raw(vec![999]);
        let res = core::opendp_core__measurement_invoke(&chain, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);

        let d_in = AnyObject::new_raw(999u32);
        let d_out = core::opendp_core__measurement_map(&chain, d_in);
        let d_out: f64 = Fallible::from(d_out)?.downcast()?;
        assert_eq!(d_out, 1000.);
        Ok(())
    }

    #[test]
    fn test_make_chain_tt() -> Fallible<()> {
        let transformation0 = util::into_raw(make_test_transformation::<i32>()?.into_any());
        let transformation1 = util::into_raw(make_test_transformation::<i32>()?.into_any());
        let chain = Result::from(opendp_combinators__make_chain_tt(
            transformation1,
            transformation0,
        ))?;
        let arg = AnyObject::new_raw(vec![999]);
        let res = core::opendp_core__transformation_invoke(&chain, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![999]);

        let d_in = AnyObject::new_raw(999u32);
        let d_out = core::opendp_core__transformation_map(&chain, d_in);
        let d_out: u32 = Fallible::from(d_out)?.downcast()?;
        assert_eq!(d_out, 999);
        Ok(())
    }

    #[test]
    fn test_make_chain_pm_ffi() -> Fallible<()> {
        let measurement0 = util::into_raw(make_test_measurement::<i32>()?.into_any());
        let postprocess1 = util::into_raw(Function::new(|arg: &i32| arg.clone()).into_any());
        let chain = Result::from(opendp_combinators__make_chain_pm(
            postprocess1,
            measurement0,
        ))?;
        let arg = AnyObject::new_raw(vec![999]);
        let res = core::opendp_core__measurement_invoke(&chain, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);

        let d_in = AnyObject::new_raw(999u32);
        let d_out = core::opendp_core__measurement_map(&chain, d_in);
        let d_out: f64 = Fallible::from(d_out)?.downcast()?;
        assert_eq!(d_out, 1000.);
        Ok(())
    }
}
