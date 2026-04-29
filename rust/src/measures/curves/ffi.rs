use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    ffi::any::{AnyObject, CallbackFn, Downcast, wrap_func},
    measures::PrivacyCurve,
};

#[bootstrap(
    name = "new_privacy_profile",
    features("contrib", "honest-but-curious"),
    arguments(curve(rust_type = "f64"), calibration(rust_type = "bool")),
    returns(rust_type = "PrivacyCurve")
)]
#[deprecated(
    since = "0.15.0",
    note = "For Python, use `dp.PrivacyCurve.new_profile`. For R, use `privacy_curve(profile = _)`."
)]
/// Deprecated alias.
///
/// # Why honest-but-curious?
///
/// The privacy profile should implement a well-defined $\delta(\epsilon)$ curve:
///
/// * monotonically decreasing
/// * rejects epsilon values that are less than zero or nan
/// * returns delta values only within $[0, 1]$
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__new_privacy_profile(
    curve: *const CallbackFn,
) -> FfiResult<*mut AnyObject> {
    opendp_measures___privacy_curve_new_profile(curve)
}

#[bootstrap(
    name = "_privacy_curve_new_profile",
    features("contrib", "honest-but-curious"),
    arguments(curve(rust_type = "f64")),
    returns(rust_type = "PrivacyCurve")
)]
/// Construct a PrivacyCurve from a user-defined callback.
///
/// # Arguments
/// * `curve` - A privacy curve mapping epsilon to delta
///
/// # Why honest-but-curious?
///
/// The privacy profile should implement a well-defined $\delta(\epsilon)$ curve:
///
/// * monotonically decreasing
/// * rejects epsilon values that are less than zero or nan
/// * returns delta values only within $[0, 1]$
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_curve_new_profile(
    curve: *const CallbackFn,
) -> FfiResult<*mut AnyObject> {
    let curve = wrap_func(try_as_ref!(curve).clone());
    FfiResult::Ok(AnyObject::new_raw(PrivacyCurve::new_profile(
        move |epsilon: f64| curve(&AnyObject::new(epsilon))?.downcast::<f64>(),
    )))
}

#[bootstrap(
    name = "_privacy_curve_new_tradeoff",
    features("contrib", "honest-but-curious"),
    arguments(curve(rust_type = "f64")),
    returns(rust_type = "PrivacyCurve")
)]
/// Construct a PrivacyCurve from a user-defined callback.
///
/// # Arguments
/// * `curve` - An $f$-DP tradeoff curve mapping alpha to beta
///
/// # Why honest-but-curious?
///
/// The tradeoff curve should implement a well-defined $\beta(\alpha)$ curve.
/// In particular, canonical-noise sampling assumes the following requirements:
///
/// * accepts finite alpha values on $[0, 1]$
/// * returns finite beta values only within $[0, 1]$
/// * is nonincreasing on $[0, 1]$
/// * is symmetric in the sense that $\beta(\beta(\alpha)) = \alpha$
/// * has a fixed point $c$ with $c < 1/2$
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_curve_new_tradeoff(
    curve: *const CallbackFn,
) -> FfiResult<*mut AnyObject> {
    let curve = wrap_func(try_as_ref!(curve).clone());
    FfiResult::Ok(AnyObject::new_raw(PrivacyCurve::new_tradeoff(
        move |alpha: f64| curve(&AnyObject::new(alpha))?.downcast::<f64>(),
    )))
}

#[bootstrap(
    name = "_privacy_curve_delta",
    arguments(curve(rust_type = b"null"), delta(rust_type = "f64"))
)]
/// Internal function. Use a PrivacyCurve to find delta at a given `epsilon`.
///
/// # Arguments
/// * `curve` - The PrivacyCurve.
/// * `epsilon` - What to fix epsilon to compute delta.
///
/// # Returns
/// Delta at a given `epsilon`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_curve_delta(
    curve: *const AnyObject,
    epsilon: f64,
) -> FfiResult<*mut AnyObject> {
    try_!(try_as_ref!(curve).downcast_ref::<PrivacyCurve>())
        .delta(epsilon)
        .map(AnyObject::new)
        .into()
}

#[bootstrap(
    name = "_privacy_curve_epsilon",
    arguments(profile(rust_type = b"null"), delta(rust_type = "f64"))
)]
/// Internal function. Use a PrivacyCurve to find epsilon at a given `delta`.
///
/// # Arguments
/// * `profile` - The PrivacyCurve.
/// * `delta` - What to fix delta to compute epsilon.
///
/// # Returns
/// Epsilon at a given `delta`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_curve_epsilon(
    profile: *const AnyObject,
    delta: f64,
) -> FfiResult<*mut AnyObject> {
    try_!(try_as_ref!(profile).downcast_ref::<PrivacyCurve>())
        .epsilon(delta)
        .map(AnyObject::new)
        .into()
}

#[bootstrap(
    name = "_privacy_curve_beta",
    arguments(curve(rust_type = b"null"), alpha(rust_type = "f64"))
)]
/// Internal function. Use a PrivacyCurve to find beta at a given alpha.
///
/// # Arguments
/// * `curve` - The PrivacyCurve.
/// * `alpha` - What to fix alpha to compute beta.
///
/// # Returns
/// Beta at a given `alpha`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_curve_beta(
    curve: *const AnyObject,
    alpha: f64,
) -> FfiResult<*mut AnyObject> {
    try_!(try_as_ref!(curve).downcast_ref::<PrivacyCurve>())
        .beta(alpha)
        .map(AnyObject::new)
        .into()
}

#[bootstrap(
    name = "_privacy_curve_new_approxdp",
    features("contrib"),
    arguments(pairs(rust_type = "Vec<(f64, f64)>")),
    returns(hint = "PrivacyCurve")
)]
/// Construct an (ε, δ)-DP privacy profile from epsilon-delta pairs.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_curve_new_approxdp(
    pairs: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let pairs = try_!(try_as_ref!(pairs).downcast_ref::<Vec<(f64, f64)>>()).clone();
    FfiResult::Ok(AnyObject::new_raw(try_!(PrivacyCurve::new_approxdp(pairs))))
}

#[bootstrap(
    name = "_privacy_curve_new_gdp",
    features("contrib"),
    arguments(mu(rust_type = "f64")),
    returns(hint = "PrivacyCurve")
)]
/// Construct the tradeoff curve corresponding to Gaussian differential privacy with parameter `mu`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_curve_new_gdp(mu: f64) -> FfiResult<*mut AnyObject> {
    FfiResult::Ok(AnyObject::new_raw(try_!(PrivacyCurve::new_gdp(mu))))
}
