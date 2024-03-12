use std::{ffi::c_char, os::raw::c_void};

use crate::core::IntoAnyMeasurementFfiResultExt;
use crate::{
    core::{Domain, FfiResult, MetricSpace},
    domains::ExprDomain,
    ffi::{
        any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast},
        util::Type,
    },
    measurements::LaplaceOuterMetric,
    metrics::{InsertDeleteDistance, SymmetricDistance, L1},
    traits::{Float, InfCast},
    transformations::{traits::UnboundedMetric, SumOuterMetric, Summand},
    error::Fallible,
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_private_mean_expr(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<MI, TA, QO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
    ) -> Fallible<AnyMeasurement>
    where
        MI: SumOuterMetric<TA>,
        MI::InnerMetric: UnboundedMetric,
        TA: Summand,
        QO: InfCast<u32> + Float,

        MI::SumMetric: LaplaceOuterMetric<QO>,
        <ExprDomain<MI::LazyDomain> as Domain>::Carrier: Send + Sync,

        (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
        (ExprDomain<MI::LazyDomain>, MI::SumMetric): MetricSpace,
    {
        let input_domain = try_!(input_domain.downcast_ref::<ExprDomain<MI::LazyDomain>>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<MI>()).clone();
        let scale = *try_as_ref!(scale as *const QO);
        super::make_private_mean_expr(input_domain, input_metric, scale).into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);

    let MI = input_metric.type_.clone();
    let TA = try_!(input_domain.get_active_column_type());
    let QO = try_!(Type::try_from(QO));

    dispatch!(monomorphize, [
        (MI, [L1<SymmetricDistance>, L1<InsertDeleteDistance>, SymmetricDistance, InsertDeleteDistance]), 
        (TA, @numbers),
        (QO, @floats)
    ], (input_domain, input_metric, scale)).into()
}
