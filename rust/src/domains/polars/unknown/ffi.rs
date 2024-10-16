use opendp_derive::bootstrap;

use crate::{core::FfiResult, ffi::any::AnyDomain};

use super::UnknownValueDomain;

#[bootstrap(
    name = "unknown_value_domain",
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `UnknownValueDomain`.
#[no_mangle]
pub extern "C" fn opendp_domains__unknown_value_domain() -> FfiResult<*mut AnyDomain> {
    Ok(AnyDomain::new(UnknownValueDomain)).into()
}
