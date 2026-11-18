//! Verus harness compiling production OpenDP source, not parallel code.
// These are the production OpenDP core types and constructor source,
// imported directly; no parallel Measurement or callback model is defined.
#[path = "../../src/core/measurement_parts.rs"]
pub mod measurement_parts;
pub use measurement_parts::{Domain, Function, Measure, Metric, PrivacyMap};
#[path = "../../src/traits/operations/atom_traits.rs"]
pub mod atom_traits;
#[path = "../../src/domains/atom_domain.rs"]
pub mod atom_domain;
#[path = "../../src/metrics/discrete_distance.rs"]
pub mod discrete_distance;
#[path = "../../src/measures/max_divergence.rs"]
pub mod max_divergence;

#[path = "../../src/core/measurement.rs"]
pub mod core_measurement;
pub use core_measurement::{Measurement, MetricSpace};

#[path = "../../src/traits/samplers/bernoulli/uniform_word.rs"]
pub mod uniform_word;
#[path = "../../src/traits/samplers/bernoulli/dyadic53.rs"]
pub mod dyadic53;
#[path = "../../src/measurements/randomized_response/operator.rs"]
pub mod operator;
#[path = "../../src/traits/samplers/bernoulli/verus.rs"]
pub mod bernoulli_proof;
#[path = "../../src/measurements/randomized_response/verus.rs"]
pub mod randomized_response_bool;

#[path = "../../src/core/verus/measurement.rs"]
pub mod measurement_proof;

#[path = "../../src/traits/arithmetic/verified.rs"]
pub mod verified_arithmetic;

#[path = "../../src/measurements/randomized_response/constructor.rs"]
pub mod rr_constructor;
