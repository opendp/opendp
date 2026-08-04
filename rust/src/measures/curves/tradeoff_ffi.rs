use opendp_derive::bootstrap;

#[cfg(feature = "honest-but-curious")]
use crate::ffi::any::{CallbackFn, wrap_func};

use crate::{
    core::FfiResult,
    ffi::any::{AnyObject, Downcast},
    measures::PrivacyGuarantee,
};

#[bootstrap(
    name = "_privacy_guarantee_with_tradeoff",
    features("contrib", "honest-but-curious"),
    arguments(this(rust_type = "PrivacyGuarantee"), curve(rust_type = "f64")),
    returns(rust_type = "PrivacyGuarantee")
)]
/// Attach an f-DP tradeoff representation to a PrivacyGuarantee.
///
/// The callback must be functionally pure and return finite values in `[0, 1]`
/// that are nonincreasing and convex on `[0, 1]`. Numerically approximate
/// callbacks must be downward-conservative. Set `symmetric` to `true` only for
/// a genuinely symmetric tradeoff curve. These properties are not validated at
/// runtime.
///
/// # Why honest-but-curious?
/// The callback is user-supplied code invoked during privacy queries. OpenDP
/// can safely use its result as a privacy lower bound only when the callback
/// obeys this contract; enforcing it would require observing arbitrary code
/// and cannot be certified by the FFI boundary.
#[cfg(feature = "honest-but-curious")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_with_tradeoff(
    this: *const AnyObject,
    curve: *const CallbackFn,
    symmetric: bool,
) -> FfiResult<*mut AnyObject> {
    let this = try_!(try_as_ref!(this).downcast_ref::<PrivacyGuarantee>()).clone();
    let curve = wrap_func(try_as_ref!(curve).clone());
    let curve = move |alpha: f64| curve(&AnyObject::new(alpha))?.downcast::<f64>();
    FfiResult::Ok(AnyObject::new_raw(try_!(if symmetric {
        this.with_symmetric_tradeoff(curve)
    } else {
        this.with_tradeoff(curve)
    })))
}

#[bootstrap(
    name = "_privacy_guarantee_beta",
    arguments(curve(rust_type = "PrivacyGuarantee")),
    returns(hint = "float")
)]
/// Query beta(alpha) from a PrivacyGuarantee.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_beta(
    curve: *const AnyObject,
    alpha: f64,
) -> FfiResult<*mut AnyObject> {
    try_!(try_as_ref!(curve).downcast_ref::<PrivacyGuarantee>())
        .beta(alpha)
        .map(AnyObject::new)
        .into()
}

#[bootstrap(
    name = "_privacy_guarantee_alpha",
    arguments(curve(rust_type = "PrivacyGuarantee"), beta(rust_type = "f64")),
    returns(hint = "float")
)]
/// Query alpha(beta) from a PrivacyGuarantee.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_alpha(
    curve: *const AnyObject,
    beta: f64,
) -> FfiResult<*mut AnyObject> {
    try_!(try_as_ref!(curve).downcast_ref::<PrivacyGuarantee>())
        .alpha(beta)
        .map(AnyObject::new)
        .into()
}
