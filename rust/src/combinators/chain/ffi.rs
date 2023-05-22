use opendp_derive::bootstrap;

use crate::core::{
    AnyOdometerAnswer, AnyOdometerQuery, OdometerAnswer, OdometerQuery, StabilityMap,
};
use crate::core::{FfiResult, Function, Odometer};

use crate::error::Fallible;
use crate::ffi::any::{
    AnyFunction, AnyMeasurement, AnyObject, AnyOdometer, AnyTransformation, Downcast,
    QueryOdometerMapType,
};
use crate::ffi::util::{ExtrinsicObject, Type};
use crate::interactive::{Answer, Query, Queryable};
use crate::measures::ffi::TypedMeasure;
use crate::metrics::ffi::TypedMetric;

#[bootstrap(
    features("contrib"),
    arguments(
        measurement1(rust_type = b"null"),
        transformation0(rust_type = b"null")
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

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_chain_mt(
    measurement1: *const AnyMeasurement,
    transformation0: *const AnyTransformation,
) -> FfiResult<*mut AnyMeasurement> {
    let transformation0 = try_as_ref!(transformation0);
    let measurement1 = try_as_ref!(measurement1);
    make_chain_mt(measurement1, transformation0).into()
}

// 1. create fully adaptive composition odometer
//    - external queries are partially typed
//    - internal queries are partially typed
// 2. erase types in ffi
//    - external queries are AnyObject
//    - internal queries are left partially typed
// 3.

#[bootstrap(
    name = "make_chain_ot",
    features("contrib"),
    arguments(odometer1(rust_type = b"null"), transformation0(rust_type = b"null"))
)]
/// Construct the functional composition (`odometer1` ○ `transformation0`).
/// Returns a Measurement that when invoked, computes `odometer1(transformation0(x))`.
///
/// # Arguments
/// * `odometer1` - outer odometer
/// * `transformation0` - inner transformation
#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_chain_ot(
    odometer1: *const AnyOdometer,
    transformation0: *const AnyTransformation,
) -> FfiResult<*mut AnyOdometer> {
    let transformation0 = try_as_ref!(transformation0);
    let odometer1 = try_as_ref!(odometer1);

    fn monomorphize<QI: 'static + Clone, QX: 'static + Clone, QO: 'static>(
        transformation0: &AnyTransformation,
        odometer1: &AnyOdometer,
    ) -> Fallible<AnyOdometer> {
        let stability_map = transformation0.stability_map.clone();
        let transformation0 = try_!(transformation0.with_map(
            try_!(TypedMetric::<QI>::new(transformation0.input_metric.clone())),
            try_!(TypedMetric::<QX>::new(
                transformation0.output_metric.clone()
            )),
            StabilityMap::new_fallible(move |d_in: &QI| {
                stability_map
                    .eval(&AnyObject::new(d_in.clone()))?
                    .downcast::<QX>()
            },)
        ));

        let function = odometer1.function.clone();
        let odometer1 = try_!(Odometer::new(
            odometer1.input_domain.clone(),
            try_!(TypedMetric::<QX>::new(odometer1.input_metric.clone())),
            try_!(TypedMeasure::<QO>::new(odometer1.output_measure.clone())),
            Function::new_fallible(
                move |arg: &AnyObject| -> Fallible<
                    Queryable<OdometerQuery<AnyObject, QX>, OdometerAnswer<AnyObject, QO>>,
                > {
                    let mut inner_qbl = function.eval(arg)?;

                    Ok(Queryable::new_raw(
                        move |_self, query: Query<OdometerQuery<AnyObject, QX>>| match query {
                            Query::External(OdometerQuery::Invoke(arg)) => {
                                let release = inner_qbl.invoke(arg.clone())?;
                                Ok(Answer::External(OdometerAnswer::Invoke(release)))
                            }
                            Query::External(OdometerQuery::PrivacyLoss(d_in)) => {
                                let d_out = inner_qbl.privacy_loss(AnyObject::new(d_in.clone()))?;
                                Ok(Answer::External(OdometerAnswer::PrivacyLoss(
                                    d_out.downcast::<QO>()?,
                                )))
                            }
                            Query::Internal(any) => {
                                let Answer::Internal(answer) =
                                    inner_qbl.eval_query(Query::Internal(any))?
                                else {
                                    return fallible!(FailedFunction, "expected internal answer");
                                };
                                Ok(Answer::Internal(answer))
                            }
                        },
                    ))
                },
            ),
        ));

        let odometer = try_!(super::make_chain_ot(&odometer1, &transformation0));

        let function = odometer.function.clone();
        Odometer::new(
            odometer.input_domain.clone(),
            odometer.input_metric.metric.clone(),
            odometer.output_measure.measure.clone(),
            Function::new_fallible(
                move |arg: &AnyObject| -> Fallible<Queryable<AnyOdometerQuery, AnyOdometerAnswer>> {
                    let mut inner_qbl = function.eval(arg)?;

                    Ok(Queryable::new_raw(
                        move |_self, query: Query<AnyOdometerQuery>| match query {
                            Query::External(OdometerQuery::Invoke(query)) => {
                                let answer = inner_qbl.invoke(query.clone())?;
                                Ok(Answer::External(OdometerAnswer::Invoke(answer)))
                            }
                            Query::External(OdometerQuery::PrivacyLoss(d_in)) => {
                                let answer = AnyObject::new(
                                    inner_qbl.privacy_loss(d_in.downcast_ref::<QI>()?.clone())?,
                                );
                                Ok(Answer::External(OdometerAnswer::PrivacyLoss(answer)))
                            }
                            Query::Internal(query) => {
                                if query.downcast_ref::<QueryOdometerMapType>().is_some() {
                                    return Ok(Answer::internal(Type::of::<QI>()));
                                }

                                let Answer::Internal(answer) =
                                    inner_qbl.eval_query(Query::Internal(query))?
                                else {
                                    return fallible!(
                                        FailedFunction,
                                        "internal query returned external answer"
                                    );
                                };
                                Ok(Answer::Internal(answer))
                            }
                        },
                    ))
                },
            ),
        )
    }

    let QI = transformation0.input_metric.distance_type.clone();
    let QX = transformation0.output_metric.distance_type.clone();
    let QO = odometer1.output_measure.distance_type.clone();
    dispatch!(
        monomorphize,
        [
            (QI, @numbers),
            (QX, @numbers_plus),
            (QO, [f64, (f64, f64)])
        ],
        (transformation0, odometer1)
    )
    .into()
}

#[bootstrap(
    features("contrib"),
    arguments(
        transformation1(rust_type = b"null"),
        transformation0(rust_type = b"null")
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
#[unsafe(no_mangle)]
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
    arguments(postprocess1(rust_type = b"null"), measurement0(rust_type = b"null"))
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

#[unsafe(no_mangle)]
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
