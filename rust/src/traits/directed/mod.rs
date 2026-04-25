//! Interval arithmetic API for OpenDP conservative numerics.
//!
//! Design goals:
//! - Keep directed endpoints as the primitive implementation detail.
//! - Expose one public two-sided abstraction: `Interval<B>`.
//! - Preserve the A/B/C/D guarantee ladder:
//!     A: approximate native f64, no directed guarantee
//!     B: best-effort native f64, including Gaussian special functions
//!     C: certified native f64 for simple arithmetic only
//!     D: soft dashu FBig for supported certified operations
//! - Avoid sign parameters in scalar types. Sign/domain reasoning happens at
//!   interval runtime boundaries.
//! - Optimize common monotone multiplication cases before falling back to the
//!   general four-corner interval product.
//!
//! This module intentionally implements endpoint intervals `[lo, hi]`, not
//! midpoint-radius balls. The public type does not mention Up/Down; directed
//! rounding lives in the backend endpoints.

use crate::{
    error::Fallible,
    traits::{ToFloatRounded, samplers::ODPRound},
};
use dashu::base::{Abs, SquareRoot};
use dashu::{
    float::{
        FBig,
        round::mode::{Down, Up},
    },
    rational::RBig,
};
use errorfunctions::RealErrorFunctions;
use statrs::function::erf::{erfc, erfc_inv};
use std::{fmt, marker::PhantomData};

const DASHU_MIN_PRECISION: usize = 10;

// -----------------------------------------------------------------------------
// Native regimes and endpoint backends
// -----------------------------------------------------------------------------

mod private {
    pub trait Sealed {}
}

/// Approximate native arithmetic, with no directed guarantee.
#[derive(Clone, Copy, Debug)]
pub enum Approximate {}

/// Best-effort directed native arithmetic.
///
/// Simple arithmetic is nudged outward; special functions are also nudged, but
/// no formal error guarantee is claimed for the underlying implementation.
#[derive(Clone, Copy, Debug)]
pub enum BestEffort {}

/// Certified native arithmetic for simple f64 operations only.
#[derive(Clone, Copy, Debug)]
pub enum Certified {}

impl private::Sealed for Approximate {}
impl private::Sealed for BestEffort {}
impl private::Sealed for Certified {}

pub trait NativeRegime: private::Sealed + Clone + Copy + 'static {
    fn round_simple<D: ODPRound>(value: f64) -> Fallible<N64<D, Self>>
    where
        Self: Sized;
}

impl NativeRegime for Approximate {
    #[inline]
    fn round_simple<D: ODPRound>(value: f64) -> Fallible<N64<D, Self>> {
        N64::<D, Self>::raw(value)
    }
}

impl NativeRegime for BestEffort {
    #[inline]
    fn round_simple<D: ODPRound>(value: f64) -> Fallible<N64<D, Self>> {
        N64::<D, Self>::next(value)
    }
}

impl NativeRegime for Certified {
    #[inline]
    fn round_simple<D: ODPRound>(value: f64) -> Fallible<N64<D, Self>> {
        N64::<D, Self>::next(value)
    }
}

/// Native f64 endpoint. `D` is internal endpoint direction.
#[derive(Clone, Copy)]
pub struct N64<D: ODPRound, R = Certified> {
    value: f64,
    _marker: PhantomData<(D, R)>,
}

/// Approximate native endpoint.
pub type A64<D> = N64<D, Approximate>;
/// Best-effort native endpoint.
pub type B64<D> = N64<D, BestEffort>;
/// Certified native endpoint for simple operations.
pub type C64<D> = N64<D, Certified>;

impl<D: ODPRound, R> fmt::Debug for N64<D, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("N64").field(&self.value).finish()
    }
}

/// Soft dashu endpoint.
#[derive(Clone)]
pub struct DBig<D: ODPRound> {
    value: FBig<D>,
}

impl<D: ODPRound> fmt::Debug for DBig<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DBig")
            .field(&self.value.to_f64_rounded())
            .finish()
    }
}

// -----------------------------------------------------------------------------
// Endpoint traits
// -----------------------------------------------------------------------------

/// One directed endpoint. Formula code should generally use `Interval<B>`
/// rather than naming endpoint types directly.
pub trait Endpoint: Sized + Clone {
    type Opposite: Endpoint<Opposite = Self>;

    /// Exact point construction from an already-representable f64.
    fn exact(value: f64) -> Fallible<Self>;

    /// Construction around an approximate native constant, rounded outward.
    fn approx(value: f64) -> Fallible<Self>;

    /// Convert this endpoint to f64 using its own endpoint semantics.
    fn to_f64(&self) -> Fallible<f64>;

    /// Compare two endpoints of the same concrete type without imposing
    /// ordering requirements on rounding-mode marker types.
    fn lt(&self, rhs: &Self) -> Fallible<bool>;

    #[inline]
    fn le(&self, rhs: &Self) -> Fallible<bool> {
        Ok(!rhs.lt(self)?)
    }

    #[inline]
    fn min(self, rhs: Self) -> Fallible<Self> {
        if rhs.lt(&self)? { Ok(rhs) } else { Ok(self) }
    }

    #[inline]
    fn max(self, rhs: Self) -> Fallible<Self> {
        if rhs.lt(&self)? { Ok(self) } else { Ok(rhs) }
    }

    fn lt_f64(&self, rhs: f64) -> Fallible<bool>;

    #[inline]
    fn le_f64(&self, rhs: f64) -> Fallible<bool> {
        Ok(self.lt_f64(rhs)? || !Self::exact(rhs)?.lt(self)?)
    }
}

/// Low-level operation rounded into direction `DOut`.
pub trait AddTo<Rhs>: Sized {
    type Output<DOut: ODPRound>;
    fn add_to_<DOut: ODPRound>(self, rhs: Rhs) -> Fallible<Self::Output<DOut>>;
}

pub trait SubTo<Rhs>: Sized {
    type Output<DOut: ODPRound>;
    fn sub_to_<DOut: ODPRound>(self, rhs: Rhs) -> Fallible<Self::Output<DOut>>;
}

pub trait MulTo<Rhs>: Sized {
    type Output<DOut: ODPRound>;
    fn mul_to_<DOut: ODPRound>(self, rhs: Rhs) -> Fallible<Self::Output<DOut>>;
}

pub trait DivTo<Rhs>: Sized {
    type Output<DOut: ODPRound>;
    fn div_to_<DOut: ODPRound>(self, rhs: Rhs) -> Fallible<Self::Output<DOut>>;
}

pub trait NegTo: Sized {
    type Output<DOut: ODPRound>;
    fn neg_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>>;
}

pub trait AbsTo: Sized {
    type Output<DOut: ODPRound>;
    fn abs_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>>;
}

pub trait ExpTo: Sized {
    type Output<DOut: ODPRound>;
    fn exp_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>>;
}

pub trait ExpM1To: Sized {
    type Output<DOut: ODPRound>;
    fn exp_m1_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>>;
}

pub trait LnTo: Sized {
    type Output<DOut: ODPRound>;
    fn ln_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>>;
}

pub trait SqrtTo: Sized {
    type Output<DOut: ODPRound>;
    fn sqrt_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>>;
}

/// Best-effort Gaussian/special functions.
pub trait ErfTo: Sized {
    type Output<DOut: ODPRound>;
    fn erfc_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>>;
    fn erfcx_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>>;
    fn erfc_inv_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>>;
}

// -----------------------------------------------------------------------------
// N64 endpoint impls
// -----------------------------------------------------------------------------

impl<D: ODPRound, R> N64<D, R> {
    #[inline]
    fn raw(value: f64) -> Fallible<Self> {
        Ok(Self {
            value: finite(value)?,
            _marker: PhantomData,
        })
    }

    #[inline]
    fn next(value: f64) -> Fallible<Self> {
        let value = finite(value)?;
        let value = if D::IS_UP {
            value.next_up()
        } else {
            value.next_down()
        };
        Self::raw(value)
    }

    #[inline]
    fn round(value: f64) -> Fallible<Self>
    where
        R: NativeRegime,
    {
        R::round_simple::<D>(value)
    }

    /// Coarse best-effort widening for special functions with unknown error.
    #[inline]
    fn round_erf32(value: f64) -> Fallible<Self> {
        let value = finite(value)?;
        let widened = if D::IS_UP {
            (value as f32).next_up()
        } else {
            (value as f32).next_down()
        };
        Self::next(widened as f64)
    }
}

impl<D, R> Endpoint for N64<D, R>
where
    D: ODPRound,
    R: NativeRegime,
{
    type Opposite = N64<D::C, R>;

    #[inline]
    fn exact(value: f64) -> Fallible<Self> {
        Self::raw(value)
    }

    #[inline]
    fn approx(value: f64) -> Fallible<Self> {
        Self::round(value)
    }

    #[inline]
    fn to_f64(&self) -> Fallible<f64> {
        finite(self.value)
    }

    #[inline]
    fn lt(&self, rhs: &Self) -> Fallible<bool> {
        Ok(self.value < rhs.value)
    }

    #[inline]
    fn lt_f64(&self, rhs: f64) -> Fallible<bool> {
        Ok(self.value < finite(rhs)?)
    }
}

impl<D1, D2, R> AddTo<N64<D2, R>> for N64<D1, R>
where
    D1: ODPRound,
    D2: ODPRound,
    R: NativeRegime,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn add_to_<DOut: ODPRound>(self, rhs: N64<D2, R>) -> Fallible<Self::Output<DOut>> {
        N64::<DOut, R>::round(self.value + rhs.value)
    }
}

impl<D1, D2, R> SubTo<N64<D2, R>> for N64<D1, R>
where
    D1: ODPRound,
    D2: ODPRound,
    R: NativeRegime,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn sub_to_<DOut: ODPRound>(self, rhs: N64<D2, R>) -> Fallible<Self::Output<DOut>> {
        N64::<DOut, R>::round(self.value - rhs.value)
    }
}

impl<D1, D2, R> MulTo<N64<D2, R>> for N64<D1, R>
where
    D1: ODPRound,
    D2: ODPRound,
    R: NativeRegime,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn mul_to_<DOut: ODPRound>(self, rhs: N64<D2, R>) -> Fallible<Self::Output<DOut>> {
        N64::<DOut, R>::round(self.value * rhs.value)
    }
}

impl<D1, D2, R> DivTo<N64<D2, R>> for N64<D1, R>
where
    D1: ODPRound,
    D2: ODPRound,
    R: NativeRegime,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn div_to_<DOut: ODPRound>(self, rhs: N64<D2, R>) -> Fallible<Self::Output<DOut>> {
        N64::<DOut, R>::round(self.value / rhs.value)
    }
}

impl<D, R> NegTo for N64<D, R>
where
    D: ODPRound,
    R: NativeRegime,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn neg_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        N64::<DOut, R>::raw(-self.value)
    }
}

impl<D, R> AbsTo for N64<D, R>
where
    D: ODPRound,
    R: NativeRegime,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn abs_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        N64::<DOut, R>::raw(self.value.abs())
    }
}

impl<D, R> ExpTo for N64<D, R>
where
    D: ODPRound,
    R: NativeRegime + NativeSpecial,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn exp_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        R::round_special::<DOut>(self.value.exp())
    }
}

impl<D, R> ExpM1To for N64<D, R>
where
    D: ODPRound,
    R: NativeRegime + NativeSpecial,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn exp_m1_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        R::round_special::<DOut>(self.value.exp_m1())
    }
}

impl<D, R> LnTo for N64<D, R>
where
    D: ODPRound,
    R: NativeRegime + NativeSpecial,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn ln_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        R::round_special::<DOut>(self.value.ln())
    }
}

impl<D, R> SqrtTo for N64<D, R>
where
    D: ODPRound,
    R: NativeRegime + NativeSpecial,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn sqrt_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        R::round_special::<DOut>(self.value.sqrt())
    }
}

impl<D, R> ErfTo for N64<D, R>
where
    D: ODPRound,
    R: NativeRegime + NativeSpecial,
{
    type Output<DOut: ODPRound> = N64<DOut, R>;

    #[inline]
    fn erfc_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        if self.value == f64::INFINITY {
            return N64::<DOut, R>::raw(0.0);
        }
        if self.value == f64::NEG_INFINITY {
            return N64::<DOut, R>::raw(2.0);
        }
        R::round_erf::<DOut>(erfc(self.value).clamp(0.0, 2.0))
    }

    #[inline]
    fn erfcx_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        if self.value.is_infinite() && self.value.is_sign_positive() {
            return N64::<DOut, R>::raw(0.0);
        }
        let y = self.value.erfcx();
        if !y.is_finite() || y < 0.0 {
            return fallible!(
                FailedMap,
                "erfcx({}) returned invalid value {y}",
                self.value
            );
        }
        R::round_erf::<DOut>(y.max(0.0))
    }

    #[inline]
    fn erfc_inv_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        let p = self.value;
        if !p.is_finite() || !(0.0..=2.0).contains(&p) {
            return fallible!(FailedMap, "p ({p}) must be finite and in [0, 2]");
        }
        if p == 0.0 || p == 2.0 {
            return fallible!(FailedMap, "erfc_inv endpoint {p} is infinite");
        }
        R::round_erf::<DOut>(erfc_inv(p))
    }
}

pub trait NativeSpecial: NativeRegime {
    fn round_special<D: ODPRound>(value: f64) -> Fallible<N64<D, Self>>
    where
        Self: Sized;

    fn round_erf<D: ODPRound>(value: f64) -> Fallible<N64<D, Self>>
    where
        Self: Sized;
}

impl NativeSpecial for Approximate {
    #[inline]
    fn round_special<D: ODPRound>(value: f64) -> Fallible<N64<D, Self>> {
        N64::<D, Self>::raw(value)
    }

    #[inline]
    fn round_erf<D: ODPRound>(value: f64) -> Fallible<N64<D, Self>> {
        N64::<D, Self>::raw(value)
    }
}

impl NativeSpecial for BestEffort {
    #[inline]
    fn round_special<D: ODPRound>(value: f64) -> Fallible<N64<D, Self>> {
        N64::<D, Self>::next(value)
    }

    #[inline]
    fn round_erf<D: ODPRound>(value: f64) -> Fallible<N64<D, Self>> {
        N64::<D, Self>::round_erf32(value)
    }
}

// No NativeSpecial impl for Certified.

// -----------------------------------------------------------------------------
// DBig endpoint impls
// -----------------------------------------------------------------------------

impl<D: ODPRound> DBig<D> {
    #[inline]
    fn raw(value: FBig<D>) -> Self {
        Self {
            value: normalize_fbig(value),
        }
    }

    #[inline]
    fn exact_from_f64(value: f64) -> Fallible<Self> {
        Ok(Self::raw(FBig::<D>::try_from(finite(value)?)?))
    }

    #[inline]
    fn approx_from_f64(value: f64) -> Fallible<Self> {
        let value = finite(value)?;
        let value = if D::IS_UP {
            value.next_up()
        } else {
            value.next_down()
        };
        Ok(Self::raw(FBig::<D>::try_from(value)?))
    }

    #[inline]
    pub fn to_rbig(&self) -> Fallible<RBig> {
        Ok(RBig::try_from(self.value.clone())?)
    }
}

impl<D: ODPRound> Endpoint for DBig<D> {
    type Opposite = DBig<D::C>;

    #[inline]
    fn exact(value: f64) -> Fallible<Self> {
        Self::exact_from_f64(value)
    }

    #[inline]
    fn approx(value: f64) -> Fallible<Self> {
        Self::approx_from_f64(value)
    }

    #[inline]
    fn to_f64(&self) -> Fallible<f64> {
        finite(self.value.to_f64_rounded())
    }

    #[inline]
    fn lt(&self, rhs: &Self) -> Fallible<bool> {
        Ok(self.to_rbig()? < rhs.to_rbig()?)
    }

    #[inline]
    fn lt_f64(&self, rhs: f64) -> Fallible<bool> {
        Ok(self.to_rbig()? < RBig::try_from(finite(rhs)?)?)
    }
}

impl<D1, D2> AddTo<DBig<D2>> for DBig<D1>
where
    D1: ODPRound,
    D2: ODPRound,
{
    type Output<DOut: ODPRound> = DBig<DOut>;

    #[inline]
    fn add_to_<DOut: ODPRound>(self, rhs: DBig<D2>) -> Fallible<Self::Output<DOut>> {
        Ok(DBig::raw(
            self.value.with_rounding::<DOut>() + rhs.value.with_rounding::<DOut>(),
        ))
    }
}

impl<D1, D2> SubTo<DBig<D2>> for DBig<D1>
where
    D1: ODPRound,
    D2: ODPRound,
{
    type Output<DOut: ODPRound> = DBig<DOut>;

    #[inline]
    fn sub_to_<DOut: ODPRound>(self, rhs: DBig<D2>) -> Fallible<Self::Output<DOut>> {
        Ok(DBig::raw(
            self.value.with_rounding::<DOut>() - rhs.value.with_rounding::<DOut>(),
        ))
    }
}

impl<D1, D2> MulTo<DBig<D2>> for DBig<D1>
where
    D1: ODPRound,
    D2: ODPRound,
{
    type Output<DOut: ODPRound> = DBig<DOut>;

    #[inline]
    fn mul_to_<DOut: ODPRound>(self, rhs: DBig<D2>) -> Fallible<Self::Output<DOut>> {
        Ok(DBig::raw(
            self.value.with_rounding::<DOut>() * rhs.value.with_rounding::<DOut>(),
        ))
    }
}

impl<D1, D2> DivTo<DBig<D2>> for DBig<D1>
where
    D1: ODPRound,
    D2: ODPRound,
{
    type Output<DOut: ODPRound> = DBig<DOut>;

    #[inline]
    fn div_to_<DOut: ODPRound>(self, rhs: DBig<D2>) -> Fallible<Self::Output<DOut>> {
        Ok(DBig::raw(
            self.value.with_rounding::<DOut>() / rhs.value.with_rounding::<DOut>(),
        ))
    }
}

impl<D> NegTo for DBig<D>
where
    D: ODPRound,
{
    type Output<DOut: ODPRound> = DBig<DOut>;

    #[inline]
    fn neg_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        Ok(DBig::raw((-self.value).with_rounding::<DOut>()))
    }
}

impl<D> AbsTo for DBig<D>
where
    D: ODPRound,
{
    type Output<DOut: ODPRound> = DBig<DOut>;

    #[inline]
    fn abs_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        Ok(DBig::raw(self.value.abs().with_rounding::<DOut>()))
    }
}

impl<D> ExpTo for DBig<D>
where
    D: ODPRound,
{
    type Output<DOut: ODPRound> = DBig<DOut>;

    #[inline]
    fn exp_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        Ok(DBig::raw(self.value.with_rounding::<DOut>().exp()))
    }
}

impl<D> ExpM1To for DBig<D>
where
    D: ODPRound,
{
    type Output<DOut: ODPRound> = DBig<DOut>;

    #[inline]
    fn exp_m1_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        Ok(DBig::raw(self.value.with_rounding::<DOut>().exp_m1()))
    }
}

impl<D> LnTo for DBig<D>
where
    D: ODPRound,
{
    type Output<DOut: ODPRound> = DBig<DOut>;

    #[inline]
    fn ln_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        Ok(DBig::raw(self.value.with_rounding::<DOut>().ln()))
    }
}

impl<D> SqrtTo for DBig<D>
where
    D: ODPRound,
{
    type Output<DOut: ODPRound> = DBig<DOut>;

    #[inline]
    fn sqrt_to_<DOut: ODPRound>(self) -> Fallible<Self::Output<DOut>> {
        Ok(DBig::raw(self.value.with_rounding::<DOut>().sqrt()))
    }
}

// No ErfTo impl for DBig: Dashu does not provide erfc/erfcx/erfc_inv here.

// -----------------------------------------------------------------------------
// Interval backend families: A/B/C/D
// -----------------------------------------------------------------------------

pub trait IntervalBackend: Clone + Copy + 'static {
    type Lo: Endpoint<Opposite = Self::Hi>;
    type Hi: Endpoint<Opposite = Self::Lo>;

    fn ordered(lo: &Self::Lo, hi: &Self::Hi) -> Fallible<bool>;
}

/// Approximate native f64 interval backend.
#[derive(Clone, Copy, Debug)]
pub enum A {}
/// Best-effort native f64 interval backend.
#[derive(Clone, Copy, Debug)]
pub enum B {}
/// Certified native f64 interval backend for simple ops.
#[derive(Clone, Copy, Debug)]
pub enum C {}
/// Soft dashu FBig interval backend.
#[derive(Clone, Copy, Debug)]
pub enum D {}

impl IntervalBackend for A {
    type Lo = A64<Down>;
    type Hi = A64<Up>;

    #[inline]
    fn ordered(lo: &Self::Lo, hi: &Self::Hi) -> Fallible<bool> {
        Ok(lo.value <= hi.value)
    }
}
impl IntervalBackend for B {
    type Lo = B64<Down>;
    type Hi = B64<Up>;

    #[inline]
    fn ordered(lo: &Self::Lo, hi: &Self::Hi) -> Fallible<bool> {
        Ok(lo.value <= hi.value)
    }
}
impl IntervalBackend for C {
    type Lo = C64<Down>;
    type Hi = C64<Up>;

    #[inline]
    fn ordered(lo: &Self::Lo, hi: &Self::Hi) -> Fallible<bool> {
        Ok(lo.value <= hi.value)
    }
}
impl IntervalBackend for D {
    type Lo = DBig<Down>;
    type Hi = DBig<Up>;

    #[inline]
    fn ordered(lo: &Self::Lo, hi: &Self::Hi) -> Fallible<bool> {
        Ok(lo.to_rbig()? <= hi.to_rbig()?)
    }
}

/// Backend capability for the simple interval arithmetic operations.
///
/// This trait is implemented once for any backend whose endpoints support the
/// primitive directed operations. Generic formula code should normally be
/// generic over the backend marker (`A`, `B`, `C`, `D`) and call inherent
/// methods on `Interval<Bk>`.
pub trait IntervalArithmeticBackend: IntervalBackend + Sized {
    fn add_interval(lhs: Interval<Self>, rhs: Interval<Self>) -> Fallible<Interval<Self>>;
    fn sub_interval(lhs: Interval<Self>, rhs: Interval<Self>) -> Fallible<Interval<Self>>;
    fn neg_interval(x: Interval<Self>) -> Fallible<Interval<Self>>;
    fn mul_interval(lhs: Interval<Self>, rhs: Interval<Self>) -> Fallible<Interval<Self>>;
    fn recip_interval(x: Interval<Self>) -> Fallible<Interval<Self>>;
    fn div_interval(lhs: Interval<Self>, rhs: Interval<Self>) -> Fallible<Interval<Self>>;
    fn abs_interval(x: Interval<Self>) -> Fallible<Interval<Self>>;
}

impl<T> IntervalArithmeticBackend for T
where
    T: IntervalBackend,
    T::Lo: Clone
        + AddTo<T::Lo, Output<Down> = T::Lo>
        + SubTo<T::Hi, Output<Down> = T::Lo>
        + MulTo<T::Lo, Output<Down> = T::Lo>
        + MulTo<T::Hi, Output<Down> = T::Lo>
        + MulTo<T::Lo, Output<Up> = T::Hi>
        + MulTo<T::Hi, Output<Up> = T::Hi>
        + DivTo<T::Hi, Output<Down> = T::Lo>
        + AbsTo<Output<Up> = T::Hi>
        + NegTo<Output<Up> = T::Hi>,
    T::Hi: Clone
        + AddTo<T::Hi, Output<Up> = T::Hi>
        + SubTo<T::Lo, Output<Up> = T::Hi>
        + MulTo<T::Lo, Output<Down> = T::Lo>
        + MulTo<T::Hi, Output<Down> = T::Lo>
        + MulTo<T::Lo, Output<Up> = T::Hi>
        + MulTo<T::Hi, Output<Up> = T::Hi>
        + DivTo<T::Lo, Output<Up> = T::Hi>
        + AbsTo<Output<Up> = T::Hi>
        + NegTo<Output<Down> = T::Lo>,
{
    #[inline]
    fn add_interval(lhs: Interval<Self>, rhs: Interval<Self>) -> Fallible<Interval<Self>> {
        Interval::new(
            lhs.lo.add_to_::<Down>(rhs.lo)?,
            lhs.hi.add_to_::<Up>(rhs.hi)?,
        )
    }

    #[inline]
    fn sub_interval(lhs: Interval<Self>, rhs: Interval<Self>) -> Fallible<Interval<Self>> {
        Interval::new(
            lhs.lo.sub_to_::<Down>(rhs.hi)?,
            lhs.hi.sub_to_::<Up>(rhs.lo)?,
        )
    }

    #[inline]
    fn neg_interval(x: Interval<Self>) -> Fallible<Interval<Self>> {
        Interval::new(x.hi.neg_to_::<Down>()?, x.lo.neg_to_::<Up>()?)
    }

    #[inline]
    fn mul_interval(lhs: Interval<Self>, rhs: Interval<Self>) -> Fallible<Interval<Self>> {
        let lhs_nonneg = lhs.is_nonnegative()?;
        let lhs_nonpos = lhs.is_nonpositive()?;
        let rhs_nonneg = rhs.is_nonnegative()?;
        let rhs_nonpos = rhs.is_nonpositive()?;

        if lhs_nonneg && rhs_nonneg {
            return Interval::new(
                lhs.lo.mul_to_::<Down>(rhs.lo)?,
                lhs.hi.mul_to_::<Up>(rhs.hi)?,
            );
        }

        if lhs_nonpos && rhs_nonpos {
            return Interval::new(
                lhs.hi.mul_to_::<Down>(rhs.hi)?,
                lhs.lo.mul_to_::<Up>(rhs.lo)?,
            );
        }

        if lhs_nonneg && rhs_nonpos {
            return Interval::new(
                lhs.hi.mul_to_::<Down>(rhs.lo)?,
                lhs.lo.mul_to_::<Up>(rhs.hi)?,
            );
        }

        if lhs_nonpos && rhs_nonneg {
            return Interval::new(
                lhs.lo.mul_to_::<Down>(rhs.hi)?,
                lhs.hi.mul_to_::<Up>(rhs.lo)?,
            );
        }

        let lo = min4_endpoint(
            lhs.lo.clone().mul_to_::<Down>(rhs.lo.clone())?,
            lhs.lo.clone().mul_to_::<Down>(rhs.hi.clone())?,
            lhs.hi.clone().mul_to_::<Down>(rhs.lo.clone())?,
            lhs.hi.clone().mul_to_::<Down>(rhs.hi.clone())?,
        )?;

        let hi = max4_endpoint(
            lhs.lo.clone().mul_to_::<Up>(rhs.lo.clone())?,
            lhs.lo.clone().mul_to_::<Up>(rhs.hi.clone())?,
            lhs.hi.clone().mul_to_::<Up>(rhs.lo.clone())?,
            lhs.hi.clone().mul_to_::<Up>(rhs.hi.clone())?,
        )?;

        Interval::new(lo, hi)
    }

    #[inline]
    fn recip_interval(x: Interval<Self>) -> Fallible<Interval<Self>> {
        if x.contains_zero()? {
            return fallible!(
                FailedMap,
                "cannot take reciprocal of interval containing zero"
            );
        }
        Interval::new(
            T::Lo::exact(1.0)?.div_to_::<Down>(x.hi)?,
            T::Hi::exact(1.0)?.div_to_::<Up>(x.lo)?,
        )
    }

    #[inline]
    fn div_interval(lhs: Interval<Self>, rhs: Interval<Self>) -> Fallible<Interval<Self>> {
        Self::mul_interval(lhs, Self::recip_interval(rhs)?)
    }

    #[inline]
    fn abs_interval(x: Interval<Self>) -> Fallible<Interval<Self>> {
        if x.is_nonnegative()? {
            return Ok(x);
        }
        if x.is_nonpositive()? {
            return Interval::new(x.hi.neg_to_::<Down>()?, x.lo.neg_to_::<Up>()?);
        }
        let hi = x.lo.abs_to_::<Up>()?.max(x.hi.abs_to_::<Up>()?)?;
        Interval::new(T::Lo::exact(0.0)?, hi)
    }
}

/// Backend capability for increasing exponential maps.
pub trait IntervalExpBackend: IntervalBackend + Sized {
    fn exp_interval(x: Interval<Self>) -> Fallible<Interval<Self>>;
    fn exp_m1_interval(x: Interval<Self>) -> Fallible<Interval<Self>>;
    fn ln_interval(x: Interval<Self>) -> Fallible<Interval<Self>>;
    fn sqrt_interval(x: Interval<Self>) -> Fallible<Interval<Self>>;
}

impl<T> IntervalExpBackend for T
where
    T: IntervalBackend,
    T::Lo: ExpTo<Output<Down> = T::Lo>
        + ExpM1To<Output<Down> = T::Lo>
        + LnTo<Output<Down> = T::Lo>
        + SqrtTo<Output<Down> = T::Lo>,
    T::Hi: ExpTo<Output<Up> = T::Hi>
        + ExpM1To<Output<Up> = T::Hi>
        + LnTo<Output<Up> = T::Hi>
        + SqrtTo<Output<Up> = T::Hi>,
{
    #[inline]
    fn exp_interval(x: Interval<Self>) -> Fallible<Interval<Self>> {
        Interval::new(x.lo.exp_to_::<Down>()?, x.hi.exp_to_::<Up>()?)
    }

    #[inline]
    fn exp_m1_interval(x: Interval<Self>) -> Fallible<Interval<Self>> {
        // expm1 is increasing.
        Interval::new(x.lo.exp_m1_to_::<Down>()?, x.hi.exp_m1_to_::<Up>()?)
    }

    #[inline]
    fn ln_interval(x: Interval<Self>) -> Fallible<Interval<Self>> {
        if x.lo.le_f64(0.0)? {
            return fallible!(FailedMap, "ln domain requires a positive interval");
        }
        Interval::new(x.lo.ln_to_::<Down>()?, x.hi.ln_to_::<Up>()?)
    }

    #[inline]
    fn sqrt_interval(x: Interval<Self>) -> Fallible<Interval<Self>> {
        if x.lo.lt_f64(0.0)? {
            return fallible!(FailedMap, "sqrt domain requires a nonnegative interval");
        }
        Interval::new(x.lo.sqrt_to_::<Down>()?, x.hi.sqrt_to_::<Up>()?)
    }
}

/// Backend capability for Gaussian best-effort special functions.
pub trait IntervalErfBackend: IntervalBackend + Sized {
    fn erfc_interval(x: Interval<Self>) -> Fallible<Interval<Self>>;
    fn erfcx_interval(x: Interval<Self>) -> Fallible<Interval<Self>>;
    fn erfc_inv_interval(x: Interval<Self>) -> Fallible<Interval<Self>>;
}

impl<T> IntervalErfBackend for T
where
    T: IntervalBackend,
    T::Hi: ErfTo<Output<Down> = T::Lo>,
    T::Lo: ErfTo<Output<Up> = T::Hi>,
{
    #[inline]
    fn erfc_interval(x: Interval<Self>) -> Fallible<Interval<Self>> {
        Interval::new(x.hi.erfc_to_::<Down>()?, x.lo.erfc_to_::<Up>()?)
    }

    #[inline]
    fn erfcx_interval(x: Interval<Self>) -> Fallible<Interval<Self>> {
        Interval::new(x.hi.erfcx_to_::<Down>()?, x.lo.erfcx_to_::<Up>()?)
    }

    #[inline]
    fn erfc_inv_interval(x: Interval<Self>) -> Fallible<Interval<Self>> {
        if x.lo.le_f64(0.0)? || !x.hi.lt_f64(2.0)? {
            return fallible!(
                FailedMap,
                "erfc_inv domain requires interval contained in (0, 2)"
            );
        }
        Interval::new(x.hi.erfc_inv_to_::<Down>()?, x.lo.erfc_inv_to_::<Up>()?)
    }
}

pub type AInterval = Interval<A>;
pub type BInterval = Interval<B>;
pub type CInterval = Interval<C>;
pub type DInterval = Interval<D>;

// Backwards-compatible aliases if existing call sites used "Ball" names.
pub type ABall = AInterval;
pub type BBall = BInterval;
pub type CBall = CInterval;
pub type DBall = DInterval;

/// Endpoint interval `[lo, hi]`.
#[derive(Clone)]
pub struct Interval<Bk: IntervalBackend> {
    lo: Bk::Lo,
    hi: Bk::Hi,
}

impl<Bk: IntervalBackend> fmt::Debug for Interval<Bk> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interval")
            .field("lo", &self.lower_f64())
            .field("hi", &self.upper_f64())
            .finish()
    }
}

impl<Bk: IntervalBackend> Interval<Bk> {
    #[inline]
    pub fn new(lo: Bk::Lo, hi: Bk::Hi) -> Fallible<Self> {
        if !Bk::ordered(&lo, &hi)? {
            return fallible!(
                FailedFunction,
                "invalid interval: lower endpoint exceeds upper endpoint"
            );
        }
        Ok(Self { lo, hi })
    }

    /// Exact point interval from a native f64.
    #[inline]
    pub fn point(value: f64) -> Fallible<Self> {
        Self::new(Bk::Lo::exact(value)?, Bk::Hi::exact(value)?)
    }

    /// Interval around an approximate native constant.
    #[inline]
    pub fn from_approx(value: f64) -> Fallible<Self> {
        Self::new(Bk::Lo::approx(value)?, Bk::Hi::approx(value)?)
    }

    /// Exact interval from native f64 endpoints.
    #[inline]
    pub fn between(lo: f64, hi: f64) -> Fallible<Self> {
        Self::new(Bk::Lo::exact(lo)?, Bk::Hi::exact(hi)?)
    }

    /// Interval from approximate native f64 endpoints, rounded outward.
    #[inline]
    pub fn between_approx(lo: f64, hi: f64) -> Fallible<Self> {
        Self::new(Bk::Lo::approx(lo)?, Bk::Hi::approx(hi)?)
    }

    #[inline]
    pub fn lower(&self) -> &Bk::Lo {
        &self.lo
    }

    #[inline]
    pub fn upper(&self) -> &Bk::Hi {
        &self.hi
    }

    #[inline]
    pub fn into_endpoints(self) -> (Bk::Lo, Bk::Hi) {
        (self.lo, self.hi)
    }

    #[inline]
    pub fn lower_f64(&self) -> Fallible<f64> {
        self.lo.to_f64()
    }

    #[inline]
    pub fn upper_f64(&self) -> Fallible<f64> {
        self.hi.to_f64()
    }

    #[inline]
    pub fn contains_zero(&self) -> Fallible<bool> {
        Ok(self.lo.le_f64(0.0)? && !self.hi.lt_f64(0.0)?)
    }

    #[inline]
    pub fn is_nonnegative(&self) -> Fallible<bool> {
        Ok(!self.lo.lt_f64(0.0)?)
    }

    #[inline]
    pub fn is_nonpositive(&self) -> Fallible<bool> {
        self.hi.le_f64(0.0)
    }
}

// -----------------------------------------------------------------------------
// Generic interval operations
// -----------------------------------------------------------------------------

impl<Bk: IntervalArithmeticBackend> Interval<Bk> {
    #[inline]
    pub fn add(self, rhs: Self) -> Fallible<Self> {
        Bk::add_interval(self, rhs)
    }

    #[inline]
    pub fn sub(self, rhs: Self) -> Fallible<Self> {
        Bk::sub_interval(self, rhs)
    }

    #[inline]
    pub fn neg(self) -> Fallible<Self> {
        Bk::neg_interval(self)
    }

    #[inline]
    pub fn mul(self, rhs: Self) -> Fallible<Self> {
        Bk::mul_interval(self, rhs)
    }

    #[inline]
    pub fn recip(self) -> Fallible<Self> {
        Bk::recip_interval(self)
    }

    #[inline]
    pub fn div(self, rhs: Self) -> Fallible<Self> {
        Bk::div_interval(self, rhs)
    }

    #[inline]
    pub fn abs(self) -> Fallible<Self> {
        Bk::abs_interval(self)
    }

    #[inline]
    pub fn abs_upper(self) -> Fallible<Bk::Hi> {
        Ok(Bk::abs_interval(self)?.hi)
    }
}

impl<Bk: IntervalExpBackend> Interval<Bk> {
    #[inline]
    pub fn exp(self) -> Fallible<Self> {
        Bk::exp_interval(self)
    }

    #[inline]
    pub fn exp_m1(self) -> Fallible<Self> {
        Bk::exp_m1_interval(self)
    }

    #[inline]
    pub fn ln(self) -> Fallible<Self> {
        Bk::ln_interval(self)
    }

    #[inline]
    pub fn sqrt(self) -> Fallible<Self> {
        Bk::sqrt_interval(self)
    }
}

impl<Bk: IntervalErfBackend> Interval<Bk> {
    /// Complementary error function. Decreasing on its domain.
    #[inline]
    pub fn erfc(self) -> Fallible<Self> {
        Bk::erfc_interval(self)
    }

    /// Scaled complementary error function. Treated as decreasing in the GDP
    /// regimes that use it.
    #[inline]
    pub fn erfcx(self) -> Fallible<Self> {
        Bk::erfcx_interval(self)
    }

    /// Inverse complementary error function. Decreasing on `(0, 2)`.
    #[inline]
    pub fn erfc_inv(self) -> Fallible<Self> {
        Bk::erfc_inv_interval(self)
    }
}

impl<Bk> Interval<Bk>
where
    Bk: IntervalBackend,
{
    #[inline]
    pub fn max(self, rhs: Self) -> Fallible<Self> {
        Self::new(self.lo.max(rhs.lo)?, self.hi.max(rhs.hi)?)
    }

    #[inline]
    pub fn min(self, rhs: Self) -> Fallible<Self> {
        Self::new(self.lo.min(rhs.lo)?, self.hi.min(rhs.hi)?)
    }

    #[inline]
    pub fn clamp(self, min: f64, max: f64) -> Fallible<Self> {
        self.max(Self::point(min)?)?.min(Self::point(max)?)
    }

    #[inline]
    pub fn clamp01(self) -> Fallible<Self> {
        self.clamp(0.0, 1.0)
    }
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

#[inline]
fn finite(value: f64) -> Fallible<f64> {
    value
        .is_finite()
        .then_some(value)
        .ok_or_else(|| err!(Overflow, "floating-point operation is not finite"))
}

#[inline]
fn normalize_fbig<R: ODPRound>(value: FBig<R>) -> FBig<R> {
    let precision = value.precision().max(DASHU_MIN_PRECISION);
    value.with_precision(precision).value()
}

#[inline]
fn min4_endpoint<T: Endpoint>(a: T, b: T, c: T, d: T) -> Fallible<T> {
    Ok(a.min(b)?.min(c.min(d)?)?)
}

#[inline]
fn max4_endpoint<T: Endpoint>(a: T, b: T, c: T, d: T) -> Fallible<T> {
    Ok(a.max(b)?.max(c.max(d)?)?)
}
