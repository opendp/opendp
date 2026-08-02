#[cfg(feature = "honest-but-curious")]
use opendp_derive::bootstrap;

#[cfg(feature = "honest-but-curious")]
use crate::ffi::any::{CallbackFn, Downcast, wrap_func};

#[cfg(feature = "honest-but-curious")]
use crate::{core::FfiResult, ffi::any::AnyObject, measures::PrivacyGuarantee};

#[bootstrap(
    name = "_privacy_guarantee_with_renyiDP",
    features("contrib", "honest-but-curious"),
    arguments(
        this(rust_type = "PrivacyGuarantee"),
        curve(rust_type = "f64"),
        delta(default = 0.0)
    ),
    returns(rust_type = "PrivacyGuarantee")
)]
/// Attach an RDP representation to a PrivacyGuarantee.
///
/// # Why honest-but-curious?
/// The callback must define a valid RDP curve and source delta guarantee.
#[cfg(feature = "honest-but-curious")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_with_renyiDP(
    this: *const AnyObject,
    curve: *const CallbackFn,
    delta: f64,
) -> FfiResult<*mut AnyObject> {
    let this = try_!(try_as_ref!(this).downcast_ref::<PrivacyGuarantee>()).clone();
    let curve = wrap_func(try_as_ref!(curve).clone());
    let curve = move |alpha: f64| curve(&AnyObject::new(alpha))?.downcast::<f64>();
    FfiResult::Ok(AnyObject::new_raw(try_!(this.with_renyiDP(curve, delta))))
}
