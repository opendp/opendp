//! Traits that enable building stable and private algorithms.

use crate::metrics::IntDistance;
use num::{NumCast, One, Zero};
use std::hash::Hash;
use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

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
///
/// Encapsulates the necessary traits for the new_from_constant method on maps.
/// Making a map from a constant has the general form:
///
/// ```text
/// d_out = TO::distance_cast(d_in.clone())?.inf_mul(c)?
/// ```
/// (where d_out and c are of type TO, which implements DistanceConstant)
///
/// - `InfCast<TI>` is for casting where the distance after the cast is gte the distance before the cast
/// - `InfMul` is to multiply with the constant `c` in a way that doesn't round down
/// - `ProductOrd` is now only for convenience
///
/// # Example
/// ```
/// use opendp::traits::DistanceConstant;
/// use opendp::error::Fallible;
/// fn example_map<TI, TO: DistanceConstant<TI>>(d_in: TI, c: TO) -> Fallible<TO> {
///     TO::inf_cast(d_in)?.inf_mul(&c)
/// }
///
/// assert_eq!(example_map(3.14159_f32, 2_i8).ok(), Some(8_i8));
/// // same thing, but annotate types in a different way
/// assert_eq!(example_map::<f32, i8>(3.14159, 2).ok(), Some(8));
/// ```
pub trait DistanceConstant<TI>:
    'static + InfCast<TI> + InfMul + ProductOrd + Zero + Send + Sync
{
}

impl<TI, TO> DistanceConstant<TI> for TO where
    TO: 'static + InfCast<TI> + InfMul + ProductOrd + Zero + Send + Sync
{
}

/// A shorthand to indicate the set of types that implement the most common traits, like Clone and Debug.
///
/// The other rollup traits [`Hashable`], [`Number`], [`Integer`] and [`Float`] inherit from this trait.
///
/// Examples: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, bool, String.
///
/// Refer to the constituent traits to see proof definitions on methods.
///
/// # Example
/// ```
/// use opendp::traits::Primitive;
/// fn test_func<T: Primitive>(value: T) {
///     // can be debugged
///     println!("{value:?}");
///
///     // default values exist and members of type T can be compared
///     assert_eq!(T::default(), T::default());
///
///     // can check if is null
///     value.is_null();
/// }
///
/// test_func(1i8);
/// ```
pub trait Primitive:
    'static
    + Clone
    + std::fmt::Debug
    + std::fmt::Display
    + CheckNull
    + PartialEq
    + Default
    + CheckAtom
    + Send
    + Sync
{
}
impl<T> Primitive for T where
    T: 'static
        + Clone
        + std::fmt::Debug
        + std::fmt::Display
        + CheckNull
        + PartialEq
        + Default
        + CheckAtom
        + Send
        + Sync
{
}

/// The subset of [`Primitive`] types that implement Eq and Hash.
///
/// Hashable types can be used as HashMap keys and in HashSets.
///
/// Examples: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, bool, String
///
/// This trait lists the traits that are implemented for hashable types.
/// Refer to the constituent traits to see proof definitions on methods.
///
/// # Example
/// ```
/// use opendp::traits::Hashable;
/// use std::collections::HashSet;
/// fn test_func<T: Hashable>(value: T) {
///     // can be debugged, as Hashable inherits all traits from Primitive
///     println!("{value:?}");
///     
///     // can be used in hash sets and in the keys of hashmaps
///     let mut hashset = HashSet::new();
///     hashset.insert(value);
/// }
///
/// test_func("apple".to_string());
/// ```
pub trait Hashable: Primitive + Eq + Hash {}
impl<T> Hashable for T where T: Primitive + Eq + Hash {}

/// The subset of [`Primitive`] types that have numerical operations.
///
/// Examples: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64
///
/// This trait lists many traits that are implemented for numerical types.
/// It is a shorthand to provide broad numerical functionality to a generic type,
/// without polluting trait bounds with a large number of highly-specific traits.
///
/// Refer to the constituent traits to see proof definitions on methods.
///
/// # Example
/// ```
/// use opendp::traits::Number;
/// fn test_func<T: Number>(value: T) {
///     // can be debugged, as Number inherits all traits from Primitive:
///     println!("{value:?}");
///     
///     // supports basic arithmetic and numerical properties
///     assert_eq!(T::zero().inf_mul(&value).ok(), Some(T::zero()));
/// }
///
/// test_func(1i8);
/// ```

pub trait Number:
    Primitive
    + Copy
    + NumCast
    + AlertingAbs
    + num::traits::NumOps
    + SaturatingAdd
    + SaturatingMul
    + InfAdd
    + InfSub
    + InfMul
    + InfDiv
    + ProductOrd
    + Zero
    + One
    + PartialEq
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + FiniteBounds
    + ExactIntCast<usize>
    + ExactIntCast<i32>
    + InfCast<IntDistance>
    + InfCast<usize>
    + std::iter::Sum<Self>
    + for<'a> std::iter::Sum<&'a Self>
    + DistanceConstant<Self>
{
}
impl<T> Number for T where
    T: Primitive
        + Copy
        + NumCast
        + AlertingAbs
        + num::traits::NumOps
        + SaturatingAdd
        + SaturatingMul
        + InfAdd
        + InfSub
        + InfMul
        + InfDiv
        + ProductOrd
        + Zero
        + One
        + PartialEq
        + AddAssign
        + SubAssign
        + MulAssign
        + DivAssign
        + FiniteBounds
        + ExactIntCast<usize>
        + ExactIntCast<i32>
        + InfCast<IntDistance>
        + InfCast<usize>
        + std::iter::Sum<Self>
        + for<'a> std::iter::Sum<&'a Self>
        + DistanceConstant<Self>
{
}

/// The intersection of [`Number`] types and [`Hashable`] types.
/// This happens to be integers.
///
/// Examples: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize
///
/// This trait lists many traits that are implemented for integer types.
/// It is a shorthand to provide broad integer functionality to a generic type,
/// without polluting trait bounds with a large number of highly-specific traits.
///
/// Refer to the constituent traits to see proof definitions on methods.
///
/// # Example
/// ```
/// use opendp::traits::Integer;
/// use std::collections::HashSet;
/// fn test_func<T: Integer>(value: T) {
///     // can be debugged
///     println!("{value:?}");
///
///     // supports arithmetic and has numerical properties
///     assert_eq!(T::zero().inf_mul(&value).ok(), Some(T::zero()));
///     
///     // can be used in hash sets and in the keys of hashmaps:
///     let mut hashset = HashSet::new();
///     hashset.insert(value);
/// }
///
/// test_func(1i8);
/// ```
pub trait Integer: Number + Hashable + Ord {}
impl<T> Integer for T where T: Number + Hashable + Ord {}

/// Floating-point types.
///
/// Examples: f32, f64
///
/// This trait lists many traits that are implemented for floating-point types.
/// It is a shorthand to provide broad floating-point functionality to a generic type,
/// without polluting trait bounds with a large number of highly-specific traits.
///
/// Refer to the constituent traits to see proof definitions on methods.
///
/// # Example
/// ```
/// use opendp::traits::Float;
/// fn test_func<T: Float>(value: T) {
///     // can be debugged, as Integer inherits all traits from Primitive:
///     println!("{value:?}");
///
///     // supports arithmetic and has numerical properties
///     assert_eq!(T::zero().inf_mul(&value).ok(), Some(T::zero()));
/// }
///
/// test_func(3.14159);
/// ```
pub trait Float:
    Number
    + num::Float
    + InherentNull
    + InfLn
    + InfLn1P
    + InfLog2
    + InfExp
    + InfExpM1
    + InfPowI
    + InfSqrt
    + FloatBits
    + CastInternalRational
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
        + InfPowI
        + InfSqrt
        + FloatBits
        + CastInternalRational
        + ExactIntCast<Self::Bits>
        + RoundCast<f64>
{
}
