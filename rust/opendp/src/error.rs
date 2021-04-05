use std::fmt;

use backtrace::Backtrace as _Backtrace;


#[macro_export]
macro_rules! fallible {
    ($variant:ident) => (Err(err!($variant)));
    ($variant:ident, $($inner:expr),+) => (Err(err!($variant, $($inner),+)));
}
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
    #[error("{0}")]
    Raw(Box<dyn std::error::Error>),

    #[error("TypeParse")]
    TypeParse,

    #[error("Failed function execution")]
    FailedFunction,

    #[error("FailedRelation")]
    FailedRelation,

    #[error("RelationDebug")]
    RelationDebug,

    #[error("Unable to cast type")]
    FailedCast,

    #[error("Domain mismatch")]
    DomainMismatch,

    #[error("Failed to make transformation")]
    MakeTransformation,

    #[error("Failed to make measurement")]
    MakeMeasurement,

    #[error("Invalid distance")]
    InvalidDistance,

    #[error("Not implemented")]
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

pub type Fallible<T> = Result<T, Error>;

/// A trait for calling unwrap. Differs from unwrap in that the developer has made the assertion that failure should be unreachable
/// This is ok to use in two scenarios:
/// - Tests
/// - Code with unreachable None or Err variants
pub trait UnwrapAssert {
    type Inner;
    fn unwrap_assert(self) -> Self::Inner;
}
impl<T> UnwrapAssert for Option<T> {
    type Inner = T;
    fn unwrap_assert(self) -> T {
        self.unwrap()
    }
}
impl<T> UnwrapAssert for Fallible<T> {
    type Inner = T;
    fn unwrap_assert(self) -> T {
        self.unwrap()
    }
}
