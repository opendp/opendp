//! Directed scalar arithmetic for OpenDP conservative numerics.

/// Directed rounding requested from a numerical operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Down,
    Up,
}

pub mod backend;

mod scalar;

pub use backend::{Dashu, SoftFloat};
pub use scalar::{Approximate, Certified, DirectedScalar, DirectedTranscendental, N64};
