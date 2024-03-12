use polars::prelude::*;

use crate::{
    core::{FfiResult, Function, Measurement},
    domains::{ExprDomain, LazyFrameDomain},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, AnyObject, Downcast},
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_private_select(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // dereference all the pointers
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let measurement = try_as_ref!(measurement);

    // re-pack the inner measurement to work with concrete types (ExprDomain instead of AnyDomain)
    let function = measurement.function.clone();
    let measurement = try_!(Measurement::new(
        try_!(measurement
            .input_domain
            .downcast_ref::<ExprDomain<LazyFrameDomain>>())
        .clone(),
        Function::new_fallible(move |v: &(Arc<LazyFrame>, Expr)| -> Fallible<Vec<Expr>> {
            let expr_obj = function.eval(&AnyObject::new(v.clone()))?;

            expr_obj
                .downcast_ref::<Vec<AnyObject>>()?
                .into_iter()
                .map(|obj| obj.downcast_ref::<Expr>().cloned())
                .collect::<Fallible<Vec<Expr>>>()
        },),
        measurement.input_metric.clone(),
        measurement.output_measure.clone(),
        measurement.privacy_map.clone()
    ));

    // call the original function
    let measurement = try_!(super::make_private_select::<Measurement<_, _, _, _>>(
        input_domain,
        input_metric,
        measurement
    ));

    // re-pack the resulting measurement to have erased types/be an AnyMeasurement
    let function = measurement.function.clone();
    FfiResult::from(AnyMeasurement::new(
        AnyDomain::new(measurement.input_domain.clone()),
        Function::new_fallible(move |v: &AnyObject| -> Fallible<AnyObject> {
            Ok(AnyObject::new(
                function.eval(v.downcast_ref::<LazyFrame>()?)?,
            ))
        }),
        measurement.input_metric.clone(),
        measurement.output_measure.clone(),
        measurement.privacy_map.clone(),
    ))
}
