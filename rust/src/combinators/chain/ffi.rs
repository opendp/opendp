use opendp_derive::bootstrap;

use crate::combinators::ffi::{AnyOdometerAnswer, AnyOdometerQuery};
use crate::core::{FfiResult, Function, Odometer};

use crate::error::Fallible;
use crate::ffi::any::{
    AnyFunction, AnyMeasurement, AnyObject, AnyOdometer, AnyQueryable, AnyTransformation, Downcast,
};
use crate::interactive::{Answer, Query, Queryable};

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
    arguments(odometer1(rust_type = b"null"), transformation0(rust_type = b"null")),
    dependencies("$get_dependencies(odometer1)", "$get_dependencies(transformation0)")
)]
/// Construct the functional composition (`odometer1` ○ `transformation0`).
/// Returns a Measurement that when invoked, computes `odometer1(transformation0(x))`.
///
/// # Arguments
/// * `odometer1` - outer odometer
/// * `transformation0` - inner transformation
fn make_chain_ot(
    odometer1: &AnyOdometer,
    transformation0: &AnyTransformation,
) -> Fallible<AnyOdometer> {
    let odometer1_function = odometer1.function.clone();
    let odometer1 = Odometer {
        input_domain: odometer1.input_domain.clone(),
        function: Function::new_fallible(move |arg: &AnyObject| {
            let mut inner_qbl = odometer1_function.eval(arg)?.downcast::<AnyQueryable>()?;
            Queryable::new(move |_qbl, query: Query<AnyOdometerQuery>| {
                Ok(match query {
                    Query::External(external) => Answer::External(
                        inner_qbl
                            .eval(&AnyObject::new(external.clone()))?
                            .downcast::<AnyOdometerAnswer>()?,
                    ),
                    Query::Internal(internal) => {
                        let Answer::Internal(answer) = inner_qbl.eval_query(Query::Internal(internal))?
                        else { return fallible!(FailedCast, "return type is not d_out") };

                        Answer::Internal(answer)
                    }
                })
            })
        }),
        input_metric: odometer1.input_metric.clone(),
        output_measure: odometer1.output_measure.clone(),
    };
    Ok(super::make_chain_ot(&odometer1, transformation0)?
        .into_any_queryable()
        .into_any_out())
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_chain_ot(
    odometer1: *const AnyOdometer,
    transformation0: *const AnyTransformation,
) -> FfiResult<*mut AnyOdometer> {
    let transformation0 = try_as_ref!(transformation0);
    let odometer1 = try_as_ref!(odometer1);
    make_chain_ot(odometer1, transformation0).into()
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

#[bootstrap(
    features("contrib"),
    arguments(function1(rust_type = b"null"), odometer0(rust_type = b"null")),
    dependencies("$get_dependencies(function1)", "$get_dependencies(odometer0)")
)]
/// Construct the functional composition (`function1` ○ `odometer0`).
/// Returns an Odometer that when invoked, computes `function1(odometer0(x))`.
///
/// # Arguments
/// * `function1` - outer function
/// * `odometer1` - inner odometer
fn make_chain_po(function1: &AnyFunction, odometer0: &AnyOdometer) -> Fallible<AnyOdometer> {
    let odometer0_function = odometer0.function.clone();
    let odometer0 = Odometer {
        input_domain: odometer0.input_domain.clone(),
        function: Function::new_fallible(move |arg: &AnyObject| {
            let mut inner_qbl = odometer0_function.eval(arg)?.downcast::<AnyQueryable>()?;
            Queryable::new(move |_qbl, query: Query<AnyOdometerQuery>| {
                Ok(match query {
                    Query::External(external) => Answer::External(
                        inner_qbl
                            .eval(&AnyObject::new(external.clone()))?
                            .downcast::<AnyOdometerAnswer>()?,
                    ),
                    Query::Internal(internal) => {
                        let Answer::Internal(answer) = inner_qbl.eval_query(Query::Internal(internal))?
                        else { return fallible!(FailedCast, "return type is not d_out") };

                        Answer::Internal(answer)
                    }
                })
            })
        }),
        input_metric: odometer0.input_metric.clone(),
        output_measure: odometer0.output_measure.clone(),
    };
    Ok(super::make_chain_po(&function1, &odometer0)?
        .into_any_queryable()
        .into_any_out())
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_chain_po(
    function1: *const AnyFunction,
    odometer0: *const AnyOdometer,
) -> FfiResult<*mut AnyOdometer> {
    let odometer0 = try_as_ref!(odometer0);
    let function1 = try_as_ref!(function1);
    make_chain_po(function1, odometer0).into()
}

#[cfg(test)]
mod tests {
    use crate::combinators::tests::{make_test_measurement, make_test_transformation};
    use crate::core::{self, Function};
    use crate::error::Fallible;
    use crate::ffi::any::{
        AnyObject, Downcast, IntoAnyFunctionExt, IntoAnyMeasurementExt, IntoAnyTransformationExt,
    };
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
