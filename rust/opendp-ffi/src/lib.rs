//! FFI bindings for OpenDP.
//!
//! # Overview
//!
//! This crate contains FFI bindings for [OpenDP](opendp).
//!
//! # Generic Functions
//!
//! OpenDP makes extensive use of generic types and functions. This presents problems for building an FFI interface.
//! The reason is because Rust performs monomorphization of generics when compiling. This means that a single generic function
//! is compiled to many different concrete functions, each one specific to its type parameters. That makes it impossible
//! to have an FFI function for a Rust generic function.
//!
//! ## Monomorphization
//!
//! Monomorphization is the way Rust resolves generic types at compile time. Rather than keeping any generic type information,
//! the compiler renders everything out into concrete versions using the specified types. For example, take the following code:
//! ```
//! // Code like this:
//!
//! fn hello<T: std::fmt::Display>(x: T) {
//!     println!("hello, {}!", x)
//! }
//!
//! fn main() {
//!     hello(10);
//!     hello(10.0);
//! }
//! ```
//! The compiler expands these functions into something like this:
//! ```
//! // Expands at compile time into code like this:
//!
//! fn hello_i32(x: i32) {
//!     println!("hello, {}!", x)
//! }
//! fn hello_f64(x: f64) {
//!     println!("hello, {}!", x)
//! }
//!
//! fn main() {
//!     hello_i32(10);
//!     hello_f64(10.0);
//! }
//! ```
//! Key points:
//! * Generic functions can't be called from FFI (because there isn't a single function!)
//! * Must have lexical call sites in Rust that calls any function with *all* desired concrete types
//!
//! In order to deal with this, we use a couple of different strategies, depending on the context.
//!
//! ## `Vec<Type>` and the Dispatch Pattern
//!
//! To work through a simple example, imagine we had a generic function like this:
//!
//! ## Dispatch Macro
//!
//! Why does this have to be a macro? Why couldn't we just determine the type at runtime? Because in order for the Rust compiler to monomorphize
//! a generic function, the there must be a concrete location in the code that invokes the function with the desired type. Rust has no runtime
//! notion of types. There's no way to have code that expands at runtime to a type that was unknown at compile time.
//!
//! * All generic parameters must be passed as references.
//!
//! In order
//!
//! # Combinators
//!
//! The dispatch pattern works well when the Cartesian product of all possible generic type parameters is relatively small. But if there is a large
//! number of type parameters, the number of match clauses (and the resulting monomorphizations) can become huge, making for very slow compile times.
//!
//! This becomes an issue with the OpenDP combinators. (TODO: link to module after moving combinators to separate module.)
//!
//! ## Glue Structs
//! ##
//!
//! # Memory Management
//!
//! # Bootstrap Metadata
//!
//! In order for external languages to know the FFI functions available, we use a simple bootstrapping mechanism. Every Rust module that wants to export
//! FFI functions must implement a function of the form `XXX__bootstrap()`, where `XXX` is the module path with `::` replaced by `_`. For instance,
//! the module `opendp_ffi::core` has a function `core::opendp_core__bootstrap`. This function returns a `*const c_char` containing a JSON object
//! specifying the FFI entry points of the module.
//!
//! // TODO: Show JSON example.

#![allow(clippy::upper_case_acronyms)]
#![allow(non_snake_case)]
#![recursion_limit="512"]
// #![feature(trace_macros)]
// trace_macros!(true);

#[macro_use]
extern crate lazy_static;

// internal module for err! macro resolution
mod error {
    pub use opendp::error::{Error, ErrorVariant};
}
// replacement for ? operator, for FfiResults
macro_rules! try_ {
    ($value:expr) => {
        match $value {
            Ok(x) => x,
            Err(e) => return e.into(),
        }
    }
}
// attempt to convert a raw pointer to a reference
//      as_ref      ok_or_else       try_!
// *mut T -> Option<&T> -> Fallible<&T> -> &T
macro_rules! try_as_ref {
    ($value:expr) => {
        try_!(crate::util::as_ref($value).ok_or_else(|| opendp::err!(FFI, concat!("null pointer: ", stringify!($value)))))
    }
}


#[macro_use]
mod dispatch;

pub mod any;
mod core;
mod data;
mod glue;
mod meas;
mod trans;
pub(crate) mod util;
#[cfg(feature="contrib")]
mod comb;
mod accuracy;
