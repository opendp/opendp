use crate::domains::InherentNull;
use crate::metrics::IntDistance;
use num::{One, Zero};
use std::hash::Hash;
use std::ops::{AddAssign, SubAssign, MulAssign};

mod bounded;
pub use bounded::*;

mod arithmetic;
pub use arithmetic::*;

mod cast;
pub use cast::*;

mod operations;
pub use operations::*;

pub mod samplers;

/// A type that can be used as a stability or privacy constant to scale a distance.
/// Encapsulates the necessary traits for the new_from_constant method on relations.
/// Making a relation from a constant has the general form
///     d_out = QO::distance_cast(d_in) * c    (where d_out and c have type QO: DistanceConstant)
/// Computing this needs all of the traits DistanceConstant inherits from:
/// - InfCast<QI>: casting where the distance after the cast is gte the distance before the cast
/// - QO also clearly needs to support Mul and PartialOrd used in the general form above.
pub trait DistanceConstant<TI>: 'static + Clone + InfCast<TI> + InfMul + TotalOrd {}

impl<TI, TO> DistanceConstant<TI> for TO where TO: 'static + Clone + InfCast<TI> + InfMul + TotalOrd {}

// Primitives are the broadest set of valid atomic types.
pub trait Primitive: 'static + Clone + std::fmt::Debug + CheckNull + PartialEq + Default {}
impl<T> Primitive for T where T: 'static + Clone + std::fmt::Debug + CheckNull + PartialEq + Default {}

// Hashable types are the subset of primitive types that implement Eq and Hash.
// They can be used as HashMap keys and in HashSets.
pub trait Hashable: Primitive + Eq + Hash {}
impl<T> Hashable for T where T: Primitive + Eq + Hash {}

// Number types are the subset of primitive types that have numerical operations.
pub trait Number:
    Primitive
    + Copy
    + AlertingAbs
    + SaturatingAdd
    + SaturatingMul
    + InfAdd
    + InfSub
    + InfMul
    + InfDiv
    + TotalOrd
    + Zero
    + One
    + PartialEq
    + AddAssign
    + SubAssign
    + MulAssign
    + FiniteBounds
    + ExactIntCast<usize>
    + ExactIntCast<i32>
    + InfCast<IntDistance>
    + InfCast<usize>
    + std::iter::Sum<Self>
{
}
impl<T> Number for T where
    T: Primitive
        + Copy
        + AlertingAbs
        + SaturatingAdd
        + SaturatingMul
        + InfAdd
        + InfSub
        + InfMul
        + InfDiv
        + TotalOrd
        + Zero
        + One
        + PartialEq
        + AddAssign
        + SubAssign
        + MulAssign
        + FiniteBounds
        + ExactIntCast<usize>
        + ExactIntCast<i32>
        + InfCast<IntDistance>
        + InfCast<usize>
        + std::iter::Sum<Self>
{
}

// Integers are hashable numbers. This excludes floats.
pub trait Integer: Number + Hashable {}
impl<T> Integer for T where T: Number + Hashable {}

// f32 or f64
pub trait Float:
    Number
    + num::Float
    + InherentNull
    + InfLn
    + InfLn1P
    + InfLog2
    + InfExp
    + InfExpM1
    + InfPow
    + InfSqrt
    + CastInternalReal
    + FloatBits
    + ExactIntCast<Self::Bits>
    + RoundCast<f64>
{
}
impl<T> Float for T where
    T: Number
        + num::Float
        + InherentNull
        + InfLn
        + InfLn1P
        + InfLog2
        + InfExp
        + InfExpM1
        + InfPow
        + InfSqrt
        + CastInternalReal
        + FloatBits
        + ExactIntCast<Self::Bits>
        + RoundCast<f64>
{
}
