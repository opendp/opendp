use opendp_derive::bootstrap;
use polars::lazy::frame::LazyFrame;
use std::ffi::c_char;

use crate::{
    core::{FfiResult, FfiSlice},
    error::Fallible,
    ffi::any::{AnyObject, AnyQueryable, Downcast},
    ffi::util::into_c_char_p,
    measurements::expr_noise::NoisePlugin,
    polars::{ExtractLazyFrame, OnceFrameAnswer, OnceFrameQuery},
};

#[cfg(test)]
mod test;

#[bootstrap(
    name = "decode_noise_plugin_parameters",
    arguments(kwargs(rust_type = "Vec<u8>", hint = "Sequence[int]")),
    returns(c_type = "FfiResult<char *>")
)]
/// Internal function. Decodes serialized noise plugin parameters as JSON.
///
/// Uses the same typed, data-only deserializer as the Rust noise plugin,
/// without invoking Python's pickle deserializer.
///
/// # Arguments
/// * `kwargs` - The serialized keyword arguments of an OpenDP noise plugin.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__decode_noise_plugin_parameters(
    kwargs: *const FfiSlice,
) -> FfiResult<*mut c_char> {
    let kwargs = try_as_ref!(kwargs);
    if kwargs.ptr.is_null() || kwargs.len > isize::MAX as usize {
        return err!(FFI, "Invalid noise plugin parameter buffer").into();
    }
    // SAFETY: The FFI caller must provide a valid byte buffer of length kwargs.len
    // that remains alive for this call. The Python binding owns that buffer.
    let kwargs = unsafe { std::slice::from_raw_parts(kwargs.ptr as *const u8, kwargs.len) };
    let parameters = try_!(decode_noise_plugin_parameters(kwargs));
    FfiResult::Ok(try_!(into_c_char_p(parameters)))
}

fn decode_noise_plugin_parameters(kwargs: &[u8]) -> Fallible<String> {
    let parameters: NoisePlugin = serde_pickle::from_slice(kwargs, Default::default())
        .map_err(|_| err!(FailedFunction, "Invalid noise plugin parameters"))?;

    // JSON cannot represent non-finite floats: serde_json would emit null.
    // Reject them explicitly rather than changing the decoded scale's type.
    if !parameters.scale.is_finite() {
        return fallible!(FailedFunction, "Noise plugin scale must be finite");
    }

    serde_json::to_string(&parameters).map_err(|_| {
        err!(
            FailedFunction,
            "Unable to serialize noise plugin parameters"
        )
    })
}

#[bootstrap(
    name = "onceframe_collect",
    arguments(onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"))
)]
/// Internal function. Collects a DataFrame from a OnceFrame, exhausting the OnceFrame.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__onceframe_collect(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let query = AnyObject::new(OnceFrameQuery::Collect);
    let answer: OnceFrameAnswer = try_!(try_!(queryable.eval(&query)).downcast());
    let OnceFrameAnswer::Collect(frame) = answer;

    Ok(AnyObject::new(frame)).into()
}

#[bootstrap(
    features("honest-but-curious"),
    name = "onceframe_lazy",
    arguments(onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"))
)]
/// Internal function. Extracts a LazyFrame from a OnceFrame,
/// circumventing protections against multiple evaluations.
///
/// Each collection consumes the entire allocated privacy budget.
/// To remain DP at the advertised privacy level, only collect the LazyFrame once.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
///
/// # Why honest-but-curious?
/// The privacy guarantees only apply if:
///
/// 1. The LazyFrame (compute plan) is only ever executed once.
/// 2. The analyst does not observe ordering of rows in the output.
///
/// To ensure that row ordering is not observed:
///
/// 1. Do not extend the compute plan with order-sensitive computations.
/// 2. Shuffle the output once collected ([in Polars sample all, with shuffling enabled](https://docs.pola.rs/api/python/stable/reference/dataframe/api/polars.DataFrame.sample.html)).
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__onceframe_lazy(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let answer: LazyFrame = try_!(queryable.eval_internal(&ExtractLazyFrame));
    Ok(AnyObject::new(answer)).into()
}
