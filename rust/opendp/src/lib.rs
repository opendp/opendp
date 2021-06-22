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
//! [`PrivacyRelation`]: core::PrivacyRelation
//! [`StabilityRelation`]: core::StabilityRelation
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
//! In addition, there's a companion crate, opendp-ffi, which provides FFI wrappers for opendp functionality.
//! This can be used to implement bindings in languages other than Rust.
//!
//! # User Guide
//!
//! OpenDP applications are created by using constructors and combinators to create private computation pipelines.
//! These can be written directly in Rust, or by using a language binding that uses OpenDP through an FFI interface.
//! Python is the first language binding available, but we made add others in the future.
//!
//! ## Rust Application Example
//!
//! Here's a simple example of using OpenDP from Rust to create a private sum:
//! ```
//! use opendp::core;
//! use opendp::meas;
//! use opendp::trans;
//! use opendp::trans::{manipulation, sum, make_split_lines, make_cast_default, make_clamp, make_bounded_sum};
//! use opendp::dist::{HammingDistance, L1Distance};
//! use opendp::error::*;
//! use opendp::chain::{make_chain_tt, make_chain_mt};
//! use opendp::meas::make_base_laplace;
//! use opendp::dom::VectorDomain;
//!
//! pub fn example() -> Fallible<()> {
//!     let data = "56\n15\n97\n56\n6\n17\n2\n19\n16\n50".to_owned();
//!     let bounds = (0.0, 100.0);
//!     let epsilon = 1.0;
//!     let sigma = (bounds.1 - bounds.0) / epsilon;
//!
//!     // Construct a Transformation to load the numbers.
//!     let split_lines = make_split_lines::<HammingDistance>()?;
//!     let cast = make_cast_default::<HammingDistance, String, f64>()?;
//!     let load_numbers = make_chain_tt(&cast, &split_lines, None)?;
//!
//!     // Construct a Measurement to calculate a noisy sum.
//!     let clamp = make_clamp::<VectorDomain<_>, _>(bounds.0, bounds.1)?;
//!     let bounded_sum = make_bounded_sum(bounds.0, bounds.1)?;
//!     let laplace = make_base_laplace(sigma)?;
//!     let intermediate = make_chain_tt(&bounded_sum, &clamp, None)?;
//!     let noisy_sum = make_chain_mt(&laplace, &intermediate, None)?;
//!
//!     // Put it all together.
//!     let pipeline = make_chain_mt(&noisy_sum, &load_numbers, None)?;
//!     let result = pipeline.function.eval(&data)?;
//!     println!("result = {}", result);
//!     Ok(())
//! }
//! example().unwrap_test();
//! ```
//!
//! # Contributor Guide
//!
//! Contributions to OpenDP typically take the form of what we call "Components." A Component is shorthand for
//! the collection of code that comprises a  [`Measurement`] or [`Transformation`].
//!
//! ## Adding Components
//!
//! OpenDP components take the form of constructor functions that construct new instances of [`Measurement`]
//! and [`Transformation`]. Measurement constructors go in the module [`meas`], and Transformation constructors
//! in the module [`trans`]. (We'll probably split these up as they grow.)
//!
//! There are two steps to adding a constructor function: Writing the function itself, and adding the FFI wrapper.
//!
//! ### Writing Constructors
//!
//! Constructors are functions that take configuration parameters and return an appropriately configured [`Measurement`] or [`Transformation`].
//! They typically follow a common pattern:
//! 1. Choose the appropriate input and output [`Domain`].
//! 2. Write a closure that implements the [`Function`].
//! 3. Choose the appropriate input and output [`Metric`]/[`Measure`].
//! 4. Write a closure that implements the [`PrivacyRelation`]/[`StabilityRelation`].
//!
//! #### Example Transformation Constructor
//! ```
//!# use opendp::core::{Transformation, StabilityRelation, Function};
//!# use opendp::dist::L1Distance;
//!# use opendp::dom::AllDomain;
//! pub fn make_i32_identity() -> Transformation<AllDomain<i32>, AllDomain<i32>, L1Distance<i32>, L1Distance<i32>> {
//!     let input_domain = AllDomain::new();
//!     let output_domain = AllDomain::new();
//!     let function = Function::new(|arg: &i32| -> i32 { *arg });
//!     let input_metric = L1Distance::default();
//!     let output_metric = L1Distance::default();
//!     let stability_relation = StabilityRelation::new_from_constant(1);
//!     Transformation::new(input_domain, output_domain, function, input_metric, output_metric, stability_relation)
//! }
//! make_i32_identity();
//! ```
//!
//! #### Input and Output Types
//!
//! The [`Function`] created in a constructor is allowed to have any type for its input and output [`Domain::Carrier`].
//! There's no need for special data carrying wrappers. The clue code in the FFI layer handles this transparently.
//! However, the most common are the Rust primitives (e.g., `i32`, `f64`, etc.), and collections of the primitives
//! (`Vec<i32>`, `HashMap<String, f64>`).
//!
//!
//! #### Handling Generics
//!
//! [`Measurement`]/[`Transformation`] constructors are allowed to be generic! Typically, this means that the type parameter on the
//! constructor will determine type of the input or output [`Domain::Carrier`] (or the generic type within, for instance the `i32` of `Vec<i32>`).

#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::just_underscores_and_digits)]
#![allow(clippy::type_complexity)]

// create clones of variables that are free to be consumed by a closure
macro_rules! enclose {
    ( $x:ident, $y:expr ) => (enclose!(($x), $y));
    ( ($( $x:ident ),*), $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}
macro_rules! num_cast {
    ($v:expr; $ty:ty) => (<$ty as num::NumCast>::from($v).ok_or_else(|| err!(FailedCast)))
}

#[macro_use]
pub mod error;

pub mod chain;
pub mod core;
pub mod data;
pub mod dist;
pub mod dom;
pub mod interactive;
pub mod meas;
pub mod poly;
pub mod samplers;
pub mod traits;
pub mod trans;
