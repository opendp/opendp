//! Directed scalar arithmetic for OpenDP conservative numerics.

/// Directed rounding requested from a numerical operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Down,
    Up,
}

mod scalar;
mod soft_float;

pub use scalar::{Approximate, Certified, DirectedScalar, DirectedTranscendental, N64};
pub use soft_float::SoftFloat;
