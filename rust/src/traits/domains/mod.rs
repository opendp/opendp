use crate::{core::Domain, domains::VectorDomain, error::Fallible};

#[cfg(feature = "ffi")]
mod ffi;

pub trait IsSizedDomain: Domain {
    /// # Proof Definition
    /// Returns Ok(size), if all members of the domain (`self`) have `size` elements.
    /// Otherwise returns `Err(e)`.
    fn get_size(&self) -> Fallible<usize>;
}

impl<D: Domain> IsSizedDomain for VectorDomain<D> {
    fn get_size(&self) -> Fallible<usize> {
        self.size.ok_or_else(|| {
            err!(
                FailedFunction,
                "elements of the vector domain have unknown size"
            )
        })
    }
}
