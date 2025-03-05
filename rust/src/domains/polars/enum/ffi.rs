use opendp_derive::bootstrap;
use polars::series::Series;

use crate::{
    core::FfiResult,
    ffi::any::{AnyDomain, AnyObject, Downcast},
};

use super::EnumDomain;

#[bootstrap(
    name = "enum_domain",
    arguments(categories(rust_type = "Series")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `EnumDomain`.
/// Can be used as an argument to a Polars series domain.
///
/// # Arguments
/// * `categories` - Optional ordered set of string categories
#[no_mangle]
pub extern "C" fn opendp_domains__enum_domain(
    categories: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {
    let categories = try_!(try_as_ref!(categories).downcast_ref::<Series>()).clone();
    Ok(AnyDomain::new(try_!(EnumDomain::new(categories)))).into()
}
