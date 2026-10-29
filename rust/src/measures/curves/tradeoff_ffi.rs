use opendp_derive::bootstrap;

use crate::ffi::any::{CallbackFn, wrap_func};

use crate::{
    core::FfiResult,
    ffi::any::{AnyObject, Downcast},
    measures::PrivacyGuarantee,
};

#[bootstrap(
    name = "_privacy_guarantee_with_tradeoff",
    features("contrib"),
    arguments(this(rust_type = "PrivacyGuarantee"), curve(rust_type = "Vec<f64>")),
    returns(rust_type = "PrivacyGuarantee")
)]
/// Attach a vector-valued f-DP tradeoff representation to a PrivacyGuarantee.
/// Inputs must be nondecreasing, permitting duplicates; outputs must match
/// their length and lie in [0, 1]. Only these numerical checks are enforced.
///
/// The callback must be functionally pure. Its actual numerical values must
/// agree with a continuous, convex, nonincreasing function on the entire real
/// interval `[0, 1]`, taking values in `[0, 1]` and lower-bounding the true
/// tradeoff throughout that interval. Convexity of an ideal counterpart or
/// pointwise downward rounding alone is insufficient. Set `symmetric` to
/// `true` only for a genuinely symmetric TRUE tradeoff; the numerical lower
/// bound need not itself be symmetric. These assumptions are not validated at
/// runtime; violating them can invalidate privacy bounds.
///
/// These honest-but-curious preconditions apply when converting the callback;
/// attaching a representation does not certify its mathematical truth.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_with_tradeoff(
    this: *const AnyObject,
    curve: *const CallbackFn,
    symmetric: bool,
) -> FfiResult<*mut AnyObject> {
    let this = try_!(try_as_ref!(this).downcast_ref::<PrivacyGuarantee>()).clone();
    let curve = wrap_func(try_as_ref!(curve).clone());
    let curve = move |alpha: &[f64]| curve(&AnyObject::new(alpha.to_vec()))?.downcast::<Vec<f64>>();
    FfiResult::Ok(AnyObject::new_raw(try_!(if symmetric {
        this.with_symmetric_tradeoff(curve)
    } else {
        this.with_tradeoff(curve)
    })))
}

#[bootstrap(
    name = "_privacy_guarantee_beta",
    arguments(curve(rust_type = "PrivacyGuarantee"), alpha(rust_type = "Vec<f64>"))
)]
/// Query beta(alpha) from a PrivacyGuarantee.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_beta(
    curve: *const AnyObject,
    alpha: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    try_!(try_as_ref!(curve).downcast_ref::<PrivacyGuarantee>())
        .beta(try_!(try_as_ref!(alpha).downcast_ref::<Vec<f64>>()))
        .map(AnyObject::new)
        .into()
}

#[bootstrap(
    name = "_privacy_guarantee_alpha",
    arguments(curve(rust_type = "PrivacyGuarantee"), beta(rust_type = "Vec<f64>"))
)]
/// Query alpha(beta) from a PrivacyGuarantee.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_alpha(
    curve: *const AnyObject,
    beta: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    try_!(try_as_ref!(curve).downcast_ref::<PrivacyGuarantee>())
        .alpha(try_!(try_as_ref!(beta).downcast_ref::<Vec<f64>>()))
        .map(AnyObject::new)
        .into()
}
