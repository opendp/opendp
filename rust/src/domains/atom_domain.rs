//! Actual OpenDP AtomDomain representation, default, and membership.
//! Production and Verus import this one source; other convenience methods
//! remain in domains/mod.rs.
#[cfg(verus_only)]
use vstd::prelude::*;
use std::ops::Bound;
#[cfg(verus_only)]
use crate::{atom_traits::CheckAtom, Domain};
#[cfg(not(verus_only))]
use crate::{traits::CheckAtom, core::Domain};
#[cfg(verus_only)]
type Fallible<T> = Result<T, ()>;
#[cfg(not(verus_only))]
use crate::error::*;

#[cfg_attr(verus_only, verus_verify)]
#[cfg_attr(verus_only, derive(Debug))]
#[derive(Clone, PartialEq)]
pub struct AtomDomain<T: CheckAtom> {
    pub bounds: Option<Bounds<T>>,
    pub(super) nan: bool,
}

#[cfg_attr(verus_only, verus_verify(external))]
impl<T: CheckAtom> Default for AtomDomain<T> {
    fn default() -> Self {
        AtomDomain {
            bounds: None,
            nan: T::NULLABLE,
        }
    }
}
#[cfg_attr(verus_only, verus_verify)]
impl<T: CheckAtom> Domain for AtomDomain<T> {
    type Carrier = T;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        val.check_member(self.bounds.clone(), self.nan)
    }
}

#[cfg_attr(verus_only, verus_verify)]
#[cfg_attr(verus_only, derive(Debug))]
#[derive(Clone, PartialEq)]
pub struct Bounds<T> {
    pub(super) lower: Bound<T>,
    pub(super) upper: Bound<T>,
}

#[cfg_attr(verus_only, verus_verify)]
impl<T: CheckAtom> AtomDomain<T> {
    /// A non-null atomic domain with no bounds. For Boolean inputs this
    /// is identical to AtomDomain::<bool>::default(), as bool is non-null.
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures result.bounds.is_none() && !result.nan,
    ))]
    pub fn new_non_nan() -> Self {
        AtomDomain { bounds: None, nan: false }
    }
}
