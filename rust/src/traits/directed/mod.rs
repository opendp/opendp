//! Directed scalar arithmetic for OpenDP conservative numerics.

pub mod backend;
mod scalar;

pub use backend::{Dashu, Direction, SoftFloat};
pub use scalar::{Approximate, BestEffort, Certified, DirectedScalar, DirectedTranscendental, N64};
