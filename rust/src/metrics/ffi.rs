use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    ffi::{
        any::AnyMetric,
        util::{self, into_c_char_p, Type},
    },
    metrics::{AbsoluteDistance, L1Distance, L2Distance},
};

use super::{
    ChangeOneDistance, DiscreteDistance, HammingDistance, InsertDeleteDistance, SymmetricDistance,
};
#[bootstrap(
    name = "_metric_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_core___metric_free(this: *mut AnyMetric) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "metric_debug",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Debug a `metric`.
///
/// # Arguments
/// * `this` - The metric to debug (stringify).
#[no_mangle]
pub extern "C" fn opendp_metrics__metric_debug(this: *mut AnyMetric) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(format!("{:?}", this))))
}

#[bootstrap(
    name = "metric_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the type of a `metric`.
///
/// # Arguments
/// * `this` - The metric to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_metrics__metric_type(this: *mut AnyMetric) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.type_.descriptor.to_string())))
}

#[bootstrap(
    name = "metric_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the distance type of a `metric`.
///
/// # Arguments
/// * `this` - The metric to retrieve the distance type from.
#[no_mangle]
pub extern "C" fn opendp_metrics__metric_distance_type(
    this: *mut AnyMetric,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "symmetric_distance",
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Construct an instance of the `SymmetricDistance` metric.
#[no_mangle]
pub extern "C" fn opendp_metrics__symmetric_distance() -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(SymmetricDistance::default())))
}

#[bootstrap(
    name = "insert_delete_distance",
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Construct an instance of the `InsertDeleteDistance` metric.
#[no_mangle]
pub extern "C" fn opendp_metrics__insert_delete_distance() -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(
        InsertDeleteDistance::default(),
    )))
}

#[bootstrap(
    name = "change_one_distance",
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Construct an instance of the `ChangeOneDistance` metric.
#[no_mangle]
pub extern "C" fn opendp_metrics__change_one_distance() -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(ChangeOneDistance::default())))
}

#[bootstrap(name = "hamming_distance", returns(c_type = "FfiResult<AnyMetric *>"))]
/// Construct an instance of the `HammingDistance` metric.
#[no_mangle]
pub extern "C" fn opendp_metrics__hamming_distance() -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(HammingDistance::default())))
}

#[bootstrap(returns(c_type = "FfiResult<AnyMetric *>"))]
/// Construct an instance of the `AbsoluteDistance` metric.
///
/// # Arguments
/// * `T` - The type of the distance.
fn absolute_distance<T>() -> AbsoluteDistance<T> {
    AbsoluteDistance::default()
}

#[no_mangle]
pub extern "C" fn opendp_metrics__absolute_distance(T: *const c_char) -> FfiResult<*mut AnyMetric> {
    fn monomorphize<T: 'static>() -> FfiResult<*mut AnyMetric> {
        Ok(AnyMetric::new(absolute_distance::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], ())
}

#[bootstrap(returns(c_type = "FfiResult<AnyMetric *>"))]
/// Construct an instance of the `L1Distance` metric.
///
/// # Arguments
/// * `T` - The type of the distance.
fn l1_distance<T>() -> L1Distance<T> {
    L1Distance::default()
}
#[no_mangle]
pub extern "C" fn opendp_metrics__l1_distance(T: *const c_char) -> FfiResult<*mut AnyMetric> {
    fn monomorphize<T: 'static>() -> FfiResult<*mut AnyMetric> {
        Ok(AnyMetric::new(l1_distance::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], ())
}

#[bootstrap(returns(c_type = "FfiResult<AnyMetric *>"))]
/// Construct an instance of the `L2Distance` metric.
///
/// # Arguments
/// * `T` - The type of the distance.
fn l2_distance<T>() -> L2Distance<T> {
    L2Distance::default()
}
#[no_mangle]
pub extern "C" fn opendp_metrics__l2_distance(T: *const c_char) -> FfiResult<*mut AnyMetric> {
    fn monomorphize<T: 'static>() -> FfiResult<*mut AnyMetric> {
        Ok(AnyMetric::new(l2_distance::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], ())
}

#[bootstrap(name = "discrete_distance", returns(c_type = "FfiResult<AnyMetric *>"))]
/// Construct an instance of the `DiscreteDistance` metric.
#[no_mangle]
pub extern "C" fn opendp_metrics__discrete_distance() -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(DiscreteDistance::default())))
}
