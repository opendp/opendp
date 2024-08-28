use std::{ffi::c_char, fmt::Debug, marker::PhantomData};

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measure},
    error::Fallible,
    ffi::{
        any::AnyMeasure,
        util::{self, into_c_char_p, to_str, ExtrinsicObject, Type},
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

#[derive(Clone, Default)]
pub struct UserDivergence {
    pub descriptor: String,
}

impl std::fmt::Debug for UserDivergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserDivergence({:?})", self.descriptor)
    }
}

impl PartialEq for UserDivergence {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor == other.descriptor
    }
}

impl Measure for UserDivergence {
    type Distance = ExtrinsicObject;
}

#[bootstrap(
    name = "user_divergence",
    features("honest-but-curious"),
    arguments(descriptor(rust_type = "String"))
)]
/// Construct a new UserDivergence.
/// Any two instances of an UserDivergence are equal if their string descriptors are equal.
///
/// # Arguments
/// * `descriptor` - A string description of the privacy measure.
#[no_mangle]
pub extern "C" fn opendp_measures__user_divergence(
    descriptor: *mut c_char,
) -> FfiResult<*mut AnyMeasure> {
    let descriptor = try_!(to_str(descriptor)).to_string();
    Ok(AnyMeasure::new(UserDivergence { descriptor })).into()
}

pub struct TypedMeasure<Q> {
    pub measure: AnyMeasure,
    marker: PhantomData<fn() -> Q>,
}

impl<Q: 'static> TypedMeasure<Q> {
    pub fn new(measure: AnyMeasure) -> Fallible<TypedMeasure<Q>> {
        if measure.distance_type != Type::of::<Q>() {
            return fallible!(
                FFI,
                "unexpected distance type in measure. Expected {}, got {}",
                Type::of::<Q>().to_string(),
                measure.distance_type.to_string()
            );
        }

        Ok(TypedMeasure {
            measure,
            marker: PhantomData,
        })
    }
}

impl<Q> PartialEq for TypedMeasure<Q> {
    fn eq(&self, other: &Self) -> bool {
        self.measure == other.measure
    }
}

impl<Q> Clone for TypedMeasure<Q> {
    fn clone(&self) -> Self {
        Self {
            measure: self.measure.clone(),
            marker: self.marker.clone(),
        }
    }
}

impl<Q> Debug for TypedMeasure<Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.measure)
    }
}
impl<Q> Default for TypedMeasure<Q> {
    fn default() -> Self {
        panic!()
    }
}

impl<Q> Measure for TypedMeasure<Q> {
    type Distance = Q;
}
