use polars::prelude::AnyValue;

use crate::{core::Domain, error::Fallible};

#[cfg(feature = "ffi")]
mod ffi;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownValueDomain;

/// # Proof Definition
/// UnknownValueDomain consists of Polars AnyValues,
/// with type restricted by the UnknownKind descriptor.
impl Domain for UnknownValueDomain {
    type Carrier = AnyValue<'static>;

    fn member(&self, _val: &Self::Carrier) -> Fallible<bool> {
        Ok(true)
    }
}
