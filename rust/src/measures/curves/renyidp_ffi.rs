use opendp_derive::bootstrap;

use crate::ffi::any::{CallbackFn, Downcast, wrap_func};
use crate::{core::FfiResult, ffi::any::AnyObject, measures::PrivacyGuarantee};

#[bootstrap(
    name = "_privacy_guarantee_with_renyiDP",
    features("contrib"),
    arguments(
        this(rust_type = "PrivacyGuarantee"),
        curve(rust_type = "Vec<f64>"),
        delta(default = 0.0)
    ),
    returns(rust_type = "PrivacyGuarantee")
)]
/// Attach a vector-valued RDP representation to a PrivacyGuarantee.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_with_renyiDP(
    this: *const AnyObject,
    curve: *const CallbackFn,
    delta: f64,
) -> FfiResult<*mut AnyObject> {
    let this = try_!(try_as_ref!(this).downcast_ref::<PrivacyGuarantee>()).clone();
    let curve = wrap_func(try_as_ref!(curve).clone());
    let curve =
        move |orders: &[f64]| curve(&AnyObject::new(orders.to_vec()))?.downcast::<Vec<f64>>();
    FfiResult::Ok(AnyObject::new_raw(try_!(this.with_renyiDP(curve, delta))))
}
