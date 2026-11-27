use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    ffi::any::{AnyObject, Downcast},
    measures::PrivacyGuarantee,
};

#[bootstrap(
    name = "_privacy_guarantee_with_gaussianDP",
    features("contrib", "idealized-numerics"),
    arguments(this(rust_type = "PrivacyGuarantee")),
    returns(hint = "PrivacyGuarantee")
)]
/// Attach the Gaussian-DP representation corresponding to `mu`.
#[cfg(feature = "idealized-numerics")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_with_gaussianDP(
    this: *const AnyObject,
    mu: f64,
) -> FfiResult<*mut AnyObject> {
    let this = try_!(try_as_ref!(this).downcast_ref::<PrivacyGuarantee>()).clone();
    FfiResult::Ok(AnyObject::new_raw(try_!(this.with_gaussianDP(mu))))
}
