use std::fmt;

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
        backtrace: backtrace::Backtrace::new()
    });
    // error with explicit message
    ($variant:ident, $message:expr) => (crate::error::Error {
        variant: crate::error::ErrorVariant::$variant,
        message: Some($message.to_string()), // ToString is impl'ed for String
        backtrace: backtrace::Backtrace::new()
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

#[derive(thiserror::Error, Debug)]
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
/// - unwrap_test: panics are acceptable in tests
/// - unwrap_assert: situations with unreachable None or Err variants
pub trait ExplainUnwrap {
    type Inner;
    /// use in tests
    fn unwrap_assert(self) -> Self::Inner;
    /// use if the alternate variant is structurally unreachable
    fn unwrap_test(self) -> Self::Inner;
}
impl<T> ExplainUnwrap for Option<T> {
    type Inner = T;
    fn unwrap_assert(self) -> T {
        self.unwrap()
    }
    fn unwrap_test(self) -> T {
        self.unwrap()
    }
}
impl<T> ExplainUnwrap for Fallible<T> {
    type Inner = T;
    fn unwrap_assert(self) -> T {
        self.unwrap()
    }
    fn unwrap_test(self) -> T {
        self.unwrap()
    }
}
