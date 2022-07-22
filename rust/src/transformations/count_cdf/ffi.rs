use std::{convert::TryFrom, os::raw::c_char};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast},
        util::{to_str, Type},
    },
    traits::{Float, Number, RoundCast},
    trans::{make_cdf, make_quantiles_from_counts, Interpolate},
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_cdf(T: *const c_char) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T: Float>() -> FfiResult<*mut AnyTransformation> {
        make_cdf::<T>().into_any()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [
        (T, @floats)
    ], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_quantiles_from_counts(
    bin_edges: *const AnyObject,
    alphas: *const AnyObject,
    interpolate: *const c_char,
    T: *const c_char,
    F: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T, F>(
        bin_edges: *const AnyObject,
        alphas: *const AnyObject,
        interpolate: Interpolate,
    ) -> FfiResult<*mut AnyTransformation>
    where
        T: Number + RoundCast<F>,
        F: Float + RoundCast<T>,
    {
        let bin_edges = try_!(try_as_ref!(bin_edges).downcast_ref::<Vec<T>>());
        let alphas = try_!(try_as_ref!(alphas).downcast_ref::<Vec<F>>());
        make_quantiles_from_counts::<T, F>(bin_edges.clone(), alphas.clone(), interpolate)
            .into_any()
    }
    let interpolate = match try_!(to_str(interpolate)) {
        i if i == "linear" => Interpolate::Linear,
        i if i == "nearest" => Interpolate::Nearest,
        _ => try_!(fallible!(FFI, "interpolation must be linear or nearest"))
    };
    let T = try_!(Type::try_from(T));
    let F = try_!(Type::try_from(F));
    dispatch!(monomorphize, [
        (T, @numbers),
        (F, @floats)
    ], (bin_edges, alphas, interpolate))
}
