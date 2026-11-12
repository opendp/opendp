//! Verus proof harness for OpenDP stability over standalone query semantics.
#[path = "../../query_kernel/src/row.rs"]
pub mod verified;

#[path = "../../src/transformations/make_verified_stable_lazyframe/stability.rs"]
pub mod stability;
