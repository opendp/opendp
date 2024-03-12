use polars::prelude::*;

use crate::{
    core::{FfiResult, Function, Measurement, Metric, MetricSpace},
    domains::{ExprDomain, LazyGroupByDomain},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, AnyObject, Downcast},
    metrics::{InsertDeleteDistance, SymmetricDistance, L1},
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_private_agg(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // dereference all the pointers
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyGroupByDomain>()).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let measurement = try_as_ref!(measurement);

    // re-pack the inner measurement to work with concrete types (ExprDomain instead of AnyDomain)
    let function = measurement.function.clone();
    let measurement = try_!(Measurement::new(
        try_!(measurement
            .input_domain
            .downcast_ref::<ExprDomain<LazyGroupByDomain>>())
        .clone(),
        Function::new_fallible(move |v: &(Arc<LazyGroupBy>, Expr)| -> Fallible<Vec<Expr>> {
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
    let measurement = try_!(super::make_private_agg::<Measurement<_, _, _, _>>(
        input_domain,
        input_metric,
        measurement
    ));

    // re-pack the resulting measurement to have erased types/be an AnyMeasurement
    let function = measurement.function.clone();
    FfiResult::from(Measurement::new(
        AnyDomain::new(measurement.input_domain.clone()),
        Function::new_fallible(move |v: &AnyObject| -> Fallible<AnyObject> {
            Ok(AnyObject::new(
                function.eval(v.downcast_ref::<LazyGroupBy>()?)?,
            ))
        }),
        measurement.input_metric.clone(),
        measurement.output_measure.clone(),
        measurement.privacy_map.clone(),
    ))
}

// how to do MetricSpace check for re-packed input measurement
impl MetricSpace for (ExprDomain<LazyGroupByDomain>, AnyMetric) {
    fn check_space(&self) -> Fallible<()> {
        let (domain, metric) = self.clone();

        fn monomorphize<M: 'static + Metric>(
            domain: ExprDomain<LazyGroupByDomain>,
            metric: AnyMetric,
        ) -> Fallible<()>
        where
            (ExprDomain<LazyGroupByDomain>, M): MetricSpace,
        {
            let input_metric = metric.downcast_ref::<M>()?;
            (domain.clone(), input_metric.clone()).check_space()
        }

        dispatch!(monomorphize, [
            (metric.type_, [L1<SymmetricDistance>, L1<InsertDeleteDistance>])
        ], (domain, metric))
    }
}

// how to do MetricSpace check for re-packed output measurement
impl MetricSpace for (LazyGroupByDomain, AnyMetric) {
    fn check_space(&self) -> Fallible<()> {
        let (domain, metric) = self.clone();

        fn monomorphize<M: 'static + Metric>(
            domain: LazyGroupByDomain,
            metric: AnyMetric,
        ) -> Fallible<()>
        where
            (LazyGroupByDomain, M): MetricSpace,
        {
            let input_metric = metric.downcast_ref::<M>()?;
            (domain.clone(), input_metric.clone()).check_space()
        }

        dispatch!(monomorphize, [
            (metric.type_, [L1<SymmetricDistance>, L1<InsertDeleteDistance>])
        ], (domain, metric))
    }
}
