use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::domains::{LazyFrameDomain, VectorDomain, AtomDomain};
use crate::err;
use crate::transformations::{make_select_column, DatasetMetric};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace};
use crate::ffi::any::{AnyTransformation, Downcast, AnyDomain, AnyMetric};
use crate::ffi::util::{Type, self};
use crate::traits::Primitive;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_select_column(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    key: *const c_char,
    TOA: *const c_char,
    M: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TOA, M>(input_domain: &AnyDomain, input_metric: &AnyMetric, key: &str) -> FfiResult<*mut AnyTransformation>
    where
        TOA: Primitive,
        M: DatasetMetric,
        (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
    {
        let input_domain: LazyFrameDomain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
        let input_metric: M = try_!(try_as_ref!(input_metric).downcast_ref::<M>()).clone();
        make_select_column::<TOA, M>(input_domain, input_metric, key).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TOA = try_!(Type::try_from(TOA));
    let M = try_!(Type::try_from(M));
    let key = try_!(util::to_str(key));

    dispatch!(monomorphize, [
        (TOA, @primitives),
        (M, @dataset_metrics)
    ], (input_domain, input_metric, key))
}
