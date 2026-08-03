use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    ffi::any::{AnyObject, CallbackFn, Downcast, wrap_func},
    measures::PrivacyGuarantee,
};

#[bootstrap(
    name = "new_privacy_profile",
    features("contrib", "honest-but-curious"),
    arguments(curve(rust_type = "f64")),
    returns(rust_type = "PrivacyGuarantee")
)]
#[deprecated(
    since = "0.15.0",
    note = "For Python, use `dp.PrivacyGuarantee(profile=_)`. For R, use `privacy_guarantee(profile = _)`."
)]
/// Deprecated alias.
///
/// # Arguments
/// * `curve` - A privacy profile mapping epsilon to delta
///
/// # Why honest-but-curious?
///
/// For Python, see `dp.PrivacyGuarantee(profile=_)`. For R, see `privacy_guarantee(profile = _)`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__new_privacy_profile(
    curve: *const CallbackFn,
) -> FfiResult<*mut AnyObject> {
    let curve = wrap_func(try_as_ref!(curve).clone());
    let curve = move |epsilon: f64| curve(&AnyObject::new(epsilon))?.downcast::<f64>();
    let curve = try_!(PrivacyGuarantee::new().with_profile(curve));
    FfiResult::Ok(AnyObject::new_raw(curve))
}

#[bootstrap(
    name = "_new_privacy_guarantee",
    features("contrib"),
    returns(rust_type = "PrivacyGuarantee")
)]
/// Construct a new empty PrivacyGuarantee.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___new_privacy_guarantee() -> FfiResult<*mut AnyObject> {
    FfiResult::Ok(AnyObject::new_raw(PrivacyGuarantee::new()))
}

#[bootstrap(
    name = "_privacy_guarantee_with_profile",
    features("contrib", "honest-but-curious"),
    arguments(
        this(rust_type = "PrivacyGuarantee"),
        curve(rust_type = "f64"),
        log(default = false)
    ),
    returns(rust_type = "PrivacyGuarantee")
)]
/// Attach a privacy-profile representation to a PrivacyGuarantee.
///
/// For tight conversion to f-DP, the profile should also preserve the
/// hockey-stick structure of true privacy profiles:
///
/// * λ ↦ δ(log λ) is convex and nonincreasing for λ >= 1
///
/// If this property is not satisfied, `beta(alpha)` remains conservative,
/// but may be loose because the optimizer may miss the best epsilon.
///
/// # Arguments
/// * `this` - The PrivacyGuarantee to extend.
/// * `curve` - A privacy profile mapping epsilon to delta
/// * `log` - Whether the profile returns log(delta), or delta
///
/// # Why honest-but-curious?
///
/// The privacy profile should implement a well-defined $\delta(\epsilon)$ curve:
///
/// * is functionally pure
/// * nonincreasing
/// * returns delta values only within $[0, 1]$
/// * returned values are upward-conservative if numerically approximate
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_with_profile(
    this: *const AnyObject,
    curve: *const CallbackFn,
    log: bool,
) -> FfiResult<*mut AnyObject> {
    let this = try_!(try_as_ref!(this).downcast_ref::<PrivacyGuarantee>()).clone();
    let curve = wrap_func(try_as_ref!(curve).clone());
    let curve = move |epsilon: f64| curve(&AnyObject::new(epsilon))?.downcast::<f64>();
    FfiResult::Ok(AnyObject::new_raw(try_!(if log {
        this.with_log_profile(curve)
    } else {
        this.with_profile(curve)
    })))
}

#[bootstrap(
    name = "_privacy_guarantee_with_tradeoff",
    features("contrib", "honest-but-curious"),
    arguments(this(rust_type = "PrivacyGuarantee"), curve(rust_type = "f64")),
    returns(rust_type = "PrivacyGuarantee")
)]
/// Attach a tradeoff representation to a PrivacyGuarantee.
///
/// # Arguments
/// * `this` - The PrivacyGuarantee to extend.
/// * `curve` - An $f$-DP tradeoff curve mapping alpha to beta
/// * `symmetric` - Indicates whether the tradeoff curve is symmetric
///
/// # Why honest-but-curious?
///
/// The tradeoff curve should implement a well-defined $\beta(\alpha)$ curve.
///
/// * is functionally pure
/// * returns finite beta values in [0, 1]
/// * satisfies beta(0) = 1 and beta(1) = 0
/// * is nonincreasing and convex on [0, 1]
/// * returns downward-conservative beta values if numerically approximate
/// * if symmetric, then $\beta(\beta(\alpha)) = \alpha$
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
    name = "_privacy_guarantee_with_approxDP",
    features("contrib"),
    arguments(
        this(rust_type = "PrivacyGuarantee"),
        pairs(rust_type = "Vec<(f64, f64)>")
    ),
    returns(hint = "PrivacyGuarantee")
)]
/// Attach an (ε, δ)-DP representation to a PrivacyGuarantee.
///
/// # Arguments
/// * `this` - The PrivacyGuarantee to extend.
/// * `pairs` - a vector of approx-DP pairs
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_with_approxDP(
    this: *const AnyObject,
    pairs: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let this = try_!(try_as_ref!(this).downcast_ref::<PrivacyGuarantee>()).clone();
    let pairs = try_!(try_as_ref!(pairs).downcast_ref::<Vec<(f64, f64)>>()).clone();
    FfiResult::Ok(AnyObject::new_raw(try_!(this.with_approxDP(pairs))))
}

#[bootstrap(
    name = "_privacy_guarantee_with_gaussianDP",
    features("contrib", "idealized-numerics"),
    arguments(this(rust_type = "PrivacyGuarantee")),
    returns(hint = "PrivacyGuarantee")
)]
/// Attach the GDP representation corresponding to parameter `mu` to a PrivacyGuarantee.
///
/// # Why idealized-numerics?
/// While the calculations have best-effort protections against float underestimation,
/// error bounds for transcendentals like `erfcx` are not known and could be underestimated.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_with_gaussianDP(
    this: *const AnyObject,
    mu: f64,
) -> FfiResult<*mut AnyObject> {
    let this = try_!(try_as_ref!(this).downcast_ref::<PrivacyGuarantee>()).clone();
    FfiResult::Ok(AnyObject::new_raw(try_!(this.with_gaussianDP(mu))))
}

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
/// # Arguments
/// * `this` - The PrivacyGuarantee to extend.
/// * `curve` - An RDP callback mapping order alpha to epsilon(alpha)
/// * `delta` - Source delta in OpenDP's additive approximate-RDP convention;
///   the RDP conversion theorem adds it to the exact-RDP conversion delta
///
/// # Why honest-but-curious?
///
/// The callback should implement a well-defined RDP curve:
///
/// * is functionally pure
/// * returns a finite or infinite non-negative value
/// * returns an upper bound on the true RDP value at order `alpha`
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

#[bootstrap(
    name = "_privacy_guarantee_with_zCDP",
    features("contrib"),
    arguments(this(rust_type = "PrivacyGuarantee"), delta(default = 0.0)),
    returns(rust_type = "PrivacyGuarantee")
)]
/// Attach a zCDP representation to a PrivacyGuarantee.
///
/// # Arguments
/// * `this` - The PrivacyGuarantee to extend.
/// * `rho` - The zCDP parameter
/// * `delta` - Source delta in OpenDP's additive approximate-zCDP convention;
///   the zCDP conversion theorem adds it to the exact-zCDP conversion delta
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_with_zCDP(
    this: *const AnyObject,
    rho: f64,
    delta: f64,
) -> FfiResult<*mut AnyObject> {
    let this = try_!(try_as_ref!(this).downcast_ref::<PrivacyGuarantee>()).clone();
    FfiResult::Ok(AnyObject::new_raw(try_!(this.with_zCDP(rho, delta))))
}

#[bootstrap(
    name = "_privacy_guarantee_delta",
    arguments(curve(rust_type = b"null")),
    returns(hint = "float")
)]
/// Internal function. Use a PrivacyGuarantee to find delta at a given `epsilon`.
///
/// # Arguments
/// * `curve` - The PrivacyGuarantee.
/// * `epsilon` - What to fix epsilon to compute delta.
///
/// # Returns
/// Delta at a given `epsilon`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_delta(
    curve: *const AnyObject,
    epsilon: f64,
) -> FfiResult<*mut AnyObject> {
    try_!(try_as_ref!(curve).downcast_ref::<PrivacyGuarantee>())
        .delta(epsilon)
        .map(AnyObject::new)
        .into()
}

#[bootstrap(
    name = "_privacy_guarantee_beta",
    arguments(curve(rust_type = "PrivacyGuarantee")),
    returns(hint = "float")
)]
/// Internal function. Use a PrivacyGuarantee to find beta at a given alpha.
///
/// # Arguments
/// * `curve` - The PrivacyGuarantee.
/// * `alpha` - What to fix alpha to compute beta.
///
/// # Returns
/// Beta at a given `alpha`.
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
    name = "_privacy_guarantee_epsilon",
    arguments(profile(rust_type = b"null")),
    returns(hint = "float")
)]
/// Internal function. Use a PrivacyGuarantee to find epsilon at a given `delta`.
///
/// # Arguments
/// * `profile` - The PrivacyGuarantee.
/// * `delta` - What to fix delta to compute epsilon.
///
/// # Returns
/// Epsilon at a given `delta`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___privacy_guarantee_epsilon(
    profile: *const AnyObject,
    delta: f64,
) -> FfiResult<*mut AnyObject> {
    try_!(try_as_ref!(profile).downcast_ref::<PrivacyGuarantee>())
        .epsilon(delta)
        .map(AnyObject::new)
        .into()
}

#[bootstrap(
    name = "_privacy_guarantee_alpha",
    arguments(curve(rust_type = b"null"), beta(rust_type = "f64")),
    returns(hint = "float")
)]
/// Internal function. Use a PrivacyGuarantee to find alpha at a given beta.
///
/// # Arguments
/// * `curve` - The PrivacyGuarantee.
/// * `beta` - What to fix beta to compute alpha.
///
/// # Returns
/// Alpha at a given `beta`.
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
