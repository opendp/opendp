use std::backtrace::Backtrace as _Backtrace;
use std::fmt;

use dashu::base::ConversionError;

#[macro_export]
macro_rules! fallible {
    ($variant:ident) => (Err(err!($variant)));
    ($variant:ident, $($inner:expr),+) => (Err(err!($variant, $($inner),+)));
}

#[macro_export]
macro_rules! err {
    ($variant:ident) => ($crate::error::Error {
        variant: $crate::error::ErrorVariant::$variant,
        message: None,
        backtrace: err!(@backtrace)
    });
    ($variant:ident, $message:expr) =>
        (err!(@new $variant, format!($message)));
    ($variant:ident, $template:expr, $($args:expr),+) =>
        (err!(@new $variant, format!($template, $($args,)+)));

    (@new $variant:ident, $message:expr) => ($crate::error::Error {
        variant: $crate::error::ErrorVariant::$variant,
        message: Some($message),
        backtrace: err!(@backtrace)
    });

    (@backtrace) => (std::backtrace::Backtrace::capture());
}

pub struct Error {
    pub variant: ErrorVariant,
    pub message: Option<String>,
    pub backtrace: _Backtrace,
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let message = self
            .message
            .as_ref()
            .map(|value| format!("({value:?})"))
            .unwrap_or_default();
        write!(f, "{:?}{message}\n{}", self.variant, self.backtrace)
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.variant == other.variant && self.message == other.message
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum ErrorVariant {
    FailedFunction,
    FailedCast,
    EntropyExhausted,
}

impl fmt::Display for ErrorVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorVariant::FailedFunction => f.write_str("FailedFunction"),
            ErrorVariant::FailedCast => f.write_str("FailedCast"),
            ErrorVariant::EntropyExhausted => f.write_str("EntropyExhausted"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.variant)
    }
}

impl std::error::Error for Error {}

impl From<ErrorVariant> for Error {
    fn from(variant: ErrorVariant) -> Self {
        Self {
            variant,
            message: None,
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }
}

impl From<ConversionError> for Error {
    fn from(conversion_error: ConversionError) -> Self {
        Self {
            variant: ErrorVariant::FailedCast,
            message: Some(conversion_error.to_string()),
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }
}

pub type Fallible<T> = Result<T, Error>;
