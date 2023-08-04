use polars::prelude::*;

use crate::{
    core::{FfiResult, Function, Metric, Transformation},
    domains::{ExprDomain, LazyFrameDomain},
    error::Fallible,
    ffi::{util::{self, AnyTransformationPtr}, any::{AnyDomain, AnyMetric, AnyObject, Downcast, AnyTransformation}},
    metrics::{InsertDeleteDistance, SymmetricDistance, L1}, 
};

use super::IsDatasetMetric;


#[no_mangle]
pub extern "C" fn opendp_transformations__make_with_columns(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    transformations: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    // dereference all the pointers
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let transformations = try_!(try_as_ref!(transformations).downcast_ref::<Vec<AnyTransformationPtr>>());

    // re-pack the inner measurement to work with concrete types (ExprDomain instead of AnyDomain)
    let transformations = try_!(transformations.iter().map(|t| -> Fallible<Transformation<_, _, _, _>> {
        let t = util::as_ref(t.0.clone()).ok_or_else(|| err!(FFI, "transformation is null"))?;
        let function = t.function.clone();
        Transformation::new(
            t.input_domain
                .downcast_ref::<ExprDomain<LazyFrameDomain>>()?
                .clone(),
            t.output_domain
                .downcast_ref::<ExprDomain<LazyFrameDomain>>()?
                .clone(),
            Function::new_fallible(move |v: &(Arc<LazyFrame>, Expr)| -> Fallible<(Arc<LazyFrame>, Expr)> {
                let expr_obj = function.eval(&AnyObject::new(v.clone()))?;
                expr_obj.downcast_ref::<(Arc<LazyFrame>, Expr)>().cloned()
            }),
            t.input_metric.clone(),
            t.output_metric.clone(),
            t.stability_map.clone()
        )
    }).collect::<Fallible<Vec<_>>>());
    
    // call the original function
    let transformation = try_!(super::make_with_columns::<Transformation<_, _, _, _>>(
        input_domain,
        input_metric,
        transformations
    ));

    // re-pack the resulting measurement to have erased types/be an AnyMeasurement
    let function = transformation.function.clone();
    FfiResult::from(Transformation::new(
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


impl IsDatasetMetric for AnyMetric {
    fn is_dataset_metric(&self) -> Fallible<()> {
        fn monomorphize<M: Metric>() -> Fallible<()> {
            Ok(())
        }

        dispatch!(monomorphize, [
            (self.type_, [SymmetricDistance, InsertDeleteDistance, L1<SymmetricDistance>, L1<InsertDeleteDistance>])
        ], ())
    }
}
