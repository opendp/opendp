//! Directed scalar arithmetic for OpenDP conservative numerics.

/// Directed rounding requested from a numerical operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Down,
    Up,
}

mod scalar;

pub use scalar::{Approximate, BestEffort, Certified, DirectedScalar, DirectedTranscendental, N64};
