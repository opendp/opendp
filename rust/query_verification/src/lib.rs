//! Verus proof harness for OpenDP stability over standalone query semantics.
#[path = "../../query_kernel/src/row.rs"]
pub mod verified;

#[path = "../../src/transformations/make_stable_vplan/stability.rs"]
pub mod stability;

#[path = "../../src/transformations/make_stable_vplan/input/proof.rs"]
pub mod input_proof;

#[path = "../../src/transformations/make_stable_vplan/filter/proof.rs"]
pub mod filter_proof;

#[path = "../../src/transformations/make_stable_vplan/project/proof.rs"]
pub mod project_proof;

#[path = "../../src/transformations/make_stable_vplan/table/proof.rs"]
pub mod table_proof;

#[path = "../../src/transformations/make_stable_vplan/deferred/proof.rs"]
pub mod deferred_proof;
