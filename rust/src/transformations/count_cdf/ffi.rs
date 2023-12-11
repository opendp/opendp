use std::{convert::TryFrom, os::raw::c_char};

use crate::{
    core::{FfiResult, IntoAnyFunctionFfiResultExt},
    error::Fallible,
    ffi::{
        any::{AnyFunction, AnyObject, Downcast},
        util::{to_str, Type},
    },
    traits::{Float, Number, RoundCast},
    transformations::{make_cdf, make_quantiles_from_counts, Interpolation},
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_cdf(
    TA: *const c_char,
) -> FfiResult<*mut AnyFunction> {
    fn monomorphize<TA: Float>() -> Fallible<AnyFunction> {
        make_cdf::<TA>().into_any()
    }
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [
        (TA, @floats)
    ], ())
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_quantiles_from_counts(
    bin_edges: *const AnyObject,
    alphas: *const AnyObject,
    interpolation: *const c_char,
    TA: *const c_char,
    F: *const c_char,
) -> FfiResult<*mut AnyFunction> {
    fn monomorphize<TA, F>(
        bin_edges: *const AnyObject,
        alphas: *const AnyObject,
        interpolation: Interpolation,
    ) -> Fallible<AnyFunction>
    where
        TA: Number + RoundCast<F>,
        F: Float + RoundCast<TA>,
    {
        let bin_edges = try_as_ref!(bin_edges).downcast_ref::<Vec<TA>>()?;
        let alphas = try_as_ref!(alphas).downcast_ref::<Vec<F>>()?;
        make_quantiles_from_counts::<TA, F>(bin_edges.clone(), alphas.clone(), interpolation)
            .into_any()
    }
    let interpolation = match try_!(to_str(interpolation)) {
        i if i == "linear" => Interpolation::Linear,
        i if i == "nearest" => Interpolation::Nearest,
        _ => try_!(fallible!(
            FFI,
            "interpolation must be `linear` or `nearest`"
        )),
    };
    let TA = try_!(Type::try_from(TA));
    let F = try_!(Type::try_from(F));
    dispatch!(monomorphize, [
        (TA, @numbers),
        (F, @floats)
    ], (bin_edges, alphas, interpolation))
    .into()
}
