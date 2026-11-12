//! Verus proof harness for OpenDP stability over standalone query semantics.
#[path = "../../query_kernel/src/row.rs"]
pub mod verified;

#[path = "../../src/transformations/make_stable_vplan/stability.rs"]
pub mod stability;

#[path = "../../src/transformations/make_stable_vplan/certified.rs"]
pub mod certified;
