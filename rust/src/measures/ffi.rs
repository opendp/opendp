use std::{cmp::Ordering, ffi::c_char};

use crate::{
    ffi::{
        any::Downcast,
        util::{ExtrinsicObject, c_bool},
    },
    measures::PrivacyCurveDP,
};
use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measure},
    domains::ffi::ExtrinsicElement,
    error::Fallible,
    ffi::{
        any::AnyMeasure,
        util::{self, into_c_char_p},
    },
    measures::{Approximate, PureDP, zCDP},
    traits::ProductOrd,
};

use super::RenyiDP;

#[bootstrap(
    name = "_measure_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___measure_free(this: *mut AnyMeasure) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "_measure_equal",
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Check whether two measures are equal.
///
/// # Arguments
/// * `left` - Measure to compare.
/// * `right` - Measure to compare.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___measure_equal(
    left: *mut AnyMeasure,
    right: *const AnyMeasure,
) -> FfiResult<*mut c_bool> {
    let status = try_as_ref!(left) == try_as_ref!(right);
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

#[bootstrap(
    name = "measure_debug",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Debug a `measure`.
///
/// # Arguments
/// * `this` - The measure to debug (stringify).
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__measure_debug(this: *mut AnyMeasure) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(format!("{:?}", this))))
}

#[bootstrap(
    name = "measure_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the type of a `measure`.
///
/// # Arguments
/// * `this` - The measure to retrieve the type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__measure_type(this: *mut AnyMeasure) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.type_.descriptor.to_string())))
}

#[bootstrap(
    name = "measure_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the distance type of a `measure`.
///
/// # Arguments
/// * `this` - The measure to retrieve the distance type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__measure_distance_type(
    this: *mut AnyMeasure,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(name = "max_divergence")]
#[deprecated(since = "0.15.0", note = "Use `pure_dp` instead.")]
/// Deprecated alias for `pure_dp`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__max_divergence() -> FfiResult<*mut AnyMeasure> {
    opendp_measures__pure_dp()
}

#[bootstrap(name = "pure_dp")]
/// Privacy measure used to define $\epsilon$-pure differential privacy.
///
/// In the following proof definition, $d$ corresponds to $\epsilon$ when also quantified over all adjacent datasets.
/// That is, $\epsilon$ is the greatest possible $d$
/// over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Proof Definition
///
/// For any two distributions $Y, Y'$ and any non-negative $d$,
/// $Y, Y'$ are $d$-close under the pure-DP privacy measure whenever
///
/// $D_\infty(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S]}{\Pr[Y' \in S]} \Big] \leq d$.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__pure_dp() -> FfiResult<*mut AnyMeasure> {
    Ok(AnyMeasure::new(PureDP)).into()
}

#[bootstrap(name = "smoothed_max_divergence")]
#[deprecated(since = "0.15.0", note = "Use `privacy_curve_dp` instead.")]
/// Deprecated alias for `privacy_curve_dp`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__smoothed_max_divergence() -> FfiResult<*mut AnyMeasure> {
    opendp_measures__privacy_curve_dp()
}

#[bootstrap(name = "profile_dp")]
/// Privacy measure used to define $\epsilon(\delta)$-approximate differential privacy.
///
/// In the following proof definition, $d$ corresponds to a privacy profile when also quantified over all adjacent datasets.
/// That is, a privacy profile $\epsilon(\delta)$ is no smaller than $d(\delta)$ for all possible choices of $\delta$,
/// and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// The distance $d$ is of type PrivacyProfile, so it can be invoked with an $\epsilon$
/// to retrieve the corresponding $\delta$.
///
/// # Proof Definition
///
/// For any two distributions $Y, Y'$ and any curve $d(\cdot)$,
/// $Y, Y'$ are $d$-close under the profile-DP privacy measure whenever,
/// for any choice of non-negative $\epsilon$, and $\delta = d(\epsilon)$,
///
/// $D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon$.
///
/// Note that $\epsilon$ and $\delta$ are not privacy parameters $\epsilon$ and $\delta$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__privacy_curve_dp() -> FfiResult<*mut AnyMeasure> {
    Ok(AnyMeasure::new(PrivacyCurveDP)).into()
}

#[bootstrap(name = "fixed_smoothed_max_divergence")]
#[deprecated(since = "0.15.0", note = "Use `approx_dp` instead.")]
/// Deprecated alias for `approx_dp`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__fixed_smoothed_max_divergence() -> FfiResult<*mut AnyMeasure> {
    opendp_measures__approx_dp()
}

#[bootstrap(name = "approx_dp")]
/// Privacy measure used to define $(\epsilon, \delta)$-approximate differential privacy.
///
/// In the following definition, $d$ corresponds to $(\epsilon, \delta)$ when also quantified over all adjacent datasets.
/// That is, $(\epsilon, \delta)$ is no smaller than $d$ (by product ordering),
/// over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Proof Definition
///
/// For any two distributions $Y, Y'$ and any 2-tuple $d$ of non-negative numbers $\epsilon$ and $\delta$,
/// $Y, Y'$ are $d$-close under the approx-DP privacy measure whenever
///
/// $D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon$.
///
/// Note that this $\epsilon$ and $\delta$ are not privacy parameters $\epsilon$ and $\delta$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__approx_dp() -> FfiResult<*mut AnyMeasure> {
    Ok(AnyMeasure::new(Approximate(PureDP))).into()
}

#[bootstrap(
    rust_path = "measures/struct.Approximate",
    generics(M(suppress)),
    arguments(measure(c_type = "AnyMeasure *", rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMeasure *>", hint = "ApproximateDivergence")
)]
/// Privacy measure used to define $\delta$-approximate PM-differential privacy.
///
/// In the following definition, $d$ corresponds to privacy parameters $(d', \delta)$
/// when also quantified over all adjacent datasets
/// ($d'$ is the privacy parameter corresponding to privacy measure PM).
/// That is, $(d', \delta)$ is no smaller than $d$ (by product ordering),
/// over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Arguments
/// * `measure` - inner privacy measure
///
/// # Proof Definition
///
/// For any two distributions $Y, Y'$ and 2-tuple $d = (d', \delta)$,
/// where $d'$ is the distance with respect to privacy measure PM,
/// $Y, Y'$ are $d$-close under the approximate PM measure whenever,
/// for any choice of $\delta \in [0, 1]$,
/// there exist events $E$ (depending on $Y$) and $E'$ (depending on $Y'$)
/// such that $\Pr[E] \ge 1 - \delta$, $\Pr[E'] \ge 1 - \delta$, and
///
/// $D_{\mathrm{PM}}^\delta(Y|_E, Y'|_{E'}) = D_{\mathrm{PM}}(Y|_E, Y'|_{E'})$
///
/// where $Y|_E$ denotes the distribution of $Y$ conditioned on the event $E$.
///
/// Note that this $\delta$ is not privacy parameter $\delta$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
fn approximate<M: Measure>(measure: M) -> Approximate<M> {
    Approximate(measure)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__approximate(
    measure: *const AnyMeasure,
) -> FfiResult<*mut AnyMeasure> {
    fn monomorphize<MO: 'static + Measure>(measure: &AnyMeasure) -> Fallible<AnyMeasure> {
        let measure = measure.downcast_ref::<MO>()?.clone();
        Ok(AnyMeasure::new(approximate(measure)))
    }

    let measure = try_as_ref!(measure);
    let MO = measure.type_.clone();

    dispatch!(
        monomorphize,
        [(MO, [PureDP, PrivacyCurveDP, zCDP, ExtrinsicDivergence])],
        (measure)
    )
    .into()
}

#[bootstrap(name = "_approximate_get_inner_measure")]
/// Retrieve the inner privacy measure of an approximate privacy measure.
///
/// # Arguments
/// * `privacy_measure` - The privacy measure to inspect
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___approximate_get_inner_measure(
    privacy_measure: *const AnyMeasure,
) -> FfiResult<*mut AnyMeasure> {
    let privacy_measure = try_as_ref!(privacy_measure);
    let M = privacy_measure.type_.clone();
    let T = try_!(M.get_atom());

    fn monomorphize<M: 'static + Measure>(privacy_measure: &AnyMeasure) -> Fallible<AnyMeasure> {
        let privacy_measure = privacy_measure.downcast_ref::<Approximate<M>>()?.clone();
        Ok(AnyMeasure::new(privacy_measure.0.clone()))
    }

    dispatch!(
        monomorphize,
        [(T, [PureDP, PrivacyCurveDP, zCDP, ExtrinsicDivergence])],
        (privacy_measure)
    )
    .into()
}

#[bootstrap(name = "zero_concentrated_divergence")]
#[deprecated(since = "0.15.0", note = "Use `zcdp` instead.")]
/// Deprecated alias for `zero_concentrated_divergence`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__zero_concentrated_divergence() -> FfiResult<*mut AnyMeasure> {
    opendp_measures__zcdp()
}

#[bootstrap(name = "zcdp")]
/// Privacy measure used to define $\rho$-zero concentrated differential privacy.
///
/// In the following proof definition, $d$ corresponds to $\rho$ when also quantified over all adjacent datasets.
/// That is, $\rho$ is the greatest possible $d$
/// over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Proof Definition
///
/// For any two distributions $Y, Y'$ and any non-negative $d$,
/// $Y, Y'$ are $d$-close under the zCDP privacy measure if,
/// for every possible choice of $\alpha \in (1, \infty)$,
///
/// $D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d \cdot \alpha$.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__zcdp() -> FfiResult<*mut AnyMeasure> {
    Ok(AnyMeasure::new(zCDP)).into()
}

#[bootstrap(name = "renyi_divergence")]
#[deprecated(since = "0.15.0", note = "Use `renyi_dp` instead.")]
/// Deprecated alias for `renyi_dp`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__renyi_divergence() -> FfiResult<*mut AnyMeasure> {
    opendp_measures__renyi_dp()
}

#[bootstrap(name = "renyi_dp")]
/// Privacy measure used to define $\epsilon(\alpha)$-Rényi differential privacy.
///
/// In the following proof definition, $d$ corresponds to an RDP curve when also quantified over all adjacent datasets.
/// That is, an RDP curve $\epsilon(\alpha)$ is no smaller than $d(\alpha)$ for any possible choices of $\alpha$,
/// and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Proof Definition
///
/// For any two distributions $Y, Y'$ and any curve $d$,
/// $Y, Y'$ are $d$-close under the Rényi-DP privacy measure if,
/// for any given $\alpha \in (1, \infty)$,
///
/// $D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d(\alpha)$
///
/// Note that this $\epsilon$ and $\alpha$ are not privacy parameters $\epsilon$ and $\alpha$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__renyi_dp() -> FfiResult<*mut AnyMeasure> {
    Ok(AnyMeasure::new(RenyiDP)).into()
}

#[derive(Clone)]
pub struct ExtrinsicDivergence {
    pub element: ExtrinsicElement,
}

// TODO: drop requirement for Default on privacy measures
impl Default for ExtrinsicDivergence {
    fn default() -> Self {
        Self {
            element: ExtrinsicElement::new(
                "UserDivergence".to_string(),
                ExtrinsicObject(std::ptr::null()),
            ),
        }
    }
}

impl std::fmt::Debug for ExtrinsicDivergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.element)
    }
}

impl PartialEq for ExtrinsicDivergence {
    fn eq(&self, other: &Self) -> bool {
        self.element
            .value
            .total_cmp(&other.element.value)
            .map(|ordering| ordering == Ordering::Equal)
            .unwrap_or(false)
    }
}

impl Measure for ExtrinsicDivergence {
    type Distance = ExtrinsicObject;
}

#[bootstrap(
    name = "user_divergence",
    features("honest-but-curious"),
    arguments(
        identifier(c_type = "char *", rust_type = b"null"),
        descriptor(default = b"null", rust_type = "ExtrinsicObject")
    )
)]
/// Privacy measure with meaning defined by an OpenDP Library user (you).
///
/// Any two instances of UserDivergence are equal if their descriptors compare equal.
///
/// # Proof definition
///
/// For any two distributions $Y, Y'$ and any $d$,
/// $Y, Y'$ are $d$-close under the user divergence measure ($D_U$) if,
///
/// $D_U(Y, Y') \le d$.
///
/// For $D_U$ to qualify as a privacy measure, then for any postprocessing function $f$,
/// $D_U(Y, Y') \ge D_U(f(Y), f(Y'))$.
///
/// # Arguments
/// * `identifier` - A string description of the privacy measure.
/// * `descriptor` - Additional constraints on the privacy measure.
///
/// # Why honest-but-curious?
/// The essential requirement of a privacy measure is that it is closed under postprocessing.
/// Your privacy measure `D` must satisfy that, for any pure function `f` and any two distributions `Y, Y'`, then $D(Y, Y') \ge D(f(Y), f(Y'))$.
///
/// Beyond this, you should also consider whether your privacy measure can be used to provide meaningful privacy guarantees to your privacy units.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures__user_divergence(
    identifier: *mut c_char,
    descriptor: *mut ExtrinsicObject,
) -> FfiResult<*mut AnyMeasure> {
    let value = try_as_ref!(descriptor).clone();
    let identifier = try_!(crate::ffi::util::to_str(identifier)).to_string();
    let element = ExtrinsicElement::new(identifier, value);
    Ok(AnyMeasure::new(ExtrinsicDivergence { element })).into()
}

#[bootstrap(
    name = "_extrinsic_measure_descriptor",
    arguments(measure(rust_type = b"null")),
    returns(c_type = "FfiResult<ExtrinsicObject *>")
)]
/// Retrieve the descriptor value stored in an extrinsic measure.
///
/// # Arguments
/// * `measure` - The ExtrinsicDivergence to extract the descriptor from
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measures___extrinsic_measure_descriptor(
    measure: *mut AnyMeasure,
) -> FfiResult<*mut ExtrinsicObject> {
    let measure = try_!(try_as_ref!(measure).downcast_ref::<ExtrinsicDivergence>()).clone();
    FfiResult::Ok(util::into_raw(measure.element.value.clone()))
}
