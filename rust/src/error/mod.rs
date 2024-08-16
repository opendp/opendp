//! Error handling utilities.

use std::fmt;
use std::fmt::Debug;

use std::backtrace::Backtrace as _Backtrace;

use dashu::base::ConversionError;
#[cfg(feature = "polars")]
use polars::prelude::PolarsError;

/// Create an instance of [`Fallible`]
#[macro_export]
macro_rules! fallible {
    ($variant:ident) => (Err(err!($variant)));
    ($variant:ident, $($inner:expr),+) => (Err(err!($variant, $($inner),+)));
}
// "error" is shadowed, and breaks intellij macro resolution
/// Create an instance of [`Error`]
#[macro_export]
macro_rules! err {
    // error without message
    ($variant:ident) => ($crate::error::Error {
        variant: $crate::error::ErrorVariant::$variant,
        message: None,
        backtrace: err!(@backtrace)
    });
    // error with explicit message
    ($variant:ident, $message:expr) =>
        (err!(@new $variant, format!($message)));
    // error with template formatting
    ($variant:ident, $template:expr, $($args:expr),+) =>
        (err!(@new $variant, format!($template, $($args,)+)));

    (@new $variant:ident, $message:expr) => ($crate::error::Error {
        variant: $crate::error::ErrorVariant::$variant,
        message: Some($message),
        backtrace: err!(@backtrace)
    });

    (@backtrace) => (std::backtrace::Backtrace::capture());
}

#[derive(thiserror::Error)]
pub struct Error {
    pub variant: ErrorVariant,
    pub message: Option<String>,
    pub backtrace: _Backtrace,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.variant == other.variant && self.message == other.message
    }
}

#[derive(PartialEq, thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ErrorVariant {
    #[error("FFI")]
    FFI,

    #[error("TypeParse")]
    TypeParse,

    #[error("FailedFunction")]
    FailedFunction,

    #[error("FailedMap")]
    FailedMap,

    #[error("RelationDebug")]
    RelationDebug,

    #[error("FailedCast")]
    FailedCast,

    #[error("DomainMismatch")]
    DomainMismatch,

    #[error("MetricMismatch")]
    MetricMismatch,

    #[error("MeasureMismatch")]
    MeasureMismatch,

    #[error("MakeDomain")]
    MakeDomain,

    #[error("MakeTransformation")]
    MakeTransformation,

    #[error("MakeMeasurement")]
    MakeMeasurement,

    #[error("MetricSpace")]
    MetricSpace,

    #[error("InvalidDistance")]
    InvalidDistance,

    #[error("Overflow")]
    Overflow,

    #[error("NotImplemented")]
    NotImplemented,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.variant)
    }
}

// simplify error creation from vega_lite_4
#[cfg(all(test, feature = "test-plot"))]
impl From<String> for Error {
    fn from(v: String) -> Self {
        err!(FailedFunction, "{}", v)
    }
}

#[cfg(feature = "polars")]
impl From<Error> for PolarsError {
    fn from(value: Error) -> Self {
        PolarsError::ComputeError(value.to_string().into())
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: {:?}\n{}",
            self.variant,
            self.message.as_ref().cloned().unwrap_or_default(),
            self.backtrace.to_string()
        )
    }
}

impl From<ErrorVariant> for Error {
    fn from(variant: ErrorVariant) -> Self {
        Self {
            variant,
            message: None,
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }
}

impl<T> From<Error> for Result<T, Error> {
    fn from(e: Error) -> Self {
        Err(e)
    }
}

impl From<ConversionError> for Error {
    fn from(err: ConversionError) -> Self {
        Self {
            variant: ErrorVariant::FailedCast,
            message: Some(err.to_string()),
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }
}

#[cfg(feature = "polars")]
impl From<PolarsError> for Error {
    fn from(error: PolarsError) -> Self {
        Self {
            variant: ErrorVariant::FailedFunction,
            message: Some(format!("{:?}", error)),
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }
}

pub type Fallible<T> = Result<T, Error>;

/// A trait for calling unwrap with an explanation. Makes calls to unwrap() discoverable.
pub trait ExplainUnwrap {
    type Inner;
    /// use if the None or Err variant is structurally unreachable
    fn unwrap_assert(self, explanation: &'static str) -> Self::Inner;
    /// use in tests, where panics are acceptable
    fn unwrap_test(self) -> Self::Inner;
}
impl<T> ExplainUnwrap for Option<T> {
    type Inner = T;
    fn unwrap_assert(self, _explanation: &'static str) -> T {
        self.unwrap()
    }
    fn unwrap_test(self) -> T {
        self.unwrap()
    }
}
impl<T, E: Debug> ExplainUnwrap for Result<T, E> {
    type Inner = T;
    fn unwrap_assert(self, _explanation: &'static str) -> T {
        self.unwrap()
    }
    fn unwrap_test(self) -> T {
        self.unwrap()
    }
}
pub trait WithVariant {
    fn with_variant(self, variant: ErrorVariant) -> Self;
}
impl<T> WithVariant for Fallible<T> {
    fn with_variant(self, v: ErrorVariant) -> Self {
        self.map_err(|e| Error {
            variant: v,
            message: e.message,
            backtrace: e.backtrace,
        })
    }
}

#[cfg(test)]
mod test;
