use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    ffi::any::{AnyObject, Downcast},
    measures::PrivacyGuarantee,
};

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
