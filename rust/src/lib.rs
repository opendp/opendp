//! A library for working with differential privacy.
//!
//! This library implements the framework described in the paper,
//! [A Programming Framework for OpenDP](https://opendp.org/files/2025/11/opendp_programming_framework_11may2020_1_01.pdf).
//! OpenDP (the library) is part of the larger [OpenDP Project](https://opendp.org).
//!
//! [`Domain`]: core::Domain
//! [`Domain::Carrier`]: core::Domain::Carrier
//! [`Function`]: core::Function
//! [`Metric`]: core::Metric
//! [`Measure`]: core::Measure
//! [`PrivacyMap`]: core::PrivacyMap
//! [`StabilityMap`]: core::StabilityMap
//! [`Measurement`]: core::Measurement
//! [`Transformation`]: core::Transformation
//!
//! # Overview
//!
//! OpenDP provides three main concepts:
//! * A flexible architecture for modeling privacy-preserving computations.
//! * Implementations of several common algorithms for statistical analysis and data manipulation, which can be used
//! out-of-the-box to assemble DP applications.
//! * Facilities for extending OpenDP with new algorithms, privacy models, etc.
//!
//! # User Guide
//!
//! A more thorough User Guide [can be found on the docs website](https://docs.opendp.org/en/stable/api/user-guide/index.html).
//!
//! OpenDP applications are created by using constructors and combinators to create private computation pipelines.
//! These can be written directly in Rust, or by using a language binding that uses OpenDP through an FFI interface.
//! Python is the first language binding available, but we plan to add others in the future.
//!
//!
//! ## Rust Application Example
//!
//! Here's an example of creating a private sum from a vector:
//! ```
//! use opendp::error::Fallible;
//!
//! #[cfg(all(feature = "untrusted", feature = "partials"))]
//! pub fn example() -> Fallible<()> {
//!     use opendp::combinators::{make_chain_mt, make_chain_tt};
//!     use opendp::domains::{AtomDomain, VectorDomain};
//!     use opendp::measures::MaxDivergence;
//!     use opendp::measurements::{make_laplace, then_laplace};
//!     use opendp::metrics::SymmetricDistance;
//!     use opendp::transformations::{make_clamp, make_impute_constant, make_sum, then_clamp, then_impute_constant, then_sum};
//!
//!     let data = vec![56., 15., f64::NAN, 56., 6., 17., 2., 19., 16., 50.];
//!     let bounds = (0.0, 100.0);
//!     let epsilon = 1.0;
//!     // Leave a little slack for floating-point error.
//!     let sigma = (bounds.1 - bounds.0) / (epsilon - 0.0001);
//!
//!     let input_domain = VectorDomain::new(AtomDomain::<f64>::default());
//!     let input_metric = SymmetricDistance;
//!
//!     // Impute NaN values with 0.0, producing a domain without NaNs.
//!     // Clamping requires non-NaN input values.
//!     let impute = make_impute_constant(input_domain.clone(), input_metric.clone(), 0.0)?;
//!
//!     // A transformation's output domain and metric must match the next one's input.
//!     let clamp = make_clamp(impute.output_domain.clone(), impute.output_metric.clone(), bounds)?;
//!     let imputed_and_clamped = make_chain_tt(&clamp, &impute)?;
//!     let sum = make_sum(clamp.output_domain.clone(), clamp.output_metric.clone())?;
//!     let bounded_sum = make_chain_tt(&sum, &imputed_and_clamped)?;
//!
//!     let laplace = make_laplace::<_, _, MaxDivergence>(
//!         sum.output_domain.clone(), sum.output_metric.clone(), sigma, None
//!     )?;
//!     let noisy_sum = make_chain_mt(&laplace, &bounded_sum)?;
//!
//!     // The >> operator and then_ constructors make the same chain more concise.
//!     let concise_noisy_sum = (
//!         (input_domain, input_metric)
//!         >> then_impute_constant(0.0)
//!         >> then_clamp(bounds)
//!         >> then_sum()
//!         >> then_laplace(sigma, None)
//!     )?;
//!
//!     assert!(noisy_sum.check(&1, &epsilon)?);
//!     assert!(concise_noisy_sum.check(&1, &epsilon)?);
//!     let release = concise_noisy_sum.invoke(&data)?;
//!     println!("release = {}", release);
//!     Ok(())
//! }
//! #[cfg(all(feature = "untrusted", feature = "partials"))]
//! example().unwrap();
//! ```
//!
//! # Contributor Guide
//!
//! A more thorough Contributor Guide [can be found on the docs website](https://docs.opendp.org/en/stable/contributing/index.html).
//!
//! ## Adding Constructors
//!
//! Measurement constructors go in the module [`crate::measurements`],
//! Transformation constructors in the module [`crate::transformations`], and
//! Combinator constructors in the module [`crate::combinators`].
//!
//! There are two code steps to adding a constructor function: Writing the function itself, and adding the FFI wrapper.
//!
//! ### Writing Constructors
//!
//! Constructors are functions that take some parameters and return a valid [`Measurement`] or [`Transformation`].
//! They typically follow a common pattern:
//! 1. Choose the appropriate input and output [`Domain`].
//! 2. Write a closure that implements the [`Function`].
//! 3. Choose the appropriate input and output [`Metric`]/[`Measure`].
//! 4. Write a closure that implements the [`PrivacyMap`]/[`StabilityMap`].
//!
//! #### Example Transformation Constructor
//! ```
//!# use opendp::core::{Transformation, StabilityMap, Function};
//!# use opendp::metrics::AbsoluteDistance;
//!# use opendp::domains::AtomDomain;
//! pub fn make_i32_identity() -> Transformation<AtomDomain<i32>, AbsoluteDistance<i32>, AtomDomain<i32>, AbsoluteDistance<i32>> {
//!     let input_domain = AtomDomain::default();
//!     let input_metric = AbsoluteDistance::default();
//!     let output_domain = AtomDomain::default();
//!     let output_metric = AbsoluteDistance::default();
//!     let function = Function::new(|arg: &i32| -> i32 { *arg });
//!     let stability_map = StabilityMap::new_from_constant(1);
//!     Transformation::new(input_domain, input_metric, output_domain, output_metric, function, stability_map).unwrap()
//! }
//! make_i32_identity();
//! ```
//!
//! #### Input and Output Types
//!
//! The [`Function`] created in a constructor is allowed to have any type for its input and output [`Domain::Carrier`].
//! There's no need for special data carrying wrappers. The glue code in the FFI layer handles this transparently.
//! However, the most common are the Rust primitives (e.g., `i32`, `f64`, etc.), and collections of the primitives
//! (`Vec<i32>`, `HashMap<String, f64>`).
//!
//!
//! #### Handling Generics
//!
//! [`Measurement`]/[`Transformation`] constructors are allowed to be generic! Typically, this means that the type parameter on the
//! constructor will determine type of the input or output [`Domain::Carrier`] (or the generic type within, for instance the `i32` of `Vec<i32>`).

#![allow(clippy::just_underscores_and_digits)]
#![allow(clippy::type_complexity)]
#![cfg_attr(feature = "ffi", allow(clippy::upper_case_acronyms))]
#![cfg_attr(feature = "ffi", allow(non_snake_case))]
#![recursion_limit = "512"]

// create clones of variables that are free to be consumed by a closure
// Once we have things using `enclose!` that are outside of `contrib`, this should specify `feature="ffi"`.
#[cfg(feature = "contrib")]
macro_rules! enclose {
    ( $x:ident, $y:expr ) => (enclose!(($x), $y));
    ( ($( $x:ident ),*), $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

// #![feature(trace_macros)]
// trace_macros!(true);

#[cfg(feature = "ffi")]
#[macro_use]
pub mod ffi;
#[cfg(feature = "ffi")]
#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod error;

pub mod accuracy;
pub mod combinators;
pub mod core;
pub mod data;
pub mod domains;
pub mod interactive;
#[cfg(feature = "ffi")]
pub mod internal;
pub mod measurements;
pub mod measures;
pub mod metrics;
pub mod traits;
pub mod transformations;
pub mod utilities;

#[cfg(all(feature = "polars", feature = "contrib"))]
pub mod polars;
