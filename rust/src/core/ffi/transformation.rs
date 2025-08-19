use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{Domain, Fallible, Metric, MetricSpace, Transformation},
    ffi::{
        any::{AnyDomain, AnyFunction, AnyMetric, AnyObject, AnyTransformation},
        util::{self, c_bool, into_c_char_p},
    },
};

use super::FfiResult;

/// Trait to convert Result<Transformation> into FfiResult<*mut AnyTransformation>. We can't do this with From
/// because there's a blanket implementation of From for FfiResult. We can't do this with a method on Result
/// because it comes from another crate. So we need a separate trait.
pub trait IntoAnyTransformationFfiResultExt {
    fn into_any(self) -> Fallible<AnyTransformation>;
}

impl<DI: 'static + Domain, DO: 'static + Domain, MI: 'static + Metric, MO: 'static + Metric>
    IntoAnyTransformationFfiResultExt for Fallible<Transformation<DI, MI, DO, MO>>
where
    (DI, MI): MetricSpace,
    (DO, MO): MetricSpace,
{
    fn into_any(self) -> Fallible<AnyTransformation> {
        self.map(Transformation::into_any)
    }
}

#[bootstrap(
    name = "transformation_input_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Get the input domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__transformation_input_domain(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_domain.clone()))
}

#[bootstrap(
    name = "transformation_output_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Get the output domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__transformation_output_domain(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_domain.clone()))
}

#[bootstrap(
    name = "transformation_input_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Get the input domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__transformation_input_metric(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_metric.clone()))
}

#[bootstrap(
    name = "transformation_output_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Get the output domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__transformation_output_metric(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_metric.clone()))
}

#[bootstrap(
    name = "transformation_map",
    arguments(
        transformation(rust_type = b"null"),
        distance_in(rust_type = "$transformation_input_distance_type(transformation)")
    )
)]
/// Use the `transformation` to map a given `d_in` to `d_out`.
///
/// # Arguments
/// * `transformation` - Transformation to check the map distances with.
/// * `distance_in` - Distance in terms of the input metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__transformation_map(
    transformation: *const AnyTransformation,
    distance_in: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let transformation = try_as_ref!(transformation);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = transformation.map(distance_in);
    distance_out.into()
}

#[bootstrap(
    name = "transformation_check",
    arguments(
        transformation(rust_type = b"null"),
        distance_in(rust_type = "$transformation_input_distance_type(transformation)"),
        distance_out(rust_type = "$transformation_output_distance_type(transformation)"),
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
pub extern "C" fn opendp_core__transformation_check(
    transformation: *const AnyTransformation,
    distance_in: *const AnyObject,
    distance_out: *const AnyObject,
) -> FfiResult<*mut c_bool> {
    let transformation = try_as_ref!(transformation);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = try_as_ref!(distance_out);
    let status = try_!(transformation.check(distance_in, distance_out));
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

#[bootstrap(
    name = "transformation_invoke",
    arguments(
        this(rust_type = b"null"),
        arg(rust_type = "$transformation_input_carrier_type(this)")
    )
)]
/// Invoke the `transformation` with `arg`. Returns a differentially private release.
///
/// # Arguments
/// * `this` - Transformation to invoke.
/// * `arg` - Input data to supply to the transformation. A member of the transformation's input domain.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__transformation_invoke(
    this: *const AnyTransformation,
    arg: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    this.invoke(arg).into()
}

#[bootstrap(
    name = "transformation_function",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyFunction *>", do_not_convert = true)
)]
/// Get the function from a transformation.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__transformation_function(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyFunction> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).function.clone()))
}

#[bootstrap(
    name = "_transformation_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core___transformation_free(
    this: *mut AnyTransformation,
) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "transformation_input_carrier_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input (carrier) data type of `this`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__transformation_input_carrier_type(
    this: *mut AnyTransformation,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_domain.carrier_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "transformation_input_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input distance type of `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__transformation_input_distance_type(
    this: *mut AnyTransformation,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_metric.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "transformation_output_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the output distance type of `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__transformation_output_distance_type(
    this: *mut AnyTransformation,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.output_metric.distance_type.descriptor.to_string()
    )))
}

#[cfg(test)]
mod tests {
    use crate::{
        combinators::test::make_test_transformation,
        error::{ErrorVariant, Fallible},
        ffi::any::Downcast,
    };

    use super::*;

    #[test]
    fn test_transformation_invoke() -> Fallible<()> {
        let transformation = util::into_raw(make_test_transformation::<i32>()?.into_any());
        let arg = AnyObject::new_raw(vec![999]);
        let res = opendp_core__transformation_invoke(transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![999]);
        Ok(())
    }

    #[test]
    fn test_transformation_invoke_wrong_type() -> Fallible<()> {
        let transformation = util::into_raw(make_test_transformation::<i32>()?.into_any());
        let arg = AnyObject::new_raw(999.0);
        let res = Fallible::from(opendp_core__transformation_invoke(transformation, arg));
        assert_eq!(res.err().unwrap().variant, ErrorVariant::FailedCast);
        Ok(())
    }
}
