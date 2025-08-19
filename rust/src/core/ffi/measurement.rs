use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{Domain, Fallible, Measure, Measurement, Metric, MetricSpace},
    ffi::{
        any::{AnyDomain, AnyFunction, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject},
        util::{self, c_bool, into_c_char_p},
    },
};

use super::FfiResult;

/// Trait to convert Result<Measurement> into FfiResult<*mut AnyMeasurement>. We can't do this with From
/// because there's a blanket implementation of From for FfiResult. We can't do this with a method on Result
/// because it comes from another crate. So we need a separate trait.
pub trait IntoAnyMeasurementFfiResultExt {
    fn into_any(self) -> Fallible<AnyMeasurement>;
}

impl<DI: 'static + Domain, TO: 'static, MI: 'static + Metric, MO: 'static + Measure>
    IntoAnyMeasurementFfiResultExt for Fallible<Measurement<DI, MI, MO, TO>>
where
    MO::Distance: 'static,
    (DI, MI): MetricSpace,
{
    fn into_any(self) -> Fallible<AnyMeasurement> {
        self.map(Measurement::into_any)
    }
}

#[bootstrap(
    name = "measurement_input_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Get the input domain from a `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__measurement_input_domain(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_domain.clone()))
}

#[bootstrap(
    name = "measurement_input_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Get the input domain from a `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__measurement_input_metric(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_metric.clone()))
}

#[bootstrap(
    name = "measurement_output_measure",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMeasure *>")
)]
/// Get the output domain from a `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__measurement_output_measure(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyMeasure> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_measure.clone()))
}

#[bootstrap(
    name = "measurement_function",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyFunction *>", do_not_convert = true)
)]
/// Get the function from a measurement.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__measurement_function(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyFunction> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).function.clone()))
}

#[bootstrap(
    name = "measurement_map",
    arguments(
        measurement(rust_type = b"null"),
        distance_in(rust_type = "$measurement_input_distance_type(measurement)"),
        distance_out(rust_type = "$measurement_output_distance_type(measurement)"),
    )
)]
/// Use the `measurement` to map a given `d_in` to `d_out`.
///
/// # Arguments
/// * `measurement` - Measurement to check the map distances with.
/// * `distance_in` - Distance in terms of the input metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__measurement_map(
    measurement: *const AnyMeasurement,
    distance_in: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let measurement = try_as_ref!(measurement);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = measurement.map(distance_in);
    distance_out.into()
}

#[bootstrap(
    name = "measurement_check",
    arguments(
        measurement(rust_type = b"null"),
        distance_in(rust_type = "$measurement_input_distance_type(measurement)"),
        distance_out(rust_type = "$measurement_output_distance_type(measurement)"),
    ),
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
///
/// # Arguments
/// * `measurement` - Measurement to check the privacy relation of.
/// * `d_in` - Distance in terms of the input metric.
/// * `d_out` - Distance in terms of the output metric.
///
/// # Returns
/// True indicates that the relation passed at the given distance.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__measurement_check(
    measurement: *const AnyMeasurement,
    distance_in: *const AnyObject,
    distance_out: *const AnyObject,
) -> FfiResult<*mut c_bool> {
    let measurement = try_as_ref!(measurement);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = try_as_ref!(distance_out);
    let status = try_!(measurement.check(distance_in, distance_out));
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

#[bootstrap(
    name = "measurement_invoke",
    arguments(
        this(rust_type = b"null"),
        arg(rust_type = "$measurement_input_carrier_type(this)")
    )
)]
/// Invoke the `measurement` with `arg`. Returns a differentially private release.
///
/// # Arguments
/// * `this` - Measurement to invoke.
/// * `arg` - Input data to supply to the measurement. A member of the measurement's input domain.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__measurement_invoke(
    this: *const AnyMeasurement,
    arg: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    this.invoke(arg).into()
}

#[bootstrap(
    name = "_measurement_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core___measurement_free(this: *mut AnyMeasurement) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "measurement_input_carrier_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input (carrier) data type of `this`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__measurement_input_carrier_type(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_domain.carrier_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "measurement_input_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input distance type of `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__measurement_input_distance_type(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_metric.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "measurement_output_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the output distance type of `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__measurement_output_distance_type(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.output_measure.distance_type.descriptor.to_string()
    )))
}

#[cfg(test)]
mod tests {
    use crate::{
        combinators::test::make_test_measurement,
        error::{ErrorVariant, Fallible},
        ffi::any::Downcast,
    };

    use super::*;

    #[test]
    fn test_measurement_invoke() -> Fallible<()> {
        let measurement = util::into_raw(make_test_measurement::<i32>()?.into_any());
        let arg = AnyObject::new_raw(vec![999]);
        let res = opendp_core__measurement_invoke(measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);
        Ok(())
    }

    #[test]
    fn test_measurement_invoke_wrong_type() -> Fallible<()> {
        let measurement = util::into_raw(make_test_measurement::<i32>()?.into_any());
        let arg = AnyObject::new_raw(vec![999.0]);
        let res = Fallible::from(opendp_core__measurement_invoke(measurement, arg));
        assert_eq!(res.err().unwrap().variant, ErrorVariant::FailedCast);
        Ok(())
    }
}
