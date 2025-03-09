//! A library for working with differential privacy.
//!
//! This library implements the framework described in the paper,
//! [A Programming Framework for OpenDP](https://projects.iq.harvard.edu/files/opendp/files/opendp_programming_framework_11may2020_1_01.pdf).
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
//! Here's a simple example of using OpenDP from Rust to create a private sum:
//! ```
//! use opendp::error::Fallible;
//!
//! #[cfg(all(feature = "untrusted", feature = "partials"))]
//! pub fn example() -> Fallible<()> {
//!     use opendp::transformations::{make_split_lines, then_cast_default, make_cast_default, then_clamp, then_sum};
//!     use opendp::combinators::{make_chain_tt, make_chain_mt};
//!     use opendp::measurements::then_laplace;
//!
//!     let data = "56\n15\n97\n56\n6\n17\n2\n19\n16\n50".to_owned();
//!     let bounds = (0.0, 100.0);
//!     let epsilon = 1.0;
//!     // remove some epsilon to account for floating-point error
//!     let sigma = (bounds.1 - bounds.0) / (epsilon - 0.0001);
//!
//!     // Construct a Transformation to parse a csv string.
//!     let split_lines = make_split_lines()?;
//!
//!     // The next transformation wants to conform with the output domain and metric from `split_lines`.
//!     let cast = make_cast_default::<_, String, f64>(
//!         split_lines.output_domain.clone(),
//!         split_lines.output_metric.clone())?;
//!
//!     // Since the domain and metric conforms, these two transformations may be chained.
//!     let load_numbers = make_chain_tt(&cast, &split_lines)?;
//!      
//!     // You can use the more convenient `>>` notation to chain instead.
//!     // When you use the `then_` version of the constructor,
//!     //     the `>>` operator will automatically fill the input domain and metric from the previous transformation.
//!     let load_and_clamp = load_numbers >> then_clamp(bounds);
//!     
//!     // After chaining, the resulting transformation is wrapped in a `Result`.
//!     let load_and_sum = (load_and_clamp >> then_sum())?;
//!
//!     // Construct a Measurement to calculate a noisy sum.
//!     let noisy_sum = load_and_sum >> then_laplace(sigma, None);
//!
//!     // The same measurement, written more succinctly:
//!     let noisy_sum = (
//!         make_split_lines()? >>
//!         then_cast_default() >>
//!         then_clamp(bounds) >>
//!         then_sum() >>
//!         then_laplace(sigma, None)
//!     )?;
//!
//!     // Check that the pipeline is (1, 1.0)-close
//!     assert!(noisy_sum.check(&1, &epsilon)?);
//!
//!     // Make a 1.0-epsilon-DP release
//!     let release = noisy_sum.invoke(&data)?;
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
//! pub fn make_i32_identity() -> Transformation<AtomDomain<i32>, AtomDomain<i32>, AbsoluteDistance<i32>, AbsoluteDistance<i32>> {
//!     let input_domain = AtomDomain::default();
//!     let output_domain = AtomDomain::default();
//!     let function = Function::new(|arg: &i32| -> i32 { *arg });
//!     let input_metric = AbsoluteDistance::default();
//!     let output_metric = AbsoluteDistance::default();
//!     let stability_map = StabilityMap::new_from_constant(1);
//!     Transformation::new(input_domain, output_domain, function, input_metric, output_metric, stability_map).unwrap()
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
#[cfg(feature = "contrib")]
pub mod interactive;
#[cfg(feature = "ffi")]
pub mod internal;
pub mod measurements;
pub mod measures;
pub mod metrics;
pub mod traits;
pub mod transformations;

#[cfg(feature = "polars")]
pub mod polars;
