use opendp_derive::bootstrap;

use crate::{core::FfiResult, ffi::any::AnyDomain};

use super::CategoricalDomain;

#[bootstrap(
    name = "categorical_domain",
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct a categorical domain without assuming any fixed encoding.
/// Can be used as an argument to a Polars series domain.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__categorical_domain() -> FfiResult<*mut AnyDomain> {
    Ok(AnyDomain::new(CategoricalDomain)).into()
}
