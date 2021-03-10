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
//! use opendp::trans::{MakeTransformation0, MakeTransformation1, MakeTransformation2, MakeTransformation3};
//! use opendp::dist::{HammingDistance, L1Sensitivity};
//! use opendp::core::{ChainTT, ChainMT};
//! use opendp::meas::{MakeMeasurement2, LaplaceMechanism, MakeMeasurement1};
//!
//! pub fn example() {
//!     let data = "56\n15\n97\n56\n6\n17\n2\n19\n16\n50".to_owned();
//!     let bounds = (0.0, 100.0);
//!     let epsilon = 1.0;
//!     let sigma = (bounds.1 - bounds.0) / epsilon;
//!
//!     // Construct a Transformation to load the numbers.
//!     let split_lines = trans::SplitLines::<HammingDistance>::make();
//!     let parse_series = trans::ParseSeries::<f64, HammingDistance>::make(true);
//!     let load_numbers = ChainTT::make(&parse_series, &split_lines);
//!
//!     // Construct a Measurement to calculate a noisy sum.
//!     let clamp = trans::Clamp::make(bounds.0, bounds.1);
//!     let bounded_sum = trans::BoundedSum::make2(bounds.0, bounds.1);
//!     let laplace = LaplaceMechanism::make(sigma);
//!     let intermediate = ChainTT::make(&bounded_sum, &clamp);
//!     let noisy_sum = ChainMT::make(&laplace, &intermediate);
//!
//!     // Put it all together.
//!     let pipeline = ChainMT::make(&noisy_sum, &load_numbers);
//!     let result = pipeline.function.eval(&data);
//!     println!("result = {}", result);
//!  }
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
//!# use opendp::core::Transformation;
//!# use opendp::dist::L1Sensitivity;
//!# use opendp::dom::AllDomain;
//! pub fn make_i32_identity() -> Transformation<AllDomain<i32>, AllDomain<i32>, L1Sensitivity<i32>, L1Sensitivity<i32>> {
//!     let input_domain = AllDomain::new();
//!     let output_domain = AllDomain::new();
//!     let function = |arg: &i32| -> i32 { *arg };
//!     let input_metric = L1Sensitivity::new();
//!     let output_metric = L1Sensitivity::new();
//!     let stability_relation = |d_in: &i32, d_out: &i32| *d_out >= *d_in;
//!     Transformation::new(input_domain, output_domain, function, input_metric, output_metric, stability_relation)
//! }
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

macro_rules! enclose {
    ( $x:ident, $y:expr ) => (enclose!(($x), $y));
    ( ($( $x:ident ),*), $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub mod core;
pub mod data;
pub mod dist;
pub mod dom;
pub mod meas;
pub mod trans;
pub mod traits;
