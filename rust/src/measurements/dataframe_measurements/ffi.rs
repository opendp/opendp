
use std::ffi::c_double;
use std::os::raw::{c_long};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::domains::{ LazyFrameDomain};
use crate::ffi::any::{AnyMeasurement, AnyDomain, AnyMetric, Downcast};
use crate::measurements::{ make_polarsDF_laplace};
use crate::metrics::L1Distance;
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_polarsDF_laplace(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: c_double,
    k: c_long,
) -> FfiResult<*mut AnyMeasurement> {
 
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<L1Distance::<f64>>()).clone();
    let k = k as i32;

   make_polarsDF_laplace(
        input_domain,
        input_metric,
        scale,
        Some(k)
    ).into_any()

}