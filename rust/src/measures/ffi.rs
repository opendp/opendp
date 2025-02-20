use std::{ffi::c_char, fmt::Debug, marker::PhantomData};

use crate::ffi::any::{wrap_func, AnyObject, CallbackFn, Downcast};
use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measure},
    error::Fallible,
    ffi::{
        any::AnyMeasure,
        util::{self, into_c_char_p, to_str, ExtrinsicObject, Type},
    },
    measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence},
};

use super::{PrivacyProfile, RenyiDivergence, SmoothedMaxDivergence};

#[bootstrap(
    name = "_measure_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_measures___measure_free(this: *mut AnyMeasure) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
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
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
pub extern "C" fn opendp_measures__measure_distance_type(
    this: *mut AnyMeasure,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(name = "max_divergence")]
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
/// $Y, Y'$ are $d$-close under the max divergence measure whenever
///
/// $D_\infty(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S]}{\Pr[Y' \in S]} \Big] \leq d$.
#[no_mangle]
pub extern "C" fn opendp_measures__max_divergence() -> FfiResult<*mut AnyMeasure> {
    Ok(AnyMeasure::new(MaxDivergence)).into()
}

#[bootstrap(name = "smoothed_max_divergence")]
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
/// $Y, Y'$ are $d$-close under the smoothed max divergence measure whenever,
/// for any choice of non-negative $\epsilon$, and $\delta = d(\epsilon)$,
///
/// $D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon$.
///
/// Note that $\epsilon$ and $\delta$ are not privacy parameters $\epsilon$ and $\delta$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[no_mangle]
pub extern "C" fn opendp_measures__smoothed_max_divergence() -> FfiResult<*mut AnyMeasure> {
    Ok(AnyMeasure::new(SmoothedMaxDivergence)).into()
}

#[bootstrap(name = "fixed_smoothed_max_divergence")]
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
/// $Y, Y'$ are $d$-close under the fixed smoothed max divergence measure whenever
///
/// $D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon$.
///
/// Note that this $\epsilon$ and $\delta$ are not privacy parameters $\epsilon$ and $\delta$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[no_mangle]
pub extern "C" fn opendp_measures__fixed_smoothed_max_divergence() -> FfiResult<*mut AnyMeasure> {
    Ok(AnyMeasure::new(Approximate(MaxDivergence))).into()
}

#[bootstrap(
    generics(M(suppress)),
    arguments(measure(c_type = "AnyMeasure *", rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMeasure *>")
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

#[no_mangle]
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
        [(
            MO,
            [
                MaxDivergence,
                SmoothedMaxDivergence,
                ZeroConcentratedDivergence,
                ExtrinsicDivergence
            ]
        )],
        (measure)
    )
    .into()
}

#[bootstrap(name = "zero_concentrated_divergence")]
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
/// $Y, Y'$ are $d$-close under the zero-concentrated divergence measure if,
/// for every possible choice of $\alpha \in (1, \infty)$,
///
/// $D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d \cdot \alpha$.
#[no_mangle]
pub extern "C" fn opendp_measures__zero_concentrated_divergence() -> FfiResult<*mut AnyMeasure> {
    Ok(AnyMeasure::new(ZeroConcentratedDivergence)).into()
}

#[bootstrap(name = "renyi_divergence")]
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
/// $Y, Y'$ are $d$-close under the Rényi divergence measure if,
/// for any given $\alpha \in (1, \infty)$,
///
/// $D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d(\alpha)$
///
/// Note that this $\epsilon$ and $\alpha$ are not privacy parameters $\epsilon$ and $\alpha$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[no_mangle]
pub extern "C" fn opendp_measures__renyi_divergence() -> FfiResult<*mut AnyMeasure> {
    Ok(AnyMeasure::new(RenyiDivergence)).into()
}

#[derive(Clone, Default)]
pub struct ExtrinsicDivergence {
    pub descriptor: String,
}

impl std::fmt::Debug for ExtrinsicDivergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserDivergence({:?})", self.descriptor)
    }
}

impl PartialEq for ExtrinsicDivergence {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor == other.descriptor
    }
}

impl Measure for ExtrinsicDivergence {
    type Distance = ExtrinsicObject;
}

#[bootstrap(
    name = "user_divergence",
    features("honest-but-curious"),
    arguments(descriptor(rust_type = "String"))
)]
/// Privacy measure with meaning defined by an OpenDP Library user (you).
///
/// Any two instances of UserDivergence are equal if their string descriptors are equal.
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
/// * `descriptor` - A string description of the privacy measure.
///
/// # Why honest-but-curious?
/// The essential requirement of a privacy measure is that it is closed under postprocessing.
/// Your privacy measure `D` must satisfy that, for any pure function `f` and any two distributions `Y, Y'`, then $D(Y, Y') \ge D(f(Y), f(Y'))$.
///
/// Beyond this, you should also consider whether your privacy measure can be used to provide meaningful privacy guarantees to your privacy units.
#[no_mangle]
pub extern "C" fn opendp_measures__user_divergence(
    descriptor: *mut c_char,
) -> FfiResult<*mut AnyMeasure> {
    let descriptor = try_!(to_str(descriptor)).to_string();
    Ok(AnyMeasure::new(ExtrinsicDivergence { descriptor })).into()
}

pub struct TypedMeasure<Q> {
    pub measure: AnyMeasure,
    marker: PhantomData<fn() -> Q>,
}

impl<Q: 'static> TypedMeasure<Q> {
    pub fn new(measure: AnyMeasure) -> Fallible<TypedMeasure<Q>> {
        if measure.distance_type != Type::of::<Q>() {
            return fallible!(
                FFI,
                "unexpected distance type in measure. Expected {}, got {}",
                Type::of::<Q>().to_string(),
                measure.distance_type.to_string()
            );
        }

        Ok(TypedMeasure {
            measure,
            marker: PhantomData,
        })
    }
}

impl<Q> PartialEq for TypedMeasure<Q> {
    fn eq(&self, other: &Self) -> bool {
        self.measure == other.measure
    }
}

impl<Q> Clone for TypedMeasure<Q> {
    fn clone(&self) -> Self {
        Self {
            measure: self.measure.clone(),
            marker: self.marker.clone(),
        }
    }
}

impl<Q> Debug for TypedMeasure<Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.measure)
    }
}
impl<Q> Default for TypedMeasure<Q> {
    fn default() -> Self {
        panic!()
    }
}

impl<Q> Measure for TypedMeasure<Q> {
    type Distance = Q;
}

#[bootstrap(
    name = "new_privacy_profile",
    features("contrib", "honest-but-curious"),
    arguments(curve(rust_type = "f64")),
    returns(rust_type = "PrivacyProfile")
)]
/// Construct a PrivacyProfile from a user-defined callback.
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
/// * returns delta values only within [0, 1]
#[allow(dead_code)]
fn new_privacy_profile(curve: *const CallbackFn) -> Fallible<AnyObject> {
    let _ = curve;
    panic!("this signature only exists for code generation")
}

#[no_mangle]
pub extern "C" fn opendp_measures__new_privacy_profile(
    curve: *const CallbackFn,
) -> FfiResult<*mut AnyObject> {
    let curve = wrap_func(try_as_ref!(curve).clone());
    FfiResult::Ok(AnyObject::new_raw(PrivacyProfile::new(
        move |epsilon: f64| curve(&AnyObject::new(epsilon))?.downcast::<f64>(),
    )))
}
