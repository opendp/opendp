use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measurement},
    ffi::{
        any::{AnyMeasurement, AnyObject, Downcast},
        util::AnyMeasurementPtr,
    },
};

use super::make_composition;

#[bootstrap(
    name = "make_composition",
    features("contrib"),
    arguments(measurements(rust_type = "Vec<AnyMeasurementPtr>"))
)]
/// Construct the DP composition \[`measurement0`, `measurement1`, ...\].
/// Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`
///
/// All metrics and domains must be equivalent.
///
/// **Composition Properties**
///
/// * sequential: all measurements are applied to the same dataset
/// * basic: the composition is the linear sum of the privacy usage of each query
/// * noninteractive: all mechanisms specified up-front (but each can be interactive)
/// * compositor: all privacy parameters specified up-front (via the map)
///
/// # Arguments
/// * `measurements` - A vector of Measurements to compose.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_composition(
    measurements: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let meas_ptrs = try_!(try_as_ref!(measurements).downcast_ref::<Vec<AnyMeasurementPtr>>());

    let measurements: Vec<AnyMeasurement> = try_!(
        meas_ptrs
            .iter()
            .map(|ptr| Ok(try_as_ref!(*ptr).clone()))
            .collect()
    );
    make_composition(measurements)
        .map(Measurement::into_any_out)
        .into()
}

#[bootstrap(
    name = "make_basic_composition",
    features("contrib"),
    arguments(measurements(rust_type = "Vec<AnyMeasurementPtr>"))
)]
/// Construct the DP composition \[`measurement0`, `measurement1`, ...\].
/// Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`
///
/// All metrics and domains must be equivalent.
///
/// **Composition Properties**
///
/// * sequential: all measurements are applied to the same dataset
/// * basic: the composition is the linear sum of the privacy usage of each query
/// * noninteractive: all mechanisms specified up-front (but each can be interactive)
/// * compositor: all privacy parameters specified up-front (via the map)
///
/// # Arguments
/// * `measurements` - A vector of Measurements to compose.
#[deprecated(
    since = "0.14.0",
    note = "This function has been renamed, use `make_composition` instead."
)]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_basic_composition(
    measurements: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    opendp_combinators__make_composition(measurements)
}

#[cfg(test)]
mod tests {
    use crate::combinators::test::make_test_measurement;
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;

    use super::*;

    #[test]
    fn test_make_composition_ffi() -> Fallible<()> {
        let measurement0 =
            util::into_raw(make_test_measurement::<i32>()?.into_any()) as AnyMeasurementPtr;
        let measurement1 =
            util::into_raw(make_test_measurement::<i32>()?.into_any()) as AnyMeasurementPtr;
        let measurements = vec![measurement0, measurement1];
        let basic_composition = Result::from(opendp_combinators__make_composition(
            AnyObject::new_raw(measurements),
        ))?;
        let arg = AnyObject::new_raw(vec![999]);
        let res = core::opendp_core__measurement_invoke(&basic_composition, arg);
        let res: Vec<AnyObject> = Fallible::from(res)?.downcast()?;
        let res = (
            *res[0].downcast_ref::<i32>()?,
            *res[1].downcast_ref::<i32>()?,
        );
        println!("{:?}", res);
        assert_eq!(res, (999, 999));
        Ok(())
    }
}
