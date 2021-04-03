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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.variant)
    }
}


#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ErrorVariant {
    #[error("{0}")]
    Raw(Box<dyn std::error::Error>),

    #[error("Failed function execution")]
    FailedFunction,

    #[error("Failed relation")]
    FailedRelation,

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

impl From<ErrorVariant> for Error {
    fn from(variant: ErrorVariant) -> Self {
        Self { variant, message: None, backtrace: _Backtrace::new() }
    }
}

pub type Fallible<T> = Result<T, Error>;
