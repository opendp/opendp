pub use arithmetic::*;
pub use cast::*;
pub use operations::*;

mod arithmetic;
mod cast;
mod operations;

/// A type that can be used as a stability or privacy constant to scale a distance.
/// Encapsulates the necessary traits for the new_from_constant method on relations.
/// Making a relation from a constant has the general form
///     d_out = QO::distance_cast(d_in) * c    (where d_out and c have type QO: DistanceConstant)
/// Computing this needs all of the traits DistanceConstant inherits from:
/// - InfCast<QI>: casting where the distance after the cast is gte the distance before the cast
/// - QO also clearly needs to support Mul and PartialOrd used in the general form above.
pub trait DistanceConstant<TI>: 'static + Clone + InfCast<TI> + InfMul + TotalOrd {}

impl<TI, TO> DistanceConstant<TI> for TO
    where TO: 'static + Clone + InfCast<TI> + InfMul + TotalOrd {}
