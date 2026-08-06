//! OpenDP-owned software floating-point values.

use core::fmt;

mod dashu;
pub use dashu::Dashu;

/// Directed rounding requested from a numerical operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Down,
    Up,
}

mod private {
    use core::fmt;

    /// Associates a provider marker with its private representation.
    ///
    /// Numerical operations are intentionally not abstracted here. Each
    /// concrete `SoftFloat` implements OpenDP's semantic traits directly.
    pub trait SoftFloatBackend {
        type Repr: Clone + fmt::Debug;
    }
}

pub(super) use private::SoftFloatBackend;

/// Opaque OpenDP-owned software floating-point value.
///
/// The provider marker selects storage only. Arithmetic semantics are supplied
/// by concrete [`crate::traits::DirectedScalar`] and
/// [`crate::traits::DirectedTranscendental`] implementations.
#[allow(private_bounds)]
#[derive(Clone)]
pub struct SoftFloat<B: SoftFloatBackend> {
    pub(super) repr: B::Repr,
}

impl<B: SoftFloatBackend> fmt::Debug for SoftFloat<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SoftFloat").field(&self.repr).finish()
    }
}
