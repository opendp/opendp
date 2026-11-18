//! Production atomic membership and nullability traits, shared with Verus.
#[cfg(verus_only)]
use vstd::prelude::*;
use std::fmt::Debug;
#[cfg(verus_only)]
use crate::atom_domain::Bounds;
#[cfg(not(verus_only))]
use crate::domains::Bounds;
#[cfg(verus_only)]
type Fallible<T> = Result<T, ()>;
#[cfg(not(verus_only))]
use crate::error::*;

pub trait CheckAtom: CheckNull + Sized + Clone + PartialEq + Debug + Send + Sync {
    fn is_bounded(&self, _bounds: Bounds<Self>) -> Fallible<bool> {
        #[cfg(not(verus_only))]
        return fallible!(FailedFunction, "bounds check is not implemented");
        #[cfg(verus_only)]
        return Err(());
    }
    fn check_member(&self, bounds: Option<Bounds<Self>>, nan: bool) -> Fallible<bool> {
        if let Some(bounds) = bounds {
            if !self.is_bounded(bounds)? {
                return Ok(false);
            }
        }
        if !nan && self.is_null() {
            return Ok(false);
        }
        Ok(true)
    }
}

/// Checks if a value is null.
///
/// Since [`crate::domains::AtomDomain`] may or may not contain null values,
/// this trait is necessary for its member check.
pub trait CheckNull {
    /// # Proof Definition
    /// A constant that is true if the type can represent null values.
    const NULLABLE: bool;

    /// # Proof Definition
    /// For any `self` of type `Self`, returns true if null.
    fn is_null(&self) -> bool {
        Self::NULLABLE
    }
}


impl CheckAtom for bool {}

impl CheckNull for bool {
    const NULLABLE: bool = false;
}
