use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    ffi::any::{AnyObject, Downcast},
    measures::PrivacyGuarantee,
};

#[bootstrap(
    name = "_privacy_guarantee_with_zCDP",
    features("contrib"),
    arguments(this(rust_type = "PrivacyGuarantee"), delta(default = 0.0)),
    returns(rust_type = "PrivacyGuarantee")
)]
/// Attach a zCDP representation to a PrivacyGuarantee.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_with_zCDP(
    this: *const AnyObject,
    rho: f64,
    delta: f64,
) -> FfiResult<*mut AnyObject> {
    let this = try_!(try_as_ref!(this).downcast_ref::<PrivacyGuarantee>()).clone();
    FfiResult::Ok(AnyObject::new_raw(try_!(this.with_zCDP(rho, delta))))
}
