use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    ffi::{
        any::AnyMeasure,
        util::{into_c_char_p, Type, self},
    },
    measures::{FixedSmoothedMaxDivergence, MaxDivergence, ZeroConcentratedDivergence},
};

use super::SmoothedMaxDivergence;

#[bootstrap(
    name = "_measure_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_measures___measure_free(this: *mut AnyMeasure) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "measure_debug",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Debug a `measure`.
///
/// # Arguments
/// * `this` - The measure to debug (stringify).
#[no_mangle]
pub extern "C" fn opendp_measures__measure_debug(this: *mut AnyMeasure) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(format!("{:?}", this))))
}

#[bootstrap(
    name = "measure_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the type of a `measure`.
///
/// # Arguments
/// * `this` - The measure to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_measures__measure_type(this: *mut AnyMeasure) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.type_.descriptor.to_string())))
}

#[bootstrap(
    name = "measure_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the distance type of a `measure`.
///
/// # Arguments
/// * `this` - The measure to retrieve the distance type from.
#[no_mangle]
pub extern "C" fn opendp_measures__measure_distance_type(
    this: *mut AnyMeasure,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(returns(c_type = "FfiResult<AnyMeasure *>"))]
/// Construct an instance of the `MaxDivergence` measure.
///
/// # Arguments
/// * `T` - The type of the distance.
fn max_divergence<T>() -> MaxDivergence<T> {
    MaxDivergence::default()
}
#[no_mangle]
pub extern "C" fn opendp_measures__max_divergence(T: *const c_char) -> FfiResult<*mut AnyMeasure> {
    fn monomorphize<T: 'static>() -> FfiResult<*mut AnyMeasure> {
        Ok(AnyMeasure::new(max_divergence::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], ())
}

#[bootstrap(returns(c_type = "FfiResult<AnyMeasure *>"))]
/// Construct an instance of the `SmoothedMaxDivergence` measure.
///
/// # Arguments
/// * `T` - The type of the distance.
fn smoothed_max_divergence<T>() -> SmoothedMaxDivergence<T> {
    SmoothedMaxDivergence::default()
}
#[no_mangle]
pub extern "C" fn opendp_measures__smoothed_max_divergence(
    T: *const c_char,
) -> FfiResult<*mut AnyMeasure> {
    fn monomorphize<T: 'static>() -> FfiResult<*mut AnyMeasure> {
        Ok(AnyMeasure::new(smoothed_max_divergence::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], ())
}

#[bootstrap(returns(c_type = "FfiResult<AnyMeasure *>"))]
/// Construct an instance of the `FixedSmoothedMaxDivergence` measure.
///
/// # Arguments
/// * `T` - The atomic type of the distance.
fn fixed_smoothed_max_divergence<T>() -> FixedSmoothedMaxDivergence<T> {
    FixedSmoothedMaxDivergence::default()
}

#[no_mangle]
pub extern "C" fn opendp_measures__fixed_smoothed_max_divergence(
    T: *const c_char,
) -> FfiResult<*mut AnyMeasure> {
    fn monomorphize<T: 'static>() -> FfiResult<*mut AnyMeasure> {
        Ok(AnyMeasure::new(fixed_smoothed_max_divergence::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], ())
}

#[bootstrap(returns(c_type = "FfiResult<AnyMeasure *>"))]
/// Construct an instance of the `ZeroConcentratedDivergence` measure.
///
/// # Arguments
/// * `T` - The type of the distance.
fn zero_concentrated_divergence<T>() -> ZeroConcentratedDivergence<T> {
    ZeroConcentratedDivergence::default()
}

#[no_mangle]
pub extern "C" fn opendp_measures__zero_concentrated_divergence(
    T: *const c_char,
) -> FfiResult<*mut AnyMeasure> {
    fn monomorphize<T: 'static>() -> FfiResult<*mut AnyMeasure> {
        Ok(AnyMeasure::new(zero_concentrated_divergence::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], ())
}
