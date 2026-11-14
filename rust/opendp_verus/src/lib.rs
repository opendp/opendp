//! Verification-only Cargo harness. Proofs and executable kernels live
//! beside the corresponding production OpenDP modules.
#[path = "../../src/core/verus/measurement.rs"]
pub mod measurement;
#[path = "../../src/traits/samplers/bernoulli/dyadic53.rs"]
pub mod dyadic53;
#[path = "../../src/measurements/randomized_response/kernel.rs"]
pub mod kernel;
#[path = "../../src/measurements/randomized_response/verus.rs"]
pub mod randomized_response_bool;

#[path = "../../src/traits/samplers/bernoulli/verus.rs"]
pub mod bernoulli_proof;
