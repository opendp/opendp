use std::fmt;
use std::fmt::Debug;

use backtrace::Backtrace as _Backtrace;

// create an instance of opendp::Fallible
#[macro_export]
macro_rules! fallible {
    ($variant:ident) => (Err(err!($variant)));
    ($variant:ident, $($inner:expr),+) => (Err(err!($variant, $($inner),+)));
}
// create an instance of opendp::Error
// "error" is shadowed, and breaks intellij macro resolution
#[macro_export]
macro_rules! err {
    // error without message
    ($variant:ident) => (crate::error::Error {
        variant: crate::error::ErrorVariant::$variant,
        message: None,
        backtrace: backtrace::Backtrace::new_unresolved()
    });
    // error with explicit message
    ($variant:ident, $message:expr) => (crate::error::Error {
        variant: crate::error::ErrorVariant::$variant,
        message: Some($message.to_string()), // ToString is impl'ed for String
        backtrace: backtrace::Backtrace::new_unresolved()
    });
    // args to format into message
    ($variant:ident, $template:expr, $($args:expr),+) =>
        (err!($variant, format!($template, $($args,)+)));
}

#[derive(thiserror::Error, Debug)]
pub struct Error {
    pub variant: ErrorVariant,
    pub message: Option<String>,
    pub backtrace: _Backtrace
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

    #[error("FailedRelation")]
    FailedRelation,

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

    #[error("InvalidDistance")]
    InvalidDistance,

    #[error("NotImplemented")]
    NotImplemented,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.variant)
    }
}

// simplify error creation from vega_lite_4
#[cfg(all(test, feature="test-plot"))]
impl From<String> for Error {
    fn from(v: String) -> Self {
        err!(FailedFunction, v)
    }
}

impl From<ErrorVariant> for Error {
    fn from(variant: ErrorVariant) -> Self {
        Self { variant, message: None, backtrace: _Backtrace::new() }
    }
}

impl<T> From<Error> for Result<T, Error> {
    fn from(e: Error) -> Self {
        Err(e)
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
