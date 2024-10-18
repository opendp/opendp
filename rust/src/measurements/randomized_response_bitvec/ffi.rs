use std::ffi::c_double;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt},
    domains::{BitVector, BitVectorDomain},
    ffi::{
        any::{AnyDomain, AnyMeasurement, AnyMetric, AnyObject, Downcast},
        util::{c_bool, to_bool},
    },
    metrics::DiscreteDistance,
};

use super::{debias_randomized_response_bitvec, make_randomized_response_bitvec};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_randomized_response_bitvec(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    f: c_double,
    constant_time: c_bool,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<BitVectorDomain>()).clone();
    let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<DiscreteDistance>()).clone();

    make_randomized_response_bitvec(input_domain, input_metric, f, to_bool(constant_time))
        .into_any()
        .into()
}

#[no_mangle]
pub extern "C" fn opendp_measurements__debias_randomized_response_bitvec(
    answers: *const AnyObject,
    f: c_double,
) -> FfiResult<*mut AnyObject> {
    let answers = try_!(try_as_ref!(answers).downcast_ref::<Vec<*const AnyObject>>()).clone();
    let answers: Vec<BitVector> = try_!(answers
        .into_iter()
        .map(|ptr| try_as_ref!(ptr).clone().downcast::<BitVector>())
        .collect());

    debias_randomized_response_bitvec(answers, f)
        .map(AnyObject::new)
        .into()
}
