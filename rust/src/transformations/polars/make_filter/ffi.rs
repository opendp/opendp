use polars::prelude::*;

use crate::{
    core::{FfiResult, Function, Transformation},
    domains::{ExprDomain, LazyFrameDomain},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_filter(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    transformation: *const AnyTransformation,
) -> FfiResult<*mut AnyTransformation> {
    // dereference all the pointers
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let transformation = try_as_ref!(transformation);

    // re-pack the inner measurement to work with concrete types (ExprDomain instead of AnyDomain)
    let function = transformation.function.clone();
    let transformation = try_!(Transformation::new(
        try_!(transformation
            .input_domain
            .downcast_ref::<ExprDomain<LazyFrameDomain>>())
        .clone(),
        try_!(transformation
            .output_domain
            .downcast_ref::<ExprDomain<LazyFrameDomain>>())
        .clone(),
        Function::new_fallible(
            move |v: &(Arc<LazyFrame>, Expr)| -> Fallible<(Arc<LazyFrame>, Expr)> {
                let expr_obj = function.eval(&AnyObject::new(v.clone()))?;

                expr_obj.downcast_ref::<(Arc<LazyFrame>, Expr)>().cloned()
            },
        ),
        transformation.input_metric.clone(),
        transformation.output_metric.clone(),
        transformation.stability_map.clone()
    ));

    // call the original function
    let transformation = try_!(super::make_filter::<Transformation<_, _, _, _>>(
        input_domain,
        input_metric,
        transformation
    ));

    // re-pack the resulting measurement to have erased types/be an AnyMeasurement
    let function = transformation.function.clone();
    FfiResult::from(AnyTransformation::new(
        AnyDomain::new(transformation.input_domain.clone()),
        AnyDomain::new(transformation.output_domain.clone()),
        Function::new_fallible(move |v: &AnyObject| -> Fallible<AnyObject> {
            Ok(AnyObject::new(
                function.eval(v.downcast_ref::<LazyFrame>()?)?,
            ))
        }),
        transformation.input_metric.clone(),
        transformation.output_metric.clone(),
        transformation.stability_map.clone(),
    ))
}
